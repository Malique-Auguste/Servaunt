use crate::renderer::{render, render_myfiles};
use crate::user_manager::{UserManager, LoginData};

use std::fs;
use std::sync::{Arc, Mutex};

use rocket::form::{Form, FromForm};
use rocket::fs::{TempFile, NamedFile};
use rocket::{response, State};
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Redirect, content::RawHtml};




#[get("/")]
pub async fn index() -> NamedFile {
    let response = NamedFile::open("website/html/index.html").await.ok();

    response.unwrap()
}

#[get("/")]
pub async fn signup() -> RawHtml<String> {
    let response = render("website/html/signup.html", None, None, None).unwrap();
    RawHtml(response)
}

#[post("/", data = "<user_data>")]
pub async fn signup_data(user_data: Form<LoginData>, dm: &State<Arc<Mutex<UserManager>>>) -> Result<Redirect, RawHtml<String>> {
    let mut dm_unwrapped = dm.lock().unwrap();
    match (*dm_unwrapped).add_new_user(user_data.to_user()) {
        Ok(_) => Ok(Redirect::to(uri!("/index.html"))),
        Err(e) => {
            let response = render("website/html/signup.html", None, Some(&e), None).unwrap();
            Err(RawHtml(response))
        }
    }
}

#[get("/")]
pub async fn login() -> RawHtml<String> {
    let response = render("website/html/login.html", None, None, None).unwrap();
    RawHtml(response)
}

#[post("/", data = "<user_data>")]
pub async fn login_data(user_data: Form<LoginData>, dm: &State<Arc<Mutex<UserManager>>>, jar: &CookieJar<'_>) -> Result<Redirect, RawHtml<String>> {
    let mut dm_unwrapped = dm.lock().unwrap();
    let current_user = user_data.to_user();

    match (*dm_unwrapped).login_user(&current_user, jar) {
        Ok(_) => Ok(Redirect::to(uri!("/myfiles.html"))),
        Err(e) => Err(RawHtml(render("website/html/login.html", None, Some(&e), None).unwrap()))
    }
}

#[get("/")]
pub async fn my_files(dm: &State<Arc<Mutex<UserManager>>>, jar: &CookieJar<'_>) -> RawHtml<String> {
    let dm_unwrapped = dm.lock().unwrap();
    let current_user = dm_unwrapped.get_current_user(jar).unwrap();
    println!("Logged in user: {:?}", current_user);

    let response = render_myfiles("website/html/myfiles.html", current_user).unwrap();
    RawHtml(response)
}

#[post("/",  data = "<form>")]
pub async fn upload_data(mut form: Form<Upload<'_>>, dm: &State<Arc<Mutex<UserManager>>>, jar: &CookieJar<'_>) -> RawHtml<String> {
    
    let current_user = {
        //I cannot write to the file (later in the code) while having dm_unwrapped accessible due to asyn rules?? (idk the real reason)
        //Therefore I put it in a block so that it is immediately dropped
        //This is also why i dereferenced the user and then cloned it.

        let dm_unwrapped = dm.lock().unwrap();
        (*dm_unwrapped.get_current_user(jar).unwrap()).clone()
    };
    
    let file_directory = format!("database/{}", current_user.get_name());

    let file_name = form.file.raw_name().unwrap().dangerous_unsafe_unsanitized_raw().as_str();
    //let success_message = format!("File named '{}' was uploaded successfully", file_name);
    
    let file_destination = format!("{}/{}", file_directory, file_name);

    fs::create_dir_all(file_directory).unwrap();
    form.file.persist_to(file_destination).await.unwrap();
    
    let response = render_myfiles("website/html/myfiles.html", &current_user).unwrap();
    RawHtml(response)
}


#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>
}