use crate::user_manager::*;

use core::num;
use std::{fs, io::Read};
use rocket::fs::{TempFile, NamedFile};

pub fn render(html_file_path: &str, user: Option<&User>, error: Option<&String>, message: Option<&Vec<String>>) -> Result<String, String> {
    let mut html_file = match fs::OpenOptions::new().read(true).open(html_file_path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Error encountered while opening file: {}", e))
    };

    let mut html_file_content = String::new();

    match html_file.read_to_string(&mut html_file_content) {
        Ok(_) => (),
        Err(e) => return Err(format!("Error encountered while reading file: {}", e))
    };

    html_file_content = match user {
        Some(u) => html_file_content.replace("{name}", u.get_name()),
        None => html_file_content
    };

    html_file_content = match error {
        Some(e) => html_file_content.replace("{error}", e),
        None => html_file_content.replace("{error}", ""),
    };

    html_file_content = match message {
        Some(m) => {
            let num_of_messages = html_file_content.matches("{message}").count();

            for i in 0..num_of_messages {
                html_file_content = html_file_content.replacen("{message}", &m[i], 1);
            }

            html_file_content
    
        },
        None => html_file_content.replace("{message}", ""),
    };

    Ok(html_file_content)
}

pub fn render_myfiles(html_file_path: &str, user: &User) -> Result<String, String>  {
    let mut html_file = match fs::OpenOptions::new().read(true).open(html_file_path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Error encountered while opening file: {}", e))
    };
    
    let mut html_file_content = String::new();

    match html_file.read_to_string(&mut html_file_content) {
        Ok(_) => (),
        Err(e) => return Err(format!("Error encountered while reading file: {}", e))
    };

    html_file_content = html_file_content.replace("{name}", user.get_name());

    let user_directory = format!("database/{}", user.get_name());
    let mut file_html_links = String::new();

    for file_name in fs::read_dir(user_directory).unwrap() {
        file_html_links = format!("{}\n<li>{}</li>", file_html_links, file_name.unwrap().path().display());
    }

    html_file_content = html_file_content.replace("{files}", &file_html_links);


    Ok(html_file_content)




    //get file names from dir and make list items of them to put in my files html
}