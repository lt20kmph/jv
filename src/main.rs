use argon2::password_hash::Error;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono;
use lazy_static::lazy_static;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::content;
use rusqlite;
use tera::Tera;
use uuid::Uuid;

#[macro_use]
extern crate rocket;

static SESSION_LENGTH: i64 = 60 * 60 * 24 * 7; // 1 week
static DB_PATH: &str = "./jv_db.db3";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
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

//Test function...
// TODO: replace database with rocket state (db)
fn open_my_db() -> rusqlite::Result<()> {
    let path = DB_PATH;
    let db = rusqlite::Connection::open(path)?;
    // Use the database somehow...
    println!("{}", db.is_autocommit());
    Ok(())
}

fn insert_user(email: &str, password: &str) -> rusqlite::Result<()> {
    let conn = rusqlite::Connection::open(DB_PATH)?;
    // create table if not exists with email, hash_password, salt
    // TODO: Add a unique constraint on email
    // TODO: Make schema definition happen in a seperate file.
    // TODO: Add username
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL,
            password TEXT NOT NULL,
            salt TEXT NOT NULL
        )",
        rusqlite::params![],
    )?;
    let salted_password = hash_and_salt_password(password).unwrap();
    conn.execute(
        "INSERT INTO users (email, password, salt) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            email,
            salted_password.password_hash,
            salted_password.salt.to_string()
        ],
    )?;
    println!("Inserted user");
    println!("{}", conn.is_autocommit());
    Ok(())
}

#[get("/")]
fn index(session: Session) -> content::RawHtml<String> {
    println!("Session: {:?}", session);
    let index = TEMPLATES
        .render("index.html", &tera::Context::new())
        .unwrap();
    content::RawHtml(index)
}

#[get("/signup")]
fn get_signup() -> content::RawHtml<String> {
    let signup = TEMPLATES
        .render("signup.html", &tera::Context::new())
        .unwrap();

    content::RawHtml(signup)
}

#[derive(FromForm)]
struct UserSignup<'f> {
    email: &'f str,
    password: &'f str,
}

#[derive(FromForm)]
struct UserLogin<'f> {
    email: &'f str,
    password: &'f str,
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

fn create_user_session(email: &str) -> rusqlite::Result<String, rusqlite::Error> {
    println!("Creating session for user: {}", email);
    let db = rusqlite::Connection::open(DB_PATH).unwrap();
    db.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            session_token TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            expires_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        rusqlite::params![],
    )?;

    let session_token = Uuid::new_v4().to_string();

    println!("Session token: {}", session_token);
    println!("Email: {}", email);

    let user_id: i64 = db
        .query_row(
            "SELECT id FROM users WHERE email = ?1",
            rusqlite::params![email],
            |row| row.get(0),
        )
        .unwrap();

    println!("User id: {}", user_id);

    let expires_at = chrono::Utc::now().timestamp() + SESSION_LENGTH;

    let _ = db.execute(
        "INSERT INTO sessions (session_token, user_id, expires_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![session_token, user_id, expires_at],
    )?;

    println!("{}", db.is_autocommit());

    Ok(session_token)
}

#[post("/signup", data = "<user_signup>")]
fn post_signup(user_signup: Form<UserSignup<'_>>) -> content::RawHtml<String> {
    println!("Email: {}", user_signup.email);
    println!("Pw: {}", user_signup.password);

    let _ = insert_user(user_signup.email, user_signup.password);

    let login = TEMPLATES
        .render("login.html", &tera::Context::new())
        .unwrap();

    content::RawHtml(login)
}

fn verify_password(email: &str, password: &str) -> Result<bool, Error> {
    let conn = rusqlite::Connection::open(DB_PATH).unwrap();
    let mut stmt = conn
        .prepare("SELECT password, salt FROM users WHERE email = ?1")
        .unwrap();
    let mut rows = stmt.query(rusqlite::params![email]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let db_password_hash: String = row.get(0).unwrap();
    let db_salt: String = row.get(1).unwrap();

    let salt = SaltString::from_b64(db_salt.as_str()).unwrap();

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash == db_password_hash)
}

#[get("/login")]
fn get_login() -> content::RawHtml<String> {
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
        let db = rusqlite::Connection::open(DB_PATH).unwrap();
        let session_token = request
            .cookies()
            .get_private("s_id")
            .and_then(|cookie| cookie.value().parse().ok());

        println!("Session token: {:?}", session_token);

        let user_id = db
            .query_row(
                "SELECT user_id FROM sessions WHERE session_token = ?1",
                rusqlite::params![session_token],
                |row| row.get(0),
            )
            .unwrap();

        let session = Session {
            session_token: session_token.unwrap(),
            user_id,
        };

        println!("Session: {:?}", session);
        Outcome::Success(session)
    }
}

#[post("/login", data = "<user_login>")]
fn post_login(
    user_login: Form<UserLogin<'_>>,
    cookies: &CookieJar<'_>,
) -> content::RawHtml<String> {
    println!("Email: {}", user_login.email);

    let is_valid = verify_password(user_login.email, user_login.password).unwrap();

    println!("Is valid: {}", is_valid);

    if is_valid {
        let session_token = create_user_session(user_login.email).unwrap();
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

#[launch]
fn rocket() -> _ {
    open_my_db().expect("Failed to open the database");

    rocket::build()
        .mount(
            "/",
            routes![index, get_signup, post_signup, get_login, post_login],
        )
        .mount("/", FileServer::from(relative!("static")))
}
