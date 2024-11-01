use crate::constants;
use crate::errors;
use rocket::get;
use rocket::response::content;

#[get("/js/<script_name>")]
pub async fn get(
    script_name: &str,
) -> Result<content::RawJavaScript<String>, errors::AppError> {
    let context = tera::Context::new();
    let path = format!("js/{}", script_name);
    let js = constants::TEMPLATES.render(&path, &context)?;
    Ok(content::RawJavaScript(js))
}
