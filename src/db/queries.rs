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
            is_verified BOOLEAN DEFAULT FALSE,
            verification_uuid TEXT NOT NULL,
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
            status TEXT DEFAULT 'public',
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
            status TEXT DEFAULT 'public',
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
) -> Result<String, errors::AppError> {
    let salted_password = pw_utils::hash_and_salt_password(password);

    let salted_password = match salted_password {
        Ok(salted_password) => salted_password,
        Err(e) => {
            return Err(errors::AppError::from(e));
        }
    };

    let verification_uuid = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO users (email, password, salt, verification_uuid) VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(email)
    .bind(salted_password.password_hash)
    .bind(salted_password.salt.to_string())
    .bind(&verification_uuid)
    .execute(&mut **conn)
    .await?;

    Ok(verification_uuid)
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

pub async fn get_user_from_session_token(
    session_token: &str,
    pool: &Db,
) -> Result<models::User, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            users.id, 
            users.email,
            users.role
        FROM sessions
        INNER JOIN users ON sessions.user_id = users.id
        WHERE session_token = ?1
        "#,
    )
    .bind(&session_token)
    .fetch_one(&pool.0)
    .await?;

    info!("Got user from session token");

    let user_id: i64 = row.get(0);
    let email: String = row.get(1);
    let role: models::Role = match row.get(2) {
        "reader" => models::Role::Reader,
        "writer" => models::Role::Writer,
        _ => models::Role::Reader,
    };

    sqlx::Result::Ok(models::User {
        id: user_id,
        email,
        role,
    })
}

pub async fn create_gallery(db: &Db, user_id: i64, name: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO galleries (user_id, name, gallery_text, status) VALUES (?1, ?2, '', 'public') RETURNING id
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
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO modified_images (
            user_id,
            original_image_id,
            path,
            caption,
            status
        ) VALUES (?1, ?2, ?3, ?4, ?5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(original_image_id)
    .bind(&img_path.path)
    .bind(caption)
    .bind("public")
    .execute(&db.0)
    .await?;

    let modified_image_id = row.last_insert_rowid();

    Ok(modified_image_id)
}

// Create new image, return image_path
pub async fn create_image(
    db: &Db,
    user_id: i64,
    gallery_id: i64,
    original_filename: &str,
    caption: &str,
) -> Result<models::Image, sqlx::Error> {
    let img_path = models::ImgPath::new();

    let original_image_id =
        insert_original_image(db, user_id, gallery_id, &img_path, original_filename).await?;

    let modified_image_id =
        insert_modified_image(db, user_id, original_image_id, &img_path, caption).await?;

    Ok(models::Image {
        id: modified_image_id,
        path: img_path.path,
        original_path: Some(img_path.original_path),
        caption: caption.to_string(),
    })
}

pub async fn get_gallery_images(db: &Db, gallery_id: i64) -> Vec<models::Image> {
    let mut images = vec![];

    let mut rows = sqlx::query(
        r#"
        SELECT
            modified_images.id,
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
        let id: i64 = row.get(0);
        let path: String = row.get(1);
        let caption: String = row.get(2);
        images.push(models::Image { id, path, original_path: None, caption });
    }

    images
}

pub async fn verify_user(db: &Db, verification_uuid: &str) -> Result<String, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE users SET is_verified = TRUE WHERE verification_uuid = ?1 RETURNING email
        "#,
    )
    .bind(verification_uuid)
    .fetch_one(&db.0)
    .await?;

    let email: String = row.get(0);

    Ok(email)
}

pub async fn get_galleries(db: &Db) -> Result<Vec<models::GalleryTile>, sqlx::Error> {
    let mut galleries = vec![];

    let mut rows = sqlx::query(
        r#"
        SELECT 
          galleries.id AS id,
          galleries.name AS name,
          galleries.time_created AS time_created,
          count(*) AS n_images,
          max(modified_images.path) AS last_image,
          users.email AS created_by
        FROM galleries 
        LEFT JOIN original_images ON galleries.id = original_images.gallery_id
        LEFT JOIN modified_images ON original_images.id = modified_images.original_image_id
        LEFT JOIN users ON galleries.user_id = users.id
        WHERE galleries.status != 'deleted'
        GROUP BY galleries.id, galleries.name, galleries.time_created
        ORDER BY galleries.time_created DESC
        "#,
    )
    .fetch(&db.0);

    while let Ok(row) = rows.try_next().await {
        let row = match row {
            Some(row) => row,
            None => break,
        };
        let id: i64 = row.get(0);
        let name: String = row.get(1);
        let time_created: String = row.get(2);
        let mut n_images: i64 = row.get(3);
        let last_image: Option<String> = row.get(4);
        let created_by: String = row.get(5);
        if last_image.is_none() {
            n_images = 0;
        }
        galleries.push(models::GalleryTile::new(
            id,
            name,
            last_image,
            n_images,
            time_created,
            created_by,
        ));
    }

    Ok(galleries)
}

pub async fn get_gallery(
    db: &Db,
    gallery_id: i64,
) -> Result<models::GalleryContents, errors::AppError> {
    let mut rows = sqlx::query(
        r#"
        SELECT 
            name,
            galleries.time_created,
            gallery_text,
            modified_images.id AS image_id,
            modified_images.path AS path,
            modified_images.caption AS caption
        FROM galleries 
        LEFT JOIN original_images ON galleries.id = original_images.gallery_id
        LEFT JOIN modified_images ON original_images.id = modified_images.original_image_id
        WHERE galleries.id = ?1 
        AND modified_images.status = 'public'
        "#,
    )
    .bind(gallery_id)
    .fetch(&db.0);

    let mut images = vec![];
    let mut name = None;
    let mut time_created = None;

    while let Ok(row) = rows.try_next().await {
        let row = match row {
            Some(row) => row,
            None => break,
        };
        if name.is_none() {
            info!("Getting name");
            name = Some(row.get(0));
            info!("Name: {}", name.clone().unwrap());
        }
        if time_created.is_none() {
            info!("Getting time created");
            time_created = Some(row.get(1));
            info!("Time created: {}", time_created.clone().unwrap());
        }
        let image_id: i64 = row.get(3);
        let path: String = row.get(4);
        let caption: String = row.get(5);
        if path.is_empty() {
            continue;
        }
        images.push(models::Image {
            id: image_id,
            path,
            original_path: None,
            caption,
        });
    }

    Ok(models::GalleryContents {
        id: gallery_id,
        name: name.ok_or("Couldn't get gallery name".to_string())?,
        images,
        time_created: time_created.ok_or("Couldn't get gallery time created".to_string())?,
    })
}

pub async fn delete_gallery(db: &Db, gallery_id: i64) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE galleries SET status = 'deleted' WHERE id = ?1
        "#,
    )
    .bind(gallery_id)
    .execute(&db.0)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn update_gallery(
    db: &Db,
    gallery_id: i64,
    update: models::GalleryUpdate<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE galleries SET name = ?1 WHERE id = ?2
        "#,
    )
    .bind(update.name)
    .bind(gallery_id)
    .execute(&db.0)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn delete_image(db: &Db, image_id: i64) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE modified_images SET status = 'deleted' WHERE id = ?1
        "#,
    )
    .bind(image_id)
    .execute(&db.0)
    .await?;

    Ok(())
}
