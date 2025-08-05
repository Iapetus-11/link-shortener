mod home;
mod login;

pub fn routes() -> poem::Route {
    poem::Route::new()
        .nest("", home::routes())
        .nest("/login/", login::routes())
}
