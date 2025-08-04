use std::{any::type_name, env, str::FromStr, sync::LazyLock};

pub struct Config {
    pub database_url: String,
    pub database_pool_size: u32,
    pub host_address: String,
    pub admin_password_hash: String,
    pub admin_login_expires_after_seconds: u32,
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

fn load() -> Config {
    dotenvy::dotenv().unwrap();

    let database_url: String = get_env("DATABASE_URL");
    let database_pool_size: u32 = get_env("DATABASE_POOL_SIZE");
    let host_address: String = get_env("HOST_ADDRESS");
    let admin_password_hash: String = get_env("ADMIN_PASSWORD_HASH");
    let admin_login_expires_after_seconds: u32 = get_env("ADMIN_LOGIN_EXPIRES_AFTER_SECONDS");

    Config {
        database_url,
        database_pool_size,
        host_address,
        admin_password_hash,
        admin_login_expires_after_seconds,
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(load);
