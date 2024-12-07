use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::content;
use rocket::{get, post};

#[post("/galleries", data = "<create_gallery>")]
pub async fn post(
    create_gallery: Form<models::CreateGallery<'_>>,
    session: models::Session,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let gallery_name = create_gallery.name;

    let gallery_name = match gallery_name {
        Some(name) => name,
        None => "Untitled",
    };

    let gallery_id = queries::create_gallery(db, session.user_id, gallery_name).await?;

    // Redirect to gallery page
    Ok(content::RawHtml(format!(
        "Gallery created with id: {}",
        gallery_id
    )))
}

#[post("/galleries/<gallery_id>", data = "<img_upload>")]
pub async fn post_img(
    mut img_upload: Form<models::ImgUpload<'_>>,
    session: models::Session,
    gallery_id: i64,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let original_path = img_upload.file.name();

    let original_path = match original_path {
        Some(path) => path,
        None => {
            return Err(errors::AppError {
                message: "No file uploaded".to_string(),
                code: Status::BadRequest.code,
            })
        }
    };

    let img_path = queries::create_image(
        db,
        session.user_id,
        gallery_id,
        original_path,
        img_upload.caption,
    )
    .await?;

    // Save the file (persist to should be more performant... but this should be good enough)
    // TODO: These can be done in parallel
    img_upload.file.copy_to(&img_path.original_path).await?;
    img_upload.modified_file.copy_to(&img_path.path).await?;

    let mut context = tera::Context::new();
    context.insert("path", &img_path.path);
    context.insert("caption", &img_upload.caption);
    let image_item = constants::TEMPLATES.render("image_item.html", &context)?;

    Ok(content::RawHtml(image_item))
}

#[get("/galleries")]
pub async fn get(db: &Db) -> Result<content::RawHtml<String>, errors::AppError> {
    let galleries = queries::get_galleries(db).await?;
    let mut context = tera::Context::new();
    context.insert("galleries", &galleries);
    let galleries_html = constants::TEMPLATES.render("galleries.html", &context)?;
    Ok(content::RawHtml(galleries_html))
}
