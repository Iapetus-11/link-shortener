use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Platform {
    pub id: Uuid,
    pub name: String,
    pub api_key_hash: String,
}

/// Returns true if the api key matches that of the provided platform
pub fn check_platform_api_key(platform: &Platform, api_key: &str) -> bool {
    let argon2 = Argon2::default();

    argon2
        .verify_password(
            api_key.as_bytes(),
            &PasswordHash::new(&platform.api_key_hash).unwrap(),
        )
        .is_ok()
}

pub struct PlatformApiKeyAndHash {
    api_key: String,
    api_key_hash: String,
}

/// Generate a platform API key and API key hash
pub fn generate_platform_api_key() -> PlatformApiKeyAndHash {
    let api_key = Alphanumeric.sample_string(&mut rand::rng(), 69);

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let api_key_hash = argon2
        .hash_password(api_key.as_bytes(), &salt)
        .unwrap()
        .to_string();

    PlatformApiKeyAndHash {
        api_key,
        api_key_hash,
    }
}

/// Creates a Platform, returning the unhashed API key and an object holding the Platform's data
pub async fn create_platform(
    db: &mut PgConnection,
    name: String,
) -> sqlx::Result<(String, Platform)> {
    let PlatformApiKeyAndHash {
        api_key,
        api_key_hash,
    } = generate_platform_api_key();

    let platform = sqlx::query_as!(
        Platform,
        "INSERT INTO platforms (id, name, api_key_hash) VALUES ($1, $2, $3) RETURNING id, name, api_key_hash;",
        uuid::Uuid::now_v7(),
        name,
        api_key_hash,
    ).fetch_one(&mut *db).await?;

    Ok((api_key, platform))
}

/// Fetch a platform from the DB by its ID
pub async fn get_platform(db: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            SELECT id, name, api_key_hash FROM platforms WHERE id = $1;
        "#,
        id,
    )
    .fetch_optional(&mut *db)
    .await
}
