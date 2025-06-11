use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models;
use rocket::fs::{relative, NamedFile};
use rocket::response::content;
use rocket::{delete, get};
use std::path::{Path, PathBuf};

#[get("/img/<path>")]
pub async fn get(path: PathBuf, _session: models::Session) -> Option<NamedFile> {
    let path = Path::new(relative!("img")).join(path);
    NamedFile::open(path).await.ok()
}

#[delete("/img/<image_id>")]
pub async fn delete(
    db: &Db,
    _session: models::Session,
    image_id: i64,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let img_path = queries::delete_image(db, image_id).await?;
    Ok(content::RawHtml(format!("Image deleted: {:?}", img_path)))
}
