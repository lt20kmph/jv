use crate::errors;
use crate::tera_utils;
use rocket::get;
use rocket::response::content;

#[get("/about")]
pub async fn get() -> Result<content::RawHtml<String>, errors::AppError> {
    let about = tera_utils::render_template_with_logging("about.html", &tera::Context::new())?;
    Ok(content::RawHtml(about))
}
