mod user_manager;
mod renderer;
mod routes;

use user_manager::UserManager;
use renderer::render;
use std::sync::{Arc, Mutex};

use rocket::fs::FileServer;

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    let mut dm: UserManager = UserManager::new().unwrap();
    println!("dm: {:?}", dm);
    let mut dm_wrapper = Arc::new(Mutex::new(dm));

    rocket::build()
        .mount("/", FileServer::from("website/other"))
        .mount("/", routes![routes::index])
        .mount("/index.html", routes![routes::index])
        .mount("/signup.html", routes![routes::signup, routes::signup_data])
        .mount("/login.html", routes![routes::login, routes::login_data])
        .mount("/myfiles.html", routes![routes::my_files, routes::upload_data, routes::open, routes::delete])
        .manage(dm_wrapper)
}

