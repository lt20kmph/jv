use rocket::fs::{relative, FileServer};
use rocket::response::content;
use std::fs;

#[macro_use]
extern crate rocket;
use rusqlite::{Connection, Result};

fn open_my_db() -> Result<()> {
    let path = "./jv_db.db3";
    let db = Connection::open(path)?;
    // Use the database somehow...
    println!("{}", db.is_autocommit());
    Ok(())
}

#[get("/")]
fn index() -> content::RawHtml<String> {
    let contents =
        fs::read_to_string("html/index.html").expect("Should have been able to read the file");

    content::RawHtml(contents)
}

#[launch]
fn rocket() -> _ {
    open_my_db().expect("Failed to open the database");
    rocket::build()
        .mount("/", routes![index])
        .mount("/", FileServer::from(relative!("static")))
}
