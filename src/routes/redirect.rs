use std::collections::HashMap;

use poem::{
    http::{HeaderMap, StatusCode},
    web::{Data, Path, RealIp, Redirect},
};

use crate::db::{link_visits::create_link_visit, links::get_link};

#[poem::handler]
pub async fn redirect(
    db: Data<&sqlx::PgPool>,
    Path((slug,)): Path<(String,)>,
    RealIp(remote_ip): RealIp,
    headers: &HeaderMap,
) -> poem::Result<Redirect> {
    let mut db = db.acquire().await.unwrap();

    let Some(link) = get_link(&mut db, &slug).await.unwrap() else {
        return Err(poem::Error::from_status(StatusCode::NOT_FOUND));
    };

    let remote_ip = remote_ip.map(|a| a.to_string());

    let mut header_hashmap = HashMap::<String, Vec<String>>::with_capacity(headers.keys_len());
    for (header_name, header_value) in headers {
        if let Ok(header_value) = header_value.to_str().map(|hv| hv.to_string()) {
            let header_name = header_name.as_str().to_string();
            header_hashmap
                .entry(header_name.to_lowercase())
                .or_default()
                .push(header_value);
        }
    }

    create_link_visit(
        &mut db,
        &slug,
        header_hashmap,
        match remote_ip {
            Some(ref ip) => Some(ip.as_str()),
            None => None,
        },
    )
    .await
    .unwrap();

    Ok(Redirect::temporary(link.url))
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        common::testing::app::api_test_client,
        db::{links::create_link, platforms::create_platform},
    };

    use super::*;

    #[sqlx::test]
    async fn test_redirect_but_link_not_found(db_pool: PgPool) {
        let mut db = db_pool.acquire().await.unwrap();

        let (_, platform) = create_platform(&mut db, "sad").await.unwrap();
        create_link(
            &mut db,
            &platform.id,
            None,
            "https://example.com/".to_string(),
            None,
        )
        .await
        .unwrap();

        let api = api_test_client(db_pool);
        let response = api.get("/notit/").send().await;

        response.assert_status(StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn test_redirect_success(db_pool: PgPool) {
        let mut db = db_pool.acquire().await.unwrap();

        let (_, platform) = create_platform(&mut db, "sad").await.unwrap();
        let link = create_link(
            &mut db,
            &platform.id,
            None,
            "https://example.com/".to_string(),
            None,
        )
        .await
        .unwrap();

        let api = api_test_client(db_pool);
        let response = api
            .get(format!("/{}/", link.slug))
            .header("X-Test-Header", "here is a test value")
            .send()
            .await;

        response.assert_status(StatusCode::TEMPORARY_REDIRECT);
        response.assert_header("Location", link.url);

        let link_visit = sqlx::query!("SELECT * from link_visits;")
            .fetch_one(&mut *db)
            .await
            .unwrap();
        assert_eq!(link_visit.link_slug, link.slug);

        let stored_headers = link_visit
            .headers
            .as_object()
            .unwrap()
            .get_key_value("x-test-header")
            .unwrap();
        assert_eq!(
            stored_headers.1.as_array().unwrap()[0].as_str().unwrap(),
            "here is a test value"
        );
    }
}
