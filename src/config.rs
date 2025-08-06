use std::{any::type_name, env, str::FromStr, sync::LazyLock};

pub struct Config {
    pub database_url: String,
    pub database_pool_size: u32,
    pub host_address: String,
    pub admin_password_hash: String,
    pub admin_login_expires_after_seconds: u64,
}

fn get_env<T: FromStr>(key: &str) -> T {
    let string = env::var(key).unwrap_or_else(|_| panic!("Please set {key} in your .env"));

    let parsed = string.parse::<T>();

    match parsed {
        Ok(value) => value,
        Err(_) => {
            let type_name = type_name::<T>();
            panic!("Expected {key} to be a valid {type_name} in your .env");
        }
    }
}

#[cfg(not(test))]
fn load() -> Config {
    use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

    dotenvy::dotenv().unwrap();

    let database_url: String = get_env("DATABASE_URL");
    let database_pool_size: u32 = get_env("DATABASE_POOL_SIZE");
    let host_address: String = get_env("HOST_ADDRESS");
    let admin_password_hash: String = String::from_utf8_lossy(
        &BASE64_URL_SAFE_NO_PAD
            .decode(get_env::<String>("ADMIN_PASSWORD_HASH"))
            .expect("ADMIN_PASSWORD_HASH should be a base64 encoded argon2id password hash"),
    )
    .into();
    let admin_login_expires_after_seconds: u64 = get_env("ADMIN_LOGIN_EXPIRES_AFTER_SECONDS");

    Config {
        database_url,
        database_pool_size,
        host_address,
        admin_password_hash,
        admin_login_expires_after_seconds,
    }
}

#[cfg(test)]
fn load() -> Config {
    use crate::common::argon2::setup_strong_argon2;
    use argon2::{
        PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };

    let database_url: String = get_env("DATABASE_URL");
    let database_pool_size: u32 = 1;
    let host_address: String = "localhost:8000".to_string();
    let admin_password_hash: String = setup_strong_argon2()
        .hash_password("password".as_bytes(), &SaltString::generate(OsRng))
        .unwrap()
        .to_string();
    let admin_login_expires_after_seconds: u64 = 3600;

    Config {
        database_url,
        database_pool_size,
        host_address,
        admin_password_hash,
        admin_login_expires_after_seconds,
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(load);
