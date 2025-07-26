use poem::Route;

mod admin;
mod redirect;

pub fn routes() -> Route {
    Route::new()
        .nest("/admin/", admin::routes())
        .at("/:slug/", redirect::redirect)
}
