use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::middleware::WriterSession;
use crate::models;
use rocket::form::Form;
use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::{delete, get, put};
use std::path::{Path, PathBuf};

#[get("/img/<path>")]
pub async fn get(path: PathBuf, _session: models::Session) -> Option<NamedFile> {
    let path = Path::new(relative!("img")).join(path);
    NamedFile::open(path).await.ok()
}

#[delete("/img/<image_id>")]
pub async fn delete(
    db: &Db,
    _writer_session: WriterSession,
    image_id: i64,
) -> Result<Status, errors::AppError> {
    queries::delete_image(db, image_id).await?;
    Ok(Status::NoContent)
}

#[put("/img/<image_id>", data = "<caption_update>")]
pub async fn update_caption(
    caption_update: Form<models::CaptionUpdate<'_>>,
    db: &Db,
    _writer_session: WriterSession,
    image_id: i64,
) -> Result<Status, errors::AppError> {
    queries::update_image_caption(db, image_id, caption_update.caption).await?;
    Ok(Status::NoContent)
}
