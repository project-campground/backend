# Campground Registry
This service is an [ATProto][] PDS implemented in rust, extended with lexicons for campground.

## Dependencies
`Postgres` - https://www.postgresql.org/download/
`Diesel CLI` - https://diesel.rs/guides/getting-started#installing-diesel-cli

## Configuration
Copy `Rocket.example.toml` as `Rocket.toml` and configure the fields your service. There should be no reason to change the `identity.plc_url` and `bsky_app_view` configurations from what's in the example config unless you know what you are doing.

Both `email` and `mod_email` support either `SMTP` or `Mailgun` as providers and have example configuration for either provider.

The registry expects all secret keys to be hex-encoded `secp256k1` private keys, which can easily be generated using tools like [ECDSA Key Generator](https://emn178.github.io/online-tools/ecdsa/key-generator/)

In addition to the Rocket.toml file, you can also use environment variables prefixed with `ROCKET_` to specify configuration values.

## Running
Before running the project, if you haven't already, you need to run `diesel migration run` in this directory to setup the database.

Once the database is setup you can run the project using `cargo run`.

[atproto]: https://atproto.com/