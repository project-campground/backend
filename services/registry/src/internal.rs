use rocket::{request::FromRequest, State, Config, fairing::Fairing};

use crate::config::ServiceConfig;

pub struct InternalStartup {}

#[rocket::async_trait]
impl Fairing for InternalStartup {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Internal Startup",
            kind: rocket::fairing::Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &rocket::Rocket<rocket::Orbit>) {
        let figment = rocket.figment();
        let cfg: Config = figment.extract().unwrap();
        let service_cfg = rocket.state::<ServiceConfig>().unwrap();

        let ip = cfg.address.to_string();
        let port = cfg.port;
        let base = format!("http://{}:{}/internal", ip, port);

        let builder = reqwest::ClientBuilder::new();
        let builder = builder.default_headers(
            reqwest::header::HeaderMap::from_iter([
                (reqwest::header::AUTHORIZATION, service_cfg.secret_key.parse().unwrap()),
            ])
        );
        let client = builder.build().unwrap();

        client.post(format!("{}/migrate-db", base)).send().await.unwrap();
    }
}

pub struct InternalEndpoint {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for InternalEndpoint {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let cfg: &State<ServiceConfig> = request.guard().await.unwrap();
        let key = &cfg.secret_key;
        let auth = request.headers().get_one("Authorization");
        if auth.is_none() {
            return rocket::request::Outcome::Error((rocket::http::Status::Unauthorized, ()));
        }
        let auth = auth.unwrap();
        if auth != key {
            return rocket::request::Outcome::Error((rocket::http::Status::Unauthorized, ()));
        }

        rocket::request::Outcome::Success(InternalEndpoint {})
    }
}

pub fn routes() -> Vec<rocket::Route> {
    let db_routes = crate::database::routes();

    let mut routes = Vec::new();
    routes.extend(db_routes);

    routes
}