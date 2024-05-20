use crate::user_manager::*;

use std::{fmt::{format, Debug}, fs, io::Read};
use rocket::fs::{TempFile, NamedFile};
use chrono::DateTime;
use chrono::offset::Utc;

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
    let mut html_file_content = render(html_file_path, Some(user), None, None).unwrap();

    let user_directory = format!("database\\{}", user.get_name());
    let mut file_html_links = String::new();

    match fs::read_dir(user_directory) {
        Ok(l) => for file in l {
            let file = file.unwrap();
    
            let file_name = file.file_name().into_string().unwrap();
            let file_size = format!("{:.2} MB", (file.metadata().unwrap().len() as f64) / 1e6);
    
            let file_time:DateTime<Utc> = file.metadata().unwrap().created().unwrap().into();
            let file_time = file_time.format("%d/%m/%y");

            let open_button = format!(
                "<form method='get' action = 'myfiles.html/open/{}' enctype='multipart/form-data'>
                    <button type='submit'>
                        <img src='open.svg' id = 'open_button'>
                    </button>
                </form>", file_name);

            let delete_button = format!(
                "<form method='post' action = 'myfiles.html/delete/{}' enctype='multipart/form-data'>
                    <button type='submit'>
                        <img src='delete.svg' id = 'delete_button'>
                    </button>
                </form>", file_name);

    
    
            file_html_links = format!("{}\n<tr><td>{}</td><td>{}</td><td>{}</td><td>{}{}</td></tr>", 
                                            file_html_links, file_name, file_size, file_time, open_button, delete_button);
            

        },
        Err(_) => file_html_links = "This user has not uploaded any files.".into()
    };

    

    html_file_content = html_file_content.replace("{files}", &file_html_links);


    Ok(html_file_content)




    //get file names from dir and make list items of them to put in my files html
}