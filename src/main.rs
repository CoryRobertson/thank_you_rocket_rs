#[macro_use]
extern crate rocket;
use chrono::{DateTime, Datelike, Local, Timelike};
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{Build, Rocket, State};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::{fs, thread};

/// The duration in seconds that a user must wait between each message.
pub static POST_COOLDOWN: u64 = 5;

/// The maximum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_CAP: usize = 150;

#[derive(Clone)]
/// The state struct for the rocket web frame work.
struct Messages {
    // hash map consists of the ip address as a key, and the user struct itself.
    messages: Arc<Mutex<HashMap<String, User>>>,
}

/// Saves all messages to the system in a file.
fn save_messages(messages: MutexGuard<HashMap<String, User>>) {
    if !messages.is_empty() {
        match fs::read_dir("./output") {
            Ok(_) => {
                // output dir exists
            }
            Err(_) => {
                match fs::create_dir("./output") {
                    Ok(_) => {
                        // output dir now exists
                    }
                    Err(err) => {
                        panic!(
                            "{} \n unable to create output dir, check file permissions?",
                            err
                        )
                    }
                }
            }
        }
        let file_name = {
            // maybe use this later, at the moment not sure about it.
            // let date_for_file_name = {
            //     let ts = Local::now();
            //     format!(
            //         "{}-{}-{}",
            //         ts.year(),
            //         ts.month(),
            //         ts.day(),
            //     )
            // }; // get a new date and time stamp for the file to save to

            "./output/messages.sav".to_string()
        };

        let file = File::create(file_name).unwrap(); 
        let mut bw = BufWriter::new(file);
        for (ip, user) in messages.iter() {
            let messages = &user.messages;
            let _ = bw.write(format!("{ip}:\n").as_bytes()).unwrap();
            for msg in messages {
                let date = msg.time_stamp;
                let am_pm = match date.hour12().0 {
                    true => "PM",
                    false => "AM",
                };
                let time_format = format!(
                    "{}:{:02}:{:02}{}",
                    date.hour12().1,
                    date.minute(),
                    date.second(),
                    am_pm
                );
                let time_stamp_text = format!(
                    "{}-{}-{}: {}",
                    date.year(),
                    date.month(),
                    date.day(),
                    time_format,
                );
                let _ = bw
                    .write(format!("\t[ {} ]: {}\n", time_stamp_text, msg.text).as_bytes())
                    .unwrap();
            }
        }
        let _ = bw.flush();
    }
}

#[derive(Clone, Debug)]
/// A message is a struct that contains the time they sent that individual message, as well as the text of the message itself.
struct Message {
    text: String,
    time_stamp: DateTime<Local>,
}

#[derive(Debug)]
/// A user struct is a the value portion of a hashmap with a key of an ip address, struct contains a timestamp of the time they last posted, and a vector of all their messages.
struct User {
    messages: Vec<Message>,
    last_time_post: SystemTime,
}

impl Default for User {
    /// Default user is a timestamp that is taken immediately and an empty message struct.
    fn default() -> Self {
        User {
            messages: vec![],
            last_time_post: SystemTime::now(),
        }
    }
}

impl User {
    /// Create a new user from a list of messages, time of last post established
    fn new(message: Message) -> User {
        User {
            messages: vec![message],
            last_time_post: SystemTime::now(),
        }
    }
    /// Add a new message to a user, and update their last time of posting
    fn push(&mut self, msg: String) {
        let time = Local::now();
        let message: Message = Message {
            text: msg,
            time_stamp: time,
        };
        self.messages.push(message);
        self.last_time_post = SystemTime::now();
    }
    /// Returns true if the user can post, and false if the user can not post.
    fn can_post(&self) -> bool {
        match SystemTime::now().duration_since(self.last_time_post) {
            Ok(dur) => dur.as_secs() >= POST_COOLDOWN,
            Err(_) => false,
        }
    }
}

impl Default for Messages {
    /// Default message struct is just an empty hash map.
    fn default() -> Self {
        Messages {
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[get("/view")]
/// A page to view all messages sent by this specific user, uses their ip address to look them ip in the hash map.
fn view(req: SocketAddr, messages: &State<Messages>) -> String {
    let user_ip = &req.ip().to_string();
    let msg_vec = match messages.messages.lock().unwrap().get(user_ip) {
        None => {
            vec![]
        }
        Some(user) => user.messages.clone(),
    };
    let text_vec: Vec<String> = msg_vec.into_iter().map(|msg| msg.text).collect();

    format!("{:?}", text_vec)
}

#[get("/")]
/// Base page that the web page loads to, contains buttons that take you to various other pages.
fn index() -> RawHtml<&'static str> {
    let s = r#"
    <!DOCTYPE html>
        <html>
          <head>
            <title>Thank_you_rocker_rs</title>
          </head>
          <body>
            <button onclick="window.location.href='/new';">
              Submit new message
            </button>
            <button onclick="window.location.href='/view';">
              View messages
            </button>
          </body>
        </html>
    "#;
    RawHtml(s)
}

#[derive(FromForm, Debug)]
/// Form struct for a message
struct NewMessage {
    msg: String,
}

#[get("/new")]
/// Page for creating a new message
fn new() -> RawHtml<&'static str> {
    RawHtml(
        r#"
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Submit</title>
    </head>
        <body>
            <form action="/submit_message" method="post">
                <label for="msg">Enter message</label>
                <br>
                <input type="text" name="msg" id="msg">
                <input type="submit" value="Submit Message">
            </form>
        </body>
    </html>
    "#,
    )
}

#[post("/submit_message", data = "<message>")]
/// Route for submitting a message, requires post request data that can fill out the form of a new message, verifies the message for various indicators that it shouldn't be saved.
fn submit_message(
    message: Form<NewMessage>,
    req: SocketAddr,
    messages: &State<Messages>,
) -> Redirect {
    if !message.msg.is_ascii() {
        return Redirect::to(uri!("/error_message")); // only allow user to use ascii text in their message
    }

    if message.msg.len() > MESSAGE_LENGTH_CAP {
        return Redirect::to(uri!("/too_long")); // early return and tell the user to write shorter messages
    }

    let mut lock = messages.messages.lock().unwrap();
    let user_ip = &req.ip().to_string();
    match lock.get_mut(user_ip) {
        None => {
            // let mut new_vec = vec![]; // create a new vector and add it to this users ip address
            // new_vec.push(message.msg.to_string()); // eventually push the message they sent, not just underscores
            let msg = Message {
                text: message.msg.to_string(),
                time_stamp: Local::now(),
            };
            lock.insert(user_ip.to_string(), User::new(msg)); // insert the new vector with the key of the users ip address
        }
        Some(user) => {
            // let time_since_last_post = SystemTime::now().duration_since(user.last_time_post).unwrap().as_secs();
            if user.can_post() {
                // if the last time the user posted was 5 or more seconds ago
                user.push(message.msg.to_string()); // push their new message, this also updates their last time of posting
                                                    // user.last_time_post = SystemTime::now(); // update their last post time
            } else {
                user.last_time_post = SystemTime::now();
                return Redirect::to(uri!("/slow_down")); // early return and tell the user to slow down
            }
        }
    };

    Redirect::to(uri!("/"))
}

#[get("/slow_down")]
/// Route for requiring the user to slow down their message send rate.
fn slow_down() -> String {
    "Please slow down, you are trying to post too often :)".to_string()
}

#[get("/too_long")]
/// Route for having the message sent be too long
fn too_long() -> String {
    "That message is too long, please try to make it shorter :)".to_string()
}

#[get("/error_message")]
/// Route for having the message contain bad characters
fn error_message() -> String {
    "That message for some reason was unable to be saved (most likely contains something that is not ascii). ¯\\_(ツ)_/¯".to_string()
}

#[get("/submit_message")]
/// Route for redirecting the user from a bad submit message request
fn submit_message_no_data() -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}

#[launch]
fn rocket() -> Rocket<Build> {
    // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.
    let state = Messages::default();
    let message_reference = Arc::clone(&state.messages);

    // thread that saves the messages to the file system.
    thread::spawn(move || loop {
        {
            let lock = message_reference.lock().unwrap();
            save_messages(lock);
        }
        sleep(Duration::from_secs(5));
    }); // file save loop

    rocket::build().manage(state).mount(
        "/",
        routes![
            index,
            submit_message,
            new,
            submit_message_no_data,
            view,
            slow_down,
            too_long,
            error_message
        ],
    )
}
