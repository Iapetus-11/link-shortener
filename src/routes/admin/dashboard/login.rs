use askama::Template;
use poem::{
    Body, Response, get,
    session::Session,
    web::{Data, Form, Html, Redirect},
};
use serde::Deserialize;

use crate::common::dashboard_auth::attempt_log_in_dashboard_session;

pub fn routes() -> poem::Route {
    poem::Route::new().at("", get(get_login).post(post_login))
}

#[derive(askama::Template)]
#[template(path = "views/admin/dashboard/login.html")]
struct LoginViewTemplate {
    password_was_wrong: bool,
}

#[poem::handler]
pub async fn get_login() -> Html<String> {
    Html(
        LoginViewTemplate {
            password_was_wrong: false,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Deserialize)]
pub struct PostLoginRequest {
    password: String,
}

#[poem::handler]
pub async fn post_login(
    session: &Session,
    Data(db_pool): Data<&sqlx::PgPool>,
    Form(PostLoginRequest { password }): Form<PostLoginRequest>,
) -> poem::Result<Redirect> {
    let mut db = db_pool.acquire().await.unwrap();

    if !attempt_log_in_dashboard_session(&mut db, session, &password)
        .await
        .unwrap()
    {
        return Err(poem::Error::from_response(
            Response::builder()
                .content_type("text/html")
                .body(Body::from_string(
                    LoginViewTemplate {
                        password_was_wrong: true,
                    }
                    .render()
                    .unwrap(),
                )),
        ));
    }

    Ok(Redirect::see_other("/admin/dashboard/"))
}
