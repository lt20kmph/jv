use crate::auth::pw_utils;
use crate::errors;
use rocket::futures::TryStreamExt;

use crate::constants;
use crate::models::models;
use chrono;
use log::info;
use rocket_db_pools::Connection;
use rocket_db_pools::{sqlx, sqlx::Row, Database};
use uuid::Uuid;

#[derive(Database)]
#[database("db")]
pub struct Db(sqlx::SqlitePool);

async fn create_session_table(db: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            session_token TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            expires_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&db.0)
    .await?;
    info!("Sessions table created");
    Ok(())
}

async fn create_user_table(db: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL,
            role TEXT DEFAULT 'reader',
            password TEXT NOT NULL,
            time_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            salt TEXT NOT NULL
        )
        "#,
    )
    .execute(&db.0)
    .await?;
    info!("Users table created");
    Ok(())
}

async fn create_galleries_table(db: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS galleries (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            gallery_text TEXT NOT NULL,
            time_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&db.0)
    .await?;
    info!("Galleries table created");
    Ok(())
}

async fn create_modified_images_table(db: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS modified_images (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            original_image_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            caption TEXT NOT NULL,
            time_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            time_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (original_image_id) REFERENCES original_images(id)
        )
        "#,
    )
    .execute(&db.0)
    .await?;
    info!("Modified images table created");
    Ok(())
}

async fn create_original_images_table(db: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS original_images (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            gallery_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            path TEXT NOT NULL,
            time_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (gallery_id) REFERENCES galleries(id)
        )
        "#,
    )
    .execute(&db.0)
    .await?;
    info!("Original images table created");
    Ok(())
}

pub async fn create_tables(db: &Db) -> Result<(), errors::AppError> {
    info!("Creating tables");
    create_user_table(db).await?;
    create_session_table(db).await?;
    create_galleries_table(db).await?;
    create_original_images_table(db).await?;
    create_modified_images_table(db).await?;
    Ok(())
}

async fn get_user_id_by_email(db: &Db, email: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id FROM users WHERE email = ?1
        "#,
    )
    .bind(email)
    .fetch_one(&db.0)
    .await?;
    let user_id: i64 = row.get(0);
    Ok(user_id)
}

pub async fn create_user_session(db: &Db, email: &str) -> Result<String, errors::AppError> {
    let session_token = Uuid::new_v4();
    let user_id = get_user_id_by_email(db, email).await?;
    let expires_at = chrono::Utc::now().timestamp() + constants::SESSION_LENGTH;

    sqlx::query(
        r#"
        INSERT INTO sessions (session_token, user_id, expires_at) VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(session_token.to_string())
    .bind(user_id)
    .bind(expires_at)
    .execute(&db.0)
    .await?;

    Ok(session_token.to_string())
}

pub async fn insert_user(
    mut conn: Connection<Db>,
    email: &str,
    password: &str,
) -> Result<(), errors::AppError> {
    let salted_password = pw_utils::hash_and_salt_password(password);

    let salted_password = match salted_password {
        Ok(salted_password) => salted_password,
        Err(e) => {
            return Err(errors::AppError::from(e));
        }
    };

    sqlx::query(
        r#"
        INSERT INTO users (email, password, salt) VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(email)
    .bind(salted_password.password_hash)
    .bind(salted_password.salt.to_string())
    .execute(&mut **conn)
    .await?;

    Ok(())
}

pub async fn verify_password(
    db: &Db,
    email: &str,
    password: &str,
) -> Result<bool, errors::AppError> {
    let row = sqlx::query(
        r#"
        SELECT password, salt FROM users WHERE email = ?1
        "#,
    )
    .bind(email)
    .fetch_one(&db.0)
    .await;

    let row = match row {
        Ok(row) => row,
        Err(_) => return Ok(false),
    };

    let db_password_hash: String = row.get(0);
    let db_salt: String = row.get(1);

    Ok(pw_utils::verify_password(
        db_salt,
        &db_password_hash,
        password,
    )?)
}

pub async fn get_user_id_from_session_token(
    session_token: &str,
    pool: &Db,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT user_id FROM sessions WHERE session_token = ?1
        "#,
    )
    .bind(&session_token)
    .fetch_one(&pool.0)
    .await?;

    let user_id: i64 = row.get(0);
    sqlx::Result::Ok(user_id)
}

// Create new gallery, return gallery_id
pub async fn create_gallery(db: &Db, user_id: i64, name: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO galleries (user_id, name, gallery_text) VALUES (?1, ?2, '') RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .execute(&db.0)
    .await?;

    let gallery_id = row.last_insert_rowid();

    Ok(gallery_id)
}

impl models::ImgPath {
    pub fn new() -> models::ImgPath {
        let uuid0 = Uuid::new_v4();
        let uuid1 = Uuid::new_v4();
        models::ImgPath {
            path: format!("{}/{}", constants::IMG_PATH, uuid0),
            original_path: format!("{}/{}", constants::IMG_PATH, uuid1),
        }
    }
}

async fn insert_original_image(
    db: &Db,
    user_id: i64,
    gallery_id: i64,
    img_path: &models::ImgPath,
    filename: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO original_images (
            user_id,
            gallery_id,
            filename,
            path
        ) VALUES (?1, ?2, ?3, ?4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(gallery_id)
    .bind(filename)
    .bind(&img_path.path)
    .execute(&db.0)
    .await?;

    let original_image_id = row.last_insert_rowid();

    Ok(original_image_id)
}

async fn insert_modified_image(
    db: &Db,
    user_id: i64,
    original_image_id: i64,
    img_path: &models::ImgPath,
    caption: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO modified_images (
            user_id,
            original_image_id,
            path,
            caption
        ) VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(user_id)
    .bind(original_image_id)
    .bind(&img_path.path)
    .bind(caption)
    .execute(&db.0)
    .await?;

    Ok(())
}

// Create new image, return image_path
pub async fn create_image(
    db: &Db,
    user_id: i64,
    gallery_id: i64,
    original_filename: &str,
    caption: &str,
) -> Result<models::ImgPath, sqlx::Error> {
    let img_path = models::ImgPath::new();

    let original_image_id =
        insert_original_image(db, user_id, gallery_id, &img_path, original_filename).await?;

    insert_modified_image(db, user_id, original_image_id, &img_path, caption).await?;

    Ok(img_path)
}

pub async fn get_gallery_images(db: &Db, gallery_id: i64) -> Vec<models::Image> {
    let mut images = vec![];

    let mut rows = sqlx::query(
        r#"
        SELECT
            modified_images.path,
            modified_images.caption
        FROM original_images
        JOIN modified_images ON original_images.id = modified_images.original_image_id
        WHERE original_images.gallery_id = ?1
        "#,
    )
    .bind(gallery_id)
    .fetch(&db.0);

    while let Ok(row) = rows.try_next().await {
        let row = match row {
            Some(row) => row,
            None => break,
        };
        let path: String = row.get(0);
        let caption: String = row.get(1);
        images.push(models::Image { path, caption });
    }

    images
}
