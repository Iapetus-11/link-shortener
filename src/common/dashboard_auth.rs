use poem::{
    http::StatusCode,
    web::headers::{self, HeaderMapExt},
};
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    common::argon2::hash_key,
    db::dashboard_login_token::{check_dashboard_login_token, get_dashboard_login_token},
};

pub struct GenerateLoginTokenAndHash {
    pub token: String,
    pub hash: String,
}

pub fn generate_login_token() -> GenerateLoginTokenAndHash {
    let token = Alphanumeric.sample_string(&mut rand::rng(), 96);
    let hash = hash_key(&token);

    GenerateLoginTokenAndHash { token, hash }
}

#[derive(Debug)]
pub struct AuthedDashboardUser;

impl<'a> poem::FromRequest<'a> for AuthedDashboardUser {
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

        let Ok(token_id) = basic_auth.username().parse::<uuid::Uuid>() else {
            return Err(poem::Error::from_string(
                "basic auth username must be a valid token ID",
                StatusCode::UNAUTHORIZED,
            ));
        };

        let mut db = req.data::<sqlx::PgPool>().unwrap().acquire().await.unwrap();

        let Some(token_row) = get_dashboard_login_token(&mut db, &token_id).await.unwrap() else {
            return Err(poem::Error::from_string(
                "invalid credentials",
                StatusCode::UNAUTHORIZED,
            ));
        };

        if !check_dashboard_login_token(&token_row, basic_auth.password()) {
            return Err(poem::Error::from_string(
                "invalid credentials",
                StatusCode::UNAUTHORIZED,
            ));
        }

        Ok(AuthedDashboardUser)
    }
}
