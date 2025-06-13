use argon2::password_hash::SaltString;
use chrono::DateTime;
use chrono_humanize::{Accuracy, Tense};
use log::warn;
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
pub struct GalleryUpdate<'f> {
    pub name: &'f str,
}

#[derive(FromForm)]
pub struct CaptionUpdate<'f> {
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
    pub id: i64,
    pub path: String,
    pub original_path: Option<String>,
    pub caption: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GalleryTile {
    pub id: i64,
    pub name: String,
    pub example_image_path: Option<String>,
    pub image_count: i64,
    pub time_created: String,
    pub time_created_human: String,
    pub created_by: String,
}

impl GalleryTile {
    pub fn new(
        id: i64,
        name: String,
        example_image_path: Option<String>,
        image_count: i64,
        time_created: String,
        created_by: String,
    ) -> GalleryTile {
        let time_created_chrono =
            chrono::NaiveDateTime::parse_from_str(&time_created, "%Y-%m-%d %H:%M:%S");
        let human_time = match time_created_chrono {
            Ok(time) => chrono_humanize::HumanTime::from(
                DateTime::<chrono::Utc>::from_naive_utc_and_offset(time, chrono::Utc),
            ),
            Err(err) => {
                warn!(
                    "Error parsing time_created: {}, due to {}",
                    time_created, err
                );
                chrono_humanize::HumanTime::from(chrono::Utc::now())
            }
        };
        GalleryTile {
            id,
            name,
            example_image_path,
            image_count,
            time_created,
            time_created_human: human_time.to_text_en(Accuracy::Rough, Tense::Past),
            created_by,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GalleryContents {
    pub id: i64,
    pub name: String,
    pub images: Vec<Image>,
    pub time_created: String,
}
