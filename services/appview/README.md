# Campground Registry

This service is an [ATProto][] Appview implemented in rust.

## Dependencies

`Postgres` - https://www.postgresql.org/download/
`Diesel CLI` - https://diesel.rs/guides/getting-started#installing-diesel-cli

## Configuration

Copy `Rocket.example.toml` as `Rocket.toml` and configure the fields your service. There should be no reason to change the `identity.plc_url` configuration from what's in the example config unless you know what you are doing.

Change database URL under `[default.database]`, as well as verification_key, which has a comment indicating how you can generate it.

`email` supports either `SMTP` or `Mailgun` as providers and have example configuration for either provider.

In addition to the Rocket.toml file, you can also use environment variables prefixed with `ROCKET_` to specify configuration values.

## Running

Before running the project, if you haven't already, you need to run `diesel migration run` in this directory to setup the database.

Once the database is setup you can run the project using `cargo run`.

[atproto]: https://atproto.com/
