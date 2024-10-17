use argon2::password_hash::Error;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono;
use lazy_static::lazy_static;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::content;
use rocket_db_pools::{sqlx, sqlx::Row, Connection, Database};
use std::collections::HashMap;
use tera::Tera;
use uuid::Uuid;

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("jv_db")]
struct JvDbConn(sqlx::SqlitePool);

static SESSION_LENGTH: i64 = 60 * 60 * 24 * 7; // 1 week

lazy_static! {
    static ref COLORS: HashMap<&'static str, &'static str> = [
        ("black", "#252422"),
        ("brown", "#ff9b42"),
        ("vanilla", "#e3e7af"),
        ("blue", "#6a8eae"),
        ("sugar", "#be6e46"),
    ]
    .iter()
    .copied()
    .collect();
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

async fn insert_user(
    mut conn: Connection<JvDbConn>,
    email: &str,
    password: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL,
            password TEXT NOT NULL,
            salt TEXT NOT NULL
        )
        "#,
    )
    .execute(&mut **conn)
    .await;
    let salted_password = hash_and_salt_password(password)?;
    sqlx::query(
        r#"
        INSERT INTO users (email, password, salt) VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(email)
    .bind(salted_password.password_hash)
    .bind(salted_password.salt.to_string())
    .execute(&mut **conn)
    .await;
    println!("Inserted user");
    Ok(())
}

#[get("/")]
async fn index(session: Session) -> content::RawHtml<String> {
    println!("Session: {:?}", session);
    let index = TEMPLATES
        .render("index.html", &tera::Context::new())
        .unwrap();
    content::RawHtml(index)
}

#[get("/signup")]
async fn get_signup() -> content::RawHtml<String> {
    let signup = TEMPLATES
        .render("signup.html", &tera::Context::new())
        .unwrap();

    content::RawHtml(signup)
}

#[derive(FromForm, Clone)]
struct UserSignup {
    email: String,
    password: String,
}

#[derive(FromForm)]
struct UserLogin<'f> {
    email: &'f str,
    password: &'f str,
}

#[derive(FromForm)]
struct ImgUpload<'f> {
    file: TempFile<'f>,
    caption: &'f str,
    modified_file: TempFile<'f>,
}

struct SaltedPassword {
    password_hash: String,
    salt: SaltString,
}

fn hash_and_salt_password(user_password: &str) -> Result<SaltedPassword, Error> {
    let password = user_password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password, &salt)?.to_string();

    let parsed_hash = PasswordHash::new(&password_hash)?;
    assert!(Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok());

    Ok(SaltedPassword {
        password_hash,
        salt,
    })
}

async fn create_user_session(db: &JvDbConn, email: &str) -> Result<String, Error> {
    println!("Creating session for user: {}", email);

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
    .await;

    let session_token = Uuid::new_v4();

    println!("Session token: {}", session_token);
    println!("Email: {}", email);

    let user_id: i64 = sqlx::query(
        r#"
        SELECT id FROM users WHERE email = ?1
        "#,
    )
    .bind(email)
    .fetch_one(&db.0)
    .await
    .unwrap()
    .get(0);

    println!("User id: {}", user_id);

    let expires_at = chrono::Utc::now().timestamp() + SESSION_LENGTH;

    sqlx::query(
        r#"
        INSERT INTO sessions (session_token, user_id, expires_at) VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(session_token.to_string())
    .bind(user_id)
    .bind(expires_at)
    .execute(&db.0)
    .await;

    Ok(session_token.to_string())
}

#[post("/signup", data = "<user_signup>")]
async fn post_signup(
    user_signup: Form<UserSignup>,
    conn: Connection<JvDbConn>,
) -> content::RawHtml<String> {
    insert_user(conn, &user_signup.email, &user_signup.password).await;
    let login = TEMPLATES
        .render("login.html", &tera::Context::new())
        .unwrap();
    content::RawHtml(login)
}

async fn verify_password(db: &JvDbConn, email: &str, password: &str) -> Result<bool, Error> {
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

    let salt = SaltString::from_b64(db_salt.as_str())?;

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash == db_password_hash)
}

#[get("/login")]
async fn get_login() -> content::RawHtml<String> {
    let login = TEMPLATES
        .render("login.html", &tera::Context::new())
        .unwrap();

    content::RawHtml(login)
}

#[derive(Debug)]
struct Session {
    session_token: String,
    user_id: i64,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Session, ()> {
        let session_token: String = request
            .cookies()
            .get_private("s_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .unwrap();

        let pool = request.rocket().state::<JvDbConn>().unwrap();

        let row = sqlx::query(
            r#"
            SELECT user_id FROM sessions WHERE session_token = ?1
            "#,
        )
        .bind(&session_token)
        .fetch_one(&pool.0)
        .await
        .unwrap();

        let user_id: i64 = row.get(0);

        let session = Session {
            session_token,
            user_id,
        };

        println!("Session: {:?}", session);
        Outcome::Success(session)
    }
}

#[post("/login", data = "<user_login>")]
async fn post_login(
    user_login: Form<UserLogin<'_>>,
    cookies: &CookieJar<'_>,
    jv_db: &JvDbConn,
) -> content::RawHtml<String> {
    println!("Email: {}", user_login.email);

    let is_valid = verify_password(jv_db, user_login.email, user_login.password)
        .await
        .unwrap();

    println!("Is valid: {}", is_valid);

    if is_valid {
        let session_token = create_user_session(jv_db, user_login.email).await.unwrap();

        println!("Session token: {}", session_token);
        cookies.add_private(Cookie::new("s_id", session_token));
    } else {
        // raise an error
        println!("Invalid login");
    }

    let index = TEMPLATES
        .render("index.html", &tera::Context::new())
        .unwrap();

    content::RawHtml(index)
}

#[post("/upload", data = "<img_upload>")]
async fn img_upload(mut img_upload: Form<ImgUpload<'_>>, jv_db: &JvDbConn) -> std::io::Result<()> {
    img_upload.file.persist_to("/tmp/test.jpg").await?;
    img_upload
        .modified_file
        .copy_to("./static/test_modified.jpg")
        .await?;
    Ok(())
}

#[get("/css/style.css")]
async fn get_css() -> content::RawCss<String> {
    let mut context = tera::Context::new();
    for (name, color) in COLORS.iter() {
        context.insert(*name, color);
    }
    let style = TEMPLATES.render("css/style.css", &context).unwrap();
    content::RawCss(style)
}

#[get("/js/upload.js")]
async fn get_js() -> content::RawJavaScript<String> {
    let context = tera::Context::new();
    let js = TEMPLATES.render("js/upload.js", &context).unwrap();
    content::RawJavaScript(js)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                get_signup,
                post_signup,
                get_login,
                post_login,
                get_css,
                img_upload,
                get_js
            ],
        )
        .attach(JvDbConn::init())
        .mount("/", FileServer::from(relative!("static")))
}
