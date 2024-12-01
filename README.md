This is a [Rust](https://www.rust-lang.org/) project that utilizes [PostgreSQL](https://www.postgresql.org/), [SeaweedFS](https://github.com/seaweedfs/seaweedfs), [AT Protocol](https://atproto.com/), and the [Signal Protocol](https://signal.org/docs/).

## Getting Started

1) Install Rust and Postgres from the links above.

2) Run the below command

```bash
setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\VER_NUM\lib
```

Where VER_NUM is the number of the folder after the PostgreSQL folder
 
3) Run one of the two below commands

When you want to run in the root backend folder
```bash
cargo run --bin campground-registry
```

When you want to run inside of the /services/registry folder
```bash
cargo run
```

If you encounter any errors with Rust or PostgreSQL and resolve them, you must run the below command before proceeding:
```bash
cargo clean
```
