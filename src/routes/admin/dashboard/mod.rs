use std::time::Duration;

use poem::{
    EndpointExt, Response,
    endpoint::DynEndpoint,
    session::{CookieConfig, CookieSession},
    web::cookie::SameSite,
};

use crate::config::CONFIG;

mod home;
mod login;

pub fn routes() -> Box<dyn DynEndpoint<Output = Response>> {
    poem::Route::new()
        .nest("", home::routes())
        .nest("/login/", login::routes())
        .with(CookieSession::new(
            CookieConfig::new()
                .max_age(Some(Duration::from_secs(
                    CONFIG.admin_login_expires_after_seconds,
                )))
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/admin/dashboard/"),
        ))
        .boxed()
}
