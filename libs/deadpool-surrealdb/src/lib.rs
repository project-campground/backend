use deadpool::managed;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss}, opt::auth, Surreal
};

#[cfg(feature = "rocket")] pub use rocket_pool::SurrealDBPool;
#[cfg(feature = "rocket")] pub use rocket_pool::Config;

pub enum Credentials {
    Root {
        user: String,
        pass: String,
    },
    Namespace {
        user: String,
        pass: String,
        ns: String,
    },
    Database {
        user: String,
        pass: String,
        ns: String,
        db: String,
    },
    Scope {
        ns: String,
        db: String,
        sc: String,
        params: serde_json::Value,
    },
}

pub struct Manager {
    host: String,

    ns: String,
    db: String,
    creds: Credentials,
}

impl managed::Manager for Manager {
    type Type = Surreal<Client>;
    type Error = surrealdb::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let db = match self.host.clone().split_once("://") {
            Some(("ws", host)) => Surreal::new::<Ws>(host).await?,
            Some(("wss", host)) => Surreal::new::<Wss>(host).await?,
            _ => panic!("Invalid host"),
        };
        db.use_ns(self.ns.clone()).use_db(self.db.clone()).await?;
        match &self.creds {
            Credentials::Root { user, pass } => {
                db.signin(auth::Root { username: &user, password: &pass }).await?;
            },
            Credentials::Namespace { user, pass, ns } => {
                db.signin(auth::Namespace { username: &user, password: &pass, namespace: &ns }).await?;
            },
            Credentials::Database { user, pass, ns, db: database } => {
                db.signin(auth::Database { username: &user, password: &pass, namespace: &ns, database: &database }).await?;
            },
            Credentials::Scope { ns, db: database, sc, params } => {
                db.signin(auth::Scope { namespace: &ns, database: &database, scope: &sc, params }).await?;
            },
        }
        Ok(db)
    }

    async fn recycle(&self, conn: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        conn.use_ns(self.ns.clone()).use_db(self.db.clone()).await?;
        match &self.creds {
            Credentials::Root { user, pass } => {
                conn.signin(auth::Root { username: &user, password: &pass }).await?;
            },
            Credentials::Namespace { user, pass, ns } => {
                conn.signin(auth::Namespace { username: &user, password: &pass, namespace: &ns }).await?;
            },
            Credentials::Database { user, pass, ns, db: database } => {
                conn.signin(auth::Database { username: &user, password: &pass, namespace: &ns, database: &database }).await?;
            },
            Credentials::Scope { ns, db: database, sc, params } => {
                conn.signin(auth::Scope { namespace: &ns, database: &database, scope: &sc, params }).await?;
            },
        }
        Ok(())
    }
}

pub type Pool = managed::Pool<Manager>;

#[cfg(feature = "rocket")]
mod rocket_pool {
    use deadpool::{managed::{Object, Pool, PoolError}, Runtime};
    use serde::{Deserialize, Serialize};
    use rocket::figment::Figment;
    use rocket_db_pools::Error;
    use std::time::Duration;

    use crate::Credentials;

    #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(crate = "rocket::serde")]
    pub struct Config {
        /// Database-specific connection and configuration URL.
        ///
        /// The format of the URL is database specific; consult your database's
        /// documentation.
        pub url: String,
        /// Minimum number of connections to maintain in the pool.
        ///
        /// **Note:** `deadpool` drivers do not support and thus ignore this value.
        ///
        /// _Default:_ `None`.
        pub min_connections: Option<u32>,
        /// Maximum number of connections to maintain in the pool.
        ///
        /// _Default:_ `workers * 4`.
        pub max_connections: usize,
        /// Number of seconds to wait for a connection before timing out.
        ///
        /// If the timeout elapses before a connection can be made or retrieved from
        /// a pool, an error is returned.
        ///
        /// _Default:_ `5`.
        pub connect_timeout: u64,
        /// Maximum number of seconds to keep a connection alive for.
        ///
        /// After a connection is established, it is maintained in a pool for
        /// efficient connection retrieval. When an `idle_timeout` is set, that
        /// connection will be closed after the timeout elapses. If an
        /// `idle_timeout` is not specified, the behavior is driver specific but
        /// typically defaults to keeping a connection active indefinitely.
        ///
        /// _Default:_ `None`.
        pub idle_timeout: Option<u64>,
        /// The type of credentials to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub credentials_type: String,
        /// The username to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub user: Option<String>,
        /// The password to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub pass: Option<String>,
        /// The namespace to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub ns: String,
        /// The database to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub db: String,
        /// The scope to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub sc: Option<String>,
        /// The parameters to use when connecting to the database.
        ///
        /// _Default:_ `None`.
        pub params: Option<serde_json::Value>,
    }

    pub struct SurrealDBPool {
        pool: Pool<crate::Manager>,
    }

    #[rocket::async_trait]
    impl rocket_db_pools::Pool for SurrealDBPool {
        type Error = Error<PoolError<surrealdb::Error>>;
        type Connection = Object<crate::Manager>;

        async fn init(figment: &Figment) -> Result<Self, Self::Error> {
            let config: Config = figment.extract()?;

            let creds = match config.credentials_type.as_str() {
                "root" => Credentials::Root {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                },
                "namespace" => Credentials::Namespace {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                    ns: config.ns.clone(),
                },
                "database" => Credentials::Database {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                    ns: config.ns.clone(),
                    db: config.db.clone(),
                },
                "scope" => Credentials::Scope {
                    ns: config.ns.clone(),
                    db: config.db.clone(),
                    sc: config.sc.clone().unwrap_or_default(),
                    params: config.params.clone().unwrap_or_default(),
                },
                _ => panic!("Database is misconfigured"),
            };

            let manager = crate::Manager {
                host: config.url,
                ns: config.ns,
                db: config.db,
                creds,
            };

            let pool = Pool::builder(manager)
                .max_size(config.max_connections)
                .wait_timeout(Some(Duration::from_secs(config.connect_timeout)))
                .create_timeout(Some(Duration::from_secs(config.connect_timeout)))
                .recycle_timeout(config.idle_timeout.map(Duration::from_secs))
                .runtime(Runtime::Tokio1)
                .build()
                .map_err(|_| Error::Init(PoolError::NoRuntimeSpecified))?;
            
            Ok(SurrealDBPool { pool })
        }

        async fn get(&self) -> Result<Self::Connection, Self::Error> {
            self.pool.get().await.map_err(Error::Get)
        }

        async fn close(&self) {
            self.pool.close();
        }
    }

    #[cfg(test)]
    mod tests {
        use rocket::figment::providers::Serialized;
        use rocket_db_pools::Pool;

        use super::*;

        #[actix_rt::test]
        async fn test_rocket_pool() {
            let mut server = std::process::Command::new("surreal")
                .arg("start")
                .arg("memory")
                .arg("-A")
                .arg("--user")
                .arg("test")
                .arg("--pass")
                .arg("test")
                .arg("-b")
                .arg("0.0.0.0:8282")
                .spawn()
                .unwrap();
            
            // Wait for the server to start
            while !std::net::TcpStream::connect("localhost:8282").is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            let config = Config {
                url: "ws://localhost:8282".to_string(),
                min_connections: None,
                max_connections: 4,
                connect_timeout: 5,
                idle_timeout: None,
                credentials_type: "root".to_string(),
                user: Some("test".to_string()),
                pass: Some("test".to_string()),
                ns: "test".to_string(),
                db: "test".to_string(),
                sc: None,
                params: None,
            };

            let pool = SurrealDBPool::init(&Figment::from(Serialized::from(&config, "default"))).await.unwrap();
            let conn = pool.get().await.unwrap();
            let res: Result<Option<serde_json::Value>, surrealdb::Error> = conn.query("INFO FOR DB").await.unwrap().take(0);
            assert!(res.is_ok());

            let _ = pool.close();
            server.kill().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_manager() {
        let mut server = std::process::Command::new("surreal")
            .arg("start")
            .arg("memory")
            .arg("-A")
            .arg("--user")
            .arg("test")
            .arg("--pass")
            .arg("test")
            .arg("-b")
            .arg("0.0.0.0:8282")
            .spawn()
            .unwrap();
        
        // Wait for the server to start
        while !std::net::TcpStream::connect("localhost:8282").is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        let manager = Manager {
            host: "ws://localhost:8282".to_string(),
            ns: "test".to_string(),
            db: "test".to_string(),
            creds: Credentials::Root {
                user: "test".to_string(),
                pass: "test".to_string(),
            },
        };
        let pool = Pool::builder(manager).build().unwrap();
        let conn = pool.get().await.unwrap();
        let res: Result<Option<serde_json::Value>, surrealdb::Error> = conn.query("INFO FOR DB").await.unwrap().take(0);
        assert!(res.is_ok());

        pool.close();
        server.kill().unwrap();
    }
}
