use rocket::catch;
use rocket::response::Redirect;

#[catch(401)]
pub fn not_authorized() -> Redirect {
    Redirect::to("/login")
}
