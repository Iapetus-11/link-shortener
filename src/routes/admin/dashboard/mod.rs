use poem::get;

mod home;
mod login;

pub fn routes() -> poem::Route {
    poem::Route::new()
        .nest("", home::routes())
        .at("/login/", login::routes())
}
