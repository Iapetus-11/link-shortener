use poem::{
    http::StatusCode,
    web::headers::{self, HeaderMapExt},
};

use crate::db::platforms::{Platform, check_platform_api_key, get_platform};

pub struct AuthedPlatform(pub Platform);

impl<'a> poem::FromRequest<'a> for AuthedPlatform {
    async fn from_request(
        req: &'a poem::Request,
        _body: &mut poem::RequestBody,
    ) -> poem::Result<Self> {
        let mut db = req.data::<sqlx::PgPool>().unwrap().acquire().await.unwrap();

        let basic_auth = req
            .headers()
            .typed_try_get::<headers::Authorization<headers::authorization::Basic>>();

        let basic_auth = match basic_auth {
            Err(error) => Err(poem::Error::from_string(
                error.to_string(),
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
                "basic auth username must be a platform ID",
                StatusCode::UNAUTHORIZED,
            ));
        };

        let Some(platform) = get_platform(&mut db, platform_id).await.unwrap() else {
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
