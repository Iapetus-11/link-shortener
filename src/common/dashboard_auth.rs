use poem::{session::Session, web::Redirect, Endpoint, IntoResponse};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::error::Error as StdError;
use uuid::Uuid;

use crate::{
    common::argon2::{check_key_against_hash, hash_key},
    config::CONFIG,
    db::dashboard_login_token::{create_dashboard_login_token, get_dashboard_login_token},
};

pub const DASHBOARD_SESSION_TOKEN_DATA_KEY: &str = "DASHBOARD_SESSION_TOKEN";

#[derive(Serialize, Deserialize)]
struct DashboardSessionLoginTokenData {
    id: Uuid,
    token: String,
}

/// Checks the password, returning `false` if incorrect, otherwise updating the session with a new token and returning `true`
pub async fn attempt_log_in_dashboard_session(
    db: &mut PgConnection,
    session: &Session,
    password: &str,
) -> Result<bool, Box<dyn StdError>> {
    if !check_key_against_hash(password, &CONFIG.admin_password_hash) {
        return Ok(false);
    }

    let GenerateLoginTokenAndHash {
        token,
        hash: token_hash,
    } = generate_dashboard_login_token();

    let token_row = create_dashboard_login_token(&mut *db, &token_hash).await?;

    session.set(
        DASHBOARD_SESSION_TOKEN_DATA_KEY,
        &DashboardSessionLoginTokenData {
            id: token_row.id,
            token: token.to_string(),
        },
    );

    Ok(true)
}

pub struct GenerateLoginTokenAndHash {
    pub token: String,
    pub hash: String,
}
pub fn generate_dashboard_login_token() -> GenerateLoginTokenAndHash {
    let token = Alphanumeric.sample_string(&mut rand::rng(), 96);
    let hash = hash_key(&token);
    GenerateLoginTokenAndHash { token, hash }
}

#[derive(Debug)]
pub struct AuthedDashboardUser;

struct DashboardAuthMiddlewareImpl<E> { ep: E }

impl<E: Endpoint> Endpoint for DashboardAuthMiddlewareImpl<E> {
    type Output = E::Output;
    
    async fn call(&self, req: poem::Request) -> poem::Result<Self::Output> {
        macro_rules! login_redirect_err {
            () => {
                return Err(poem::Error::from_response(
                    Redirect::see_other("/admin/dashboard/login/").into_response(),
                ))
            };
        }

        let session = req.data::<Session>().unwrap();

        let Some(token_data) =
            session.get::<DashboardSessionLoginTokenData>(DASHBOARD_SESSION_TOKEN_DATA_KEY)
        else {
            login_redirect_err!();
        };

        let mut db = req.data::<sqlx::PgPool>().unwrap().acquire().await.unwrap();

        let Some(token_row) = get_dashboard_login_token(&mut db, &token_data.id)
            .await
            .unwrap()
        else {
            login_redirect_err!();
        };

        if !check_key_against_hash(&token_data.token, &token_row.token_hash) {
            login_redirect_err!();
        }

        self.ep.call(req).await
    }
}

impl<'a> poem::FromRequest<'a> for AuthedDashboardUser {
    async fn from_request(
        req: &'a poem::Request,
        _body: &mut poem::RequestBody,
    ) -> poem::Result<Self> {
        macro_rules! login_redirect_err {
            () => {
                return Err(poem::Error::from_response(
                    Redirect::see_other("/admin/dashboard/login/").into_response(),
                ))
            };
        }

        let session = req.data::<&Session>().unwrap();

        let Some(token_data) =
            session.get::<DashboardSessionLoginTokenData>(DASHBOARD_SESSION_TOKEN_DATA_KEY)
        else {
            login_redirect_err!();
        };

        let mut db = req.data::<sqlx::PgPool>().unwrap().acquire().await.unwrap();

        let Some(token_row) = get_dashboard_login_token(&mut db, &token_data.id)
            .await
            .unwrap()
        else {
            login_redirect_err!();
        };

        if !check_key_against_hash(&token_data.token, &token_row.token_hash) {
            login_redirect_err!();
        }

        Ok(AuthedDashboardUser)
    }
}
