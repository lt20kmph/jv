use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models;
use crate::tera_utils;
use rocket::get;
use rocket::response::Redirect;

#[get("/")]
pub async fn get(
    db: &Db,
    _session: models::Session,
) -> Redirect {
    Redirect::to(rocket::uri!("/galleries"))
}
