use crate::db::queries::Db;
use crate::models;
use rocket::get;
use rocket::response::Redirect;

#[get("/")]
pub async fn get(
    _db: &Db,
    _session: models::Session,
) -> Redirect {
    Redirect::to(rocket::uri!("/galleries"))
}
