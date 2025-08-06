use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use poem::{
    EndpointExt, Server,
    listener::TcpListener,
    middleware::{AddData, CatchPanic, NormalizePath, Tracing, TrailingSlash},
};
use sqlx::{migrate::Migrator, Connection};

use crate::{
    common::{
        argon2::{argon2_hash_key, setup_strong_argon2},
        cli::take_input,
    },
    config::CONFIG,
    db::platforms::create_platform,
};
use std::{env, error::Error as StdError};

mod common;
mod config;
mod db;
mod routes;

static DB_MIGRATOR: Migrator = sqlx::migrate!();

async fn run_app() -> Result<(), Box<dyn StdError>> {
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

/// For local development use `sqlx migrate run` via the sqlx CLI, this is intended for use when deploying
async fn run_migrate_db() -> Result<(), Box<dyn StdError>> {
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&CONFIG.database_url)
        .await?;

    DB_MIGRATOR.run(&db_pool).await.unwrap();

    Ok(())
}

async fn run_create_platform() -> Result<(), Box<dyn StdError>> {
    println!("Enter platform details, press enter to advance:");
    let platform_name = take_input("Name: ")?;

    let mut db = sqlx::postgres::PgConnection::connect(&CONFIG.database_url).await?;

    let (api_key, platform) = create_platform(&mut db, &platform_name).await?;

    println!("Platform successfully completed!");
    println!("ID: {}", platform.id);
    println!("API Key: {api_key}");

    Ok(())
}

fn run_hash_admin_password() -> Result<(), Box<dyn StdError>> {
    let password = take_input("Password: ")?;

    let argon2 = setup_strong_argon2();
    let hashed = BASE64_URL_SAFE_NO_PAD.encode(argon2_hash_key(&argon2, &password));

    println!("Password Hash: {hashed}");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or("".into());

    match command.as_str() {
        "app" => run_app().await.unwrap(),
        "migrate_db" => run_migrate_db().await.unwrap(),
        "create_platform" => run_create_platform().await.unwrap(),
        "hash_admin_password" => run_hash_admin_password().unwrap(),
        "" => panic!("You must type a command, one of: api, create_platform, hash_admin_password"),
        unknown_command => {
            panic!("Unknown command {unknown_command}, you must type one of: api, create_platform")
        }
    };

    Ok(())
}
