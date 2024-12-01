This is a [Rust](https://www.rust-lang.org/) project that utilizes [PostgreSQL](https://www.postgresql.org/), [Diesel](https://diesel.rs/), [SeaweedFS](https://github.com/seaweedfs/seaweedfs), [AT Protocol](https://atproto.com/), and the [Signal Protocol](https://signal.org/docs/).

## Getting Started

1) Install Rust, Postgres, and Diesel from the links above.

2) Run the below command

```bash
setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\VER_NUM\lib
```

Where VER_NUM is the number of the folder after the PostgreSQL folder

From there, follow the directions within the service folders to run each individual service.

If you encounter any errors with Rust or PostgreSQL and resolve them, you must run the below command before proceeding:
```bash
cargo clean
```
