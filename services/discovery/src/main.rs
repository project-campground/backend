use anyhow::Result;

#[macro_use] extern crate rocket;
extern crate thiserror;

pub mod database;
pub mod config;

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build()
        .mount("/", routes![]);

    rocket.launch().await?;

    Ok(())
}