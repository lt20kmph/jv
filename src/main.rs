mod auth {
    pub mod pw_utils;
}
mod catchers;
mod constants;
mod db {
    pub mod queries;
}
mod errors;
mod middleware;
mod models {
    pub mod models;
}
mod routes {
    pub mod css;
    pub mod galleries;
    pub mod img;
    pub mod index;
    pub mod js;
    pub mod login;
    pub mod signup;
}

use db::queries;
use db::queries::Db;
use env_logger;
use log::error;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{relative, FileServer};
use rocket::{launch, routes};
use rocket::{Build, Rocket, catchers};
use rocket_db_pools::Database;
use routes::css;
use routes::galleries;
use routes::img;
use routes::index;
use routes::js;
use routes::login;
use routes::signup;

async fn create_tables(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match queries::create_tables(&db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx create tables", create_tables))
            .mount(
                "/",
                routes![
                    galleries::post,
                    galleries::post_img,
                    signup::post,
                    login::post,
                    index::get,
                    signup::get,
                    signup::verify,
                    login::get,
                    css::get,
                    js::get,
                    img::get
                ],
            )
    })
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
        .attach(stage())
        .register(
            "/",
            catchers![catchers::not_authorized],
        )
        .mount("/", FileServer::from(relative!("static")))
}
