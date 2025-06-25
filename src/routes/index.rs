use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models;
use crate::tera_utils;
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
    context.insert("gallery_id", &1);
    let index = tera_utils::render_template_with_logging("index.html", &context)?;
    Ok(content::RawHtml(index))
}
