# Campground back-end

This repository contains Campground's back-end, specifically appview, as well as other developer environment technologies like Atprotocol PDS.

This is a [Rust](https://www.rust-lang.org/) project that utilizes [PostgreSQL](https://www.postgresql.org/), [Diesel](https://diesel.rs/), [SeaweedFS](https://github.com/seaweedfs/seaweedfs), [AT Protocol](https://atproto.com/), and the [Signal Protocol](https://signal.org/docs/).

## Getting Started

### PDS

You can use Bluesky's, Eurosky's or Blacksky's PDS on the front-end, as local PDS not only architecturally equivalent, but is using same exact code on Docker, but runs locally. While it is recommended to use local PDS for testing, you can still use existing PDS by checking what PDS you are on with [pds.ls](https://pds.ls).

If you are choosing existing PDS instead of local PDS, skip this step.

1. Install [Docker](https://www.docker.com/) and [Docker-Compose](https://docs.docker.com/compose/)

2. Make sure Docker daemon is running
    - On Linux, start Docker daemon by doing:
    ```bash
    # Systemd
    sudo systemctl start docker.service
    # OpenRC
    sudo rc-service docker start
    ```
3. To start PDS, run the following command:

```bash
docker-compose -f ./dev-env/docker-compose.yml up
# With sudo or doas on Linux:
sudo docker-compose -f ./dev-env/docker-compose.yml up
```

### Appview setup

1. Install [Rust](https://www.rust-lang.org/) ([Rustup](https://rustup.rs/) preferred), [PostgreSQL](https://www.postgresql.org/), and [Diesel](https://diesel.rs/).

2. Make sure PostgreSQL works
    - On Windows, you possibly need to run this command after installing PostgreSQL:
    ```bash
    setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\VER_NUM\lib
    ```

    - On Linux, make sure PostgreSQL daemon is running:
    ```bash
    # Systemd
    sudo systemctl start postgresql
    # OpenRC
    sudo rc-service postgresql start
    ```

Where VER_NUM is the number of the folder after the PostgreSQL folder

3. PostgreSQL may also require the same directory to be added to PATH. Add the same path shown in the above command to PATH, so that apps may be able to access the Postgres libraries properly.

4. Create a user/role and a database in PostgreSQL:
    - [Create user Linux](https://phoenixnap.com/kb/postgres-create-user)
    - [Create user Windows](https://stackoverflow.com/questions/5189026/how-to-add-a-user-to-postgresql-in-windows)
    - [Create database](https://www.geeksforgeeks.org/postgresql/postgresql-create-database/) (MAKE SURE THE USER IS THE OWNER OF THE DATABASE OR HAS APPROPRIATE PERMISSIONS VIA `GRANT ALL PRIVILEGES`)

5. Setup `Rocket.toml` by copying `services/appview/Rocket.example.toml` as `services/appview/Rocket.example.toml`. You can get verification_key manually through OpenSSL or by doing:
    ```bash
    cd ./cmd/backend_manager
    cargo run
    ```
    typing arbitrary values for DB URL and port like example and 10, then selecting menu item 2, then copying private key

### Run the Appview

When Docker and PostgreSQL daemons/drivers are running, you can turn on appview by doing the following:

```bash
cd ./services/appview
cargo run
```

### And it should be done!
