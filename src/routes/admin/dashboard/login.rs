use poem::{get, web::Data};

pub fn routes() -> poem::Route {
    poem::Route::new()
        .at("", get(get_login))
        .at("", )
}

#[poem::handler]
pub async fn get_login(Data(db_pool): Data<&sqlx::PgPool>) {
    todo!();
}