use poem::{
    EndpointExt, Server,
    listener::TcpListener,
    middleware::{AddData, CatchPanic, NormalizePath, Tracing, TrailingSlash},
};
use sqlx::{ConnectOptions, Connection};

use crate::{common::cli::take_input, config::CONFIG, db::platforms::create_platform};
use std::{env, error::Error as StdError};

mod common;
mod config;
mod db;
mod routes;

async fn run_api() -> Result<(), Box<dyn StdError>> {
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(CONFIG.database_pool_size)
        .connect(&CONFIG.database_url)
        .await?;

    let app = routes::routes()
        .with(Tracing)
        .with(NormalizePath::new(TrailingSlash::Always))
        .with(AddData::new(db_pool))
        .with(CatchPanic::new());

    Server::new(TcpListener::bind(CONFIG.host_address.clone()))
        .run(app)
        .await?;

    Ok(())
}

async fn run_create_platform() -> Result<(), Box<dyn StdError>> {
    println!("Enter platform details, press enter to advance:");
    let platform_name = take_input("Name: ")?;

    let mut db = sqlx::postgres::PgConnection::connect(&CONFIG.database_url).await?;

    create_platform(&mut db, platform_name).await?;

    println!("Platform successfully completed!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or("".into());

    match command.as_str() {
        "api" => run_api().await.unwrap(),
        "create_platform" => run_create_platform().await.unwrap(),
        "" => panic!("You must type a command, one of: api, create_platform"),
        unknown_command => {
            panic!("Unknown command {unknown_command}, you must type one of: api, create_platform")
        }
    };

    Ok(())
}
