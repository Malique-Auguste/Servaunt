use std::fs;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

use rocket::http::{Cookie, CookieJar};
use rocket::form::FromForm;

use serde::{Deserialize, Serialize};

//This struct is responsible for managing the state of user data. 
//That is adding new users and logging them in
#[derive(Debug, Default)]
pub struct UserManager {
    users: Vec<User>,
    sessions: Vec<Session>
}

impl UserManager {
    //Creates a new UserManager with no active sessions and a list of users from the database
    pub fn new() -> Result<UserManager, String> {
        //opens file
        let mut user_data_file = match fs::OpenOptions::new().read(true).write(true).create(true).open("database/user-data.txt") {
            Ok(file) => file,
            Err(e) =>  return Err(format!("Error on opening user-data file: {}", e))
        };

        //reads file to users string
        let mut users_as_string = String::new();
        user_data_file.read_to_string(&mut users_as_string).unwrap();

        //if the file is empty, then a usermanager with no users is returned
        if users_as_string == String::from("") {
            Ok(
                UserManager {
                    users: Vec::new(),
                    sessions: Vec::new()
                }
            )
        }
        //if the file contains data a UserManager with users as read from the file is returned
        else {
            //splits users string and converts json to user struct
            let mut users: Vec<User> = Vec::new();
            
            for user_str in users_as_string.split("\r\n") {
                users.push(match serde_json::from_str::<User>(user_str) {
                    Ok(u) => u,
                    Err(e) => return Err(format!("Error on interpreting json from user-data file: {}", e))
                })
            }

            Ok(
                UserManager {
                    users,
                    sessions: Vec::new()
                }
            )
        }
    }

    //Adds a new user to the file based on logindata received form the client
    pub fn add_new_user(&mut self, new_user: User) -> Result<(), String> {
        //checks if user name already exists, if it doesn't then it is added to the list
        if self.users.iter().any(|user| user.name == new_user.name) {
            Err(format!("User named '{}' already exists.", new_user.name))
        }
        else {
            self.users.push(new_user);
            Ok(())
        }
    }

    //logs in a user by creating a session and returning a session cookie
    pub fn login_user(&mut self, current_user: &User, jar: &CookieJar) -> Result<(), String> {
        //checks if user name already exists, if so a session is created
        if self.users.iter().any(|user| user.name == current_user.name && user.hash == current_user.hash) {
            //deactivates existin sessions for users
            self.sessions.retain(|sess| sess.user_name != current_user.name);

            //creates session for user
            let current_session = Session::new(current_user);
            
            //adds session data to cookies
            jar.add(("session-id", current_session.id.to_string()));

            //saves session
            self.sessions.push(Session::new(current_user));
            Ok(())
        }
        else {
            Err(format!("User name or password provided doesn't correspond to any existing users."))
        }
    }

    pub fn get_current_user(&self, jar: &CookieJar) -> Result<&User, String> {
        let session_id: u64 = match jar.get("session-id") {
            Some(cookie) => match cookie.value_trimmed().parse() {
                Ok(n) => n,
                Err(e) => return Err(format!("Error on parsing session-id as a number: {}", e))
            },
            None => return Err(format!("Session-id cookie could not be found."))
        };

        let user_name = match self.sessions.iter().find(|sess| sess.id == session_id) {
            Some(sess) => &sess.user_name,
            None => return Err(format!("Session ID in cookie doesn't match any live on the server."))
        };

        Ok(self.users.iter().find(|user| user.name == *user_name).unwrap())
    }
}

impl Drop for UserManager {
    //Saves all users (this includes new users) to data file on dropping of the struct form memory
    //This tupically occurs when the server goes offline
    fn drop(&mut self) {
        let file_data = self.users.iter().map(|user| serde_json::to_string(user).unwrap()).collect::<Vec<String>>().join("\r\n");

        let mut user_data_file = fs::OpenOptions::new().write(true).create(true).open("database/user-data.txt").unwrap();
        //writeln!(user_data_file, "{:?}", file_data).unwrap();
        user_data_file.write_all(file_data.as_bytes()).unwrap();
    }
}

//this represents a user being logged in, for how long, and their activity.
//this is stored in the data manager and the session id is stored in a client side cookie
//this is done to verify that the client is who they say they are
#[derive(Debug)]
pub struct Session {
    user_name: String,
    id: u64,
    creation_timestamp: u64,
    last_action_timestamp: u64 
}

impl Session {
    //the id represents a hash of the uer name and their login time so that the cookie cannot be guessed
    fn new(user: &User) -> Session {
        let mut hasher = DefaultHasher::new();

        let user_name = user.name.clone();

        //this represents the amount of time passed in seconds since midnight on the 1st of Januray 1970. 
        let creation_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        format!("{}{}", user_name, creation_timestamp).hash(&mut hasher);
        
        let id = hasher.finish();

        Session {
            user_name,
            id,
            creation_timestamp,
            last_action_timestamp: creation_timestamp
        }
    }
}


//Represents a User and their unique hash
//The hash is a unique way of storing the password so that it cannot be determined by brute force attacks 
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    name: String,
    hash: u64,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.hash)
    } 
}

impl User {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

//This represents the login data that is submitted from clients vs http
#[derive(FromForm)] 
pub struct LoginData {
    name: String,
    password: String
}

impl LoginData {
    pub fn new(name: String, password: String)-> LoginData {
        LoginData {name, password}
    }

    pub fn to_user(&self) -> User {
        //hashes password after "salting" it with the user name.
        let mut hasher = DefaultHasher::new();
        format!("{}{}", self.password, self.name).hash(&mut hasher);

        User {
            name: self.name.clone(),
            hash: hasher.finish(),
        }
    }
}