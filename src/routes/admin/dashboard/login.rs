use askama::Template;
use poem::{
    get, post,
    web::{Data, Html},
};

pub fn routes() -> poem::Route {
    poem::Route::new()
        .at("", get(get_login).post(post_login))
}

#[derive(askama::Template)]
#[template(path = "views/admin/dashboard/login.html")]
struct LoginViewTemplate {}

#[poem::handler]
pub async fn get_login() -> Html<String> {
    Html(LoginViewTemplate {}.render().unwrap())
}

#[poem::handler]
pub async fn post_login(Data(db_pool): Data<&sqlx::PgPool>) {
    todo!();
}
