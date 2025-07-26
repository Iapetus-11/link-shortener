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
    println!("here");
    let mut db = db.acquire().await.unwrap();

    let Some(link) = get_link(&mut db, &slug).await.unwrap() else {
        return Err(poem::Error::from_status(StatusCode::NOT_FOUND));
    };

    let remote_ip = remote_ip.map(|a| a.to_string());

    let mut header_hashmap = HashMap::<String, Vec<String>>::with_capacity(headers.len());
    for (header_name, header_value) in headers {
        if let Ok(header_value) = header_value.to_str().map(|hv| hv.to_string()) {
            let header_name = header_name.as_str().to_string();

            if let Some(header_values) = header_hashmap.get_mut(&header_name) {
                header_values.push(header_value.to_string());
            } else {
                header_hashmap.insert(header_name, vec![header_value]);
            }
        }
    }

    headers.into_iter().fold(
        HashMap::<String, Vec<String>>::new(),
        |mut map, (header_name, header_value)| {
            if let Ok(header_value) = header_value.to_str().map(|hv| hv.to_string()) {
                let header_name = header_name.as_str().to_string();
                map.entry(header_name).or_default().push(header_value);
            }

            map
        },
    );

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
