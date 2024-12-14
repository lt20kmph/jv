use argon2::password_hash::SaltString;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;

#[derive(FromForm, Clone)]
pub struct UserSignup {
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct UserLogin<'f> {
    pub email: &'f str,
    pub password: &'f str,
}

#[derive(FromForm)]
pub struct ImgUpload<'f> {
    pub file: TempFile<'f>,
    pub modified_file: TempFile<'f>,
    pub caption: &'f str,
}

#[derive(FromForm)]
pub struct CreateGallery<'f> {
    pub name: Option<&'f str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub enum Role {
    Reader,
    Writer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i64,
    pub email: String,
    pub role: Role,
}

#[derive(Debug)]
pub struct Session {
    pub session_token: String,
    pub user: User,
}

pub struct SaltedPassword {
    pub password_hash: String,
    pub salt: SaltString,
}

pub struct ImgPath {
    pub path: String,
    pub original_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Image {
    pub path: String,
    pub caption: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GalleryTile {
    pub id: i64,
    pub name: String,
    pub example_image_path: String,
    pub image_count: i64,
    pub time_created: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GalleryContents {
    pub id: i64,
    pub name: String,
    pub images: Vec<Image>,
    pub time_created: String,
}
