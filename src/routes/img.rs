use crate::models::models;
use rocket::fs::relative;
use rocket::fs::NamedFile;
use rocket::get;
use std::path::{Path, PathBuf};

#[get("/img/<path>")]
pub async fn get(path: PathBuf, _session: models::Session) -> Option<NamedFile> {
    let path = Path::new(relative!("img")).join(path);
    NamedFile::open(path).await.ok()
}
