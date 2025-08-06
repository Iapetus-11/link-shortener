use poem::{Endpoint, IntoResponse, session::Session, web::Redirect};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::error::Error as StdError;
use uuid::Uuid;

use crate::{
    common::argon2::{argon2_check_key_against_hash, argon2_hash_key, setup_weak_argon2},
    config::CONFIG,
    db::dashboard_login_token::{create_dashboard_login_token, get_dashboard_login_token},
};

pub const DASHBOARD_SESSION_TOKEN_DATA_KEY: &str = "__Host-DSTD";

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
    let argon2 = setup_weak_argon2();

    if !argon2_check_key_against_hash(&argon2, password, &CONFIG.admin_password_hash) {
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

    let argon2 = setup_weak_argon2();
    let hash = argon2_hash_key(&argon2, &token);

    GenerateLoginTokenAndHash { token, hash }
}

pub async fn dashboard_auth_middleware<E: Endpoint>(
    next: E,
    req: poem::Request,
) -> poem::Result<E::Output> {
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

    let argon2 = setup_weak_argon2();
    if !argon2_check_key_against_hash(&argon2, &token_data.token, &token_row.token_hash) {
        login_redirect_err!();
    }

    next.call(req).await
}
