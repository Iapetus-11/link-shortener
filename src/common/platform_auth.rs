use poem::{
    http::StatusCode,
    web::headers::{self, HeaderMapExt},
};
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    common::argon2::{argon2_check_key_against_hash, argon2_hash_key, setup_strong_argon2},
    db::platforms::{Platform, get_platform},
};

/// Returns true if the api key matches that of the provided platform
pub fn check_platform_api_key(platform: &Platform, api_key: &str) -> bool {
    let argon2 = setup_strong_argon2();
    argon2_check_key_against_hash(&argon2, api_key, &platform.api_key_hash)
}

pub struct PlatformApiKeyAndHash {
    pub api_key: String,
    pub api_key_hash: String,
}

/// Generate a platform API key and API key hash
pub fn generate_platform_api_key() -> PlatformApiKeyAndHash {
    let api_key = Alphanumeric.sample_string(&mut rand::rng(), 69);

    let argon2 = setup_strong_argon2();
    let api_key_hash = argon2_hash_key(&argon2, &api_key);

    PlatformApiKeyAndHash {
        api_key,
        api_key_hash,
    }
}

#[derive(Debug)]
pub struct AuthedPlatform(pub Platform);

impl<'a> poem::FromRequest<'a> for AuthedPlatform {
    async fn from_request(
        req: &'a poem::Request,
        _body: &mut poem::RequestBody,
    ) -> poem::Result<Self> {
        let basic_auth = req
            .headers()
            .typed_try_get::<headers::Authorization<headers::authorization::Basic>>();

        let basic_auth = match basic_auth {
            Err(_) => Err(poem::Error::from_string(
                "invalid authorization header (must use basic auth syntax)",
                StatusCode::UNAUTHORIZED,
            )),
            Ok(None) => Err(poem::Error::from_string(
                "missing authorization header",
                StatusCode::UNAUTHORIZED,
            )),
            Ok(Some(basic_auth)) => Ok(basic_auth),
        }?;

        let Ok(platform_id) = basic_auth.username().parse::<uuid::Uuid>() else {
            return Err(poem::Error::from_string(
                "basic auth username must be a valid platform ID",
                StatusCode::UNAUTHORIZED,
            ));
        };

        let mut db = req.data::<sqlx::PgPool>().unwrap().acquire().await.unwrap();

        let Some(platform) = get_platform(&mut db, &platform_id).await.unwrap() else {
            return Err(poem::Error::from_string(
                "invalid credentials",
                StatusCode::UNAUTHORIZED,
            ));
        };

        if !check_platform_api_key(&platform, basic_auth.password()) {
            return Err(poem::Error::from_string(
                "invalid credentials",
                StatusCode::UNAUTHORIZED,
            ));
        }

        Ok(AuthedPlatform(platform))
    }
}

#[cfg(test)]
mod tests {
    use poem::{FromRequest, web::headers::Authorization};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;

    use crate::{
        common::testing::{app::platform_auth_header, db::PgPoolConn},
        db::platforms::create_platform,
    };

    #[sqlx::test]
    async fn test_generate_and_check_api_key(mut db: PgPoolConn) {
        let (api_key, platform) = create_platform(&mut db, "Villager Bot").await.unwrap();

        assert!(check_platform_api_key(&platform, &api_key));
    }

    #[sqlx::test]
    async fn test_from_request_missing_auth_header(db_pool: PgPool) {
        let result = AuthedPlatform::from_request_without_body(
            &poem::Request::builder().extension(db_pool).finish(),
        )
        .await;

        let Err(error) = result else {
            panic!("Expected error result");
        };

        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.to_string(), "missing authorization header");
    }

    #[sqlx::test]
    async fn test_from_request_invalid_basic_auth_syntax(db_pool: PgPool) {
        for header_value in [
            "Basic",
            "Basic ",
            "Basic invalid",
            "Basic ZGVleiBudXR6",
            "Basic !@#$%^&&*()-=_+,./<>?[]\\|}{",
        ] {
            let error = AuthedPlatform::from_request_without_body(
                &poem::Request::builder()
                    .header("Authorization", header_value)
                    .extension(db_pool.clone())
                    .finish(),
            )
            .await
            .unwrap_err();

            assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                error.to_string(),
                "invalid authorization header (must use basic auth syntax)"
            );
        }
    }

    #[sqlx::test]
    async fn test_from_request_non_uuid_platform_id(db_pool: PgPool) {
        for platform_id_value in ["", "username", "ca15ae9-4d7-46f-a4f-605b42aab03"] {
            let error = AuthedPlatform::from_request_without_body(
                &poem::Request::builder()
                    .extension(db_pool.clone())
                    .typed_header(Authorization::basic(platform_id_value, "password"))
                    .finish(),
            )
            .await
            .unwrap_err();

            assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                error.to_string(),
                "basic auth username must be a valid platform ID"
            );
        }
    }

    #[sqlx::test]
    async fn test_from_request_platform_not_found(db_pool: PgPool) {
        let mut db = db_pool.acquire().await.unwrap();

        let (api_key, _) = create_platform(&mut db, "test_patience").await.unwrap();

        let error = AuthedPlatform::from_request_without_body(
            &poem::Request::builder()
                .extension(db_pool)
                .typed_header(platform_auth_header(&Uuid::now_v7(), &api_key))
                .finish(),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.to_string(), "invalid credentials");
    }

    #[sqlx::test]
    async fn test_from_request_invalid_api_key(db_pool: PgPool) {
        let mut db = db_pool.acquire().await.unwrap();

        let (api_key, platform) = create_platform(&mut db, "test_patience").await.unwrap();

        let mut api_key_off_by_one_char = api_key.clone();
        api_key_off_by_one_char.pop();
        api_key_off_by_one_char.push(if api_key_off_by_one_char.ends_with('A') {
            'B'
        } else {
            'A'
        });

        for api_key_value in [api_key_off_by_one_char.as_str(), "", "test"] {
            let error = AuthedPlatform::from_request_without_body(
                &poem::Request::builder()
                    .extension(db_pool.clone())
                    .typed_header(platform_auth_header(&platform.id, api_key_value))
                    .finish(),
            )
            .await
            .unwrap_err();

            assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(error.to_string(), "invalid credentials");
        }
    }

    #[sqlx::test]
    async fn test_from_request_valid_auth_header(db_pool: PgPool) {
        let mut db = db_pool.acquire().await.unwrap();

        let (api_key, platform) = create_platform(&mut db, "Dev-Milo").await.unwrap();

        let AuthedPlatform(authed_platform) = AuthedPlatform::from_request_without_body(
            &poem::Request::builder()
                .extension(db_pool)
                .typed_header(platform_auth_header(&platform.id, &api_key))
                .finish(),
        )
        .await
        .unwrap();

        assert_eq!(authed_platform.id, platform.id);
        assert_eq!(authed_platform.name, platform.name);
        assert_eq!(authed_platform.api_key_hash, platform.api_key_hash);
    }
}
