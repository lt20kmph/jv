use crate::constants;
use crate::errors;
use rocket::get;
use rocket::response::content;

#[get("/css/<style_name>")]
pub async fn get(style_name: &str) -> Result<content::RawCss<String>, errors::AppError> {
    let mut context = tera::Context::new();
    for (name, color) in constants::COLORS.iter() {
        context.insert(*name, color);
    }
    let path = format!("css/{}", style_name);
    let style = constants::TEMPLATES.render(&path, &context)?;
    Ok(content::RawCss(style))
}
