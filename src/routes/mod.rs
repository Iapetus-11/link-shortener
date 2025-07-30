use poem::Route;

mod admin;
mod redirect;
mod static_files;

pub fn routes() -> Route {
    Route::new()
        .nest("/admin/", admin::routes())
        .nest("/static/", static_files::routes())
        .at("/:slug/", redirect::redirect)
}
