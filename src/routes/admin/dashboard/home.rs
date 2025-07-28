use askama::Template;
use poem::web::{Data, Html};

use crate::db::platforms::{Platform, get_platforms};

#[derive(askama::Template)]
#[template(path = "views/admin/home.html")]
struct HomeViewTemplate<'a> {
    platforms: &'a Vec<Platform>,
}

#[poem::handler]
pub async fn get_view(db_pool: Data<&sqlx::PgPool>) -> poem::Result<Html<String>> {
    let mut db = db_pool.acquire().await.unwrap();

    let platforms = get_platforms(&mut db).await.unwrap();

    Ok(Html(
        HomeViewTemplate {
            platforms: &platforms,
        }
        .render()
        .unwrap(),
    ))
}
