use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models;
use rocket::get;
use rocket::response::content;

#[get("/")]
pub async fn get(
    db: &Db,
    _session: models::Session,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let mut context = tera::Context::new();
    let images = queries::get_gallery_images(db, 1).await;
    context.insert("images", &images);
    let index = constants::TEMPLATES.render("index.html", &context)?;
    Ok(content::RawHtml(index))
}
