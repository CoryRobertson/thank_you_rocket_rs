// #![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#[macro_use]
extern crate rocket;
use crate::message::{get_message_list_from_ip, Message, Messages, NewMessage};
use crate::metrics::Metrics;
use crate::state_management::{load_messages, save_messages};
use crate::user::User;
use chrono::Utc;
use maud::{html, PreEscaped, DOCTYPE};
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{Build, Rocket, State};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

mod message;
mod metrics;
mod state_management;
mod user;

/// The duration in seconds that a user must wait between each message. debug only
#[cfg(debug_assertions)]
pub static POST_COOLDOWN: u64 = 5;

/// The duration in seconds that a user must wait between each message. release only
#[cfg(not(debug_assertions))]
pub static POST_COOLDOWN: u64 = 60;

/// The maximum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_CAP: usize = 150;

/// The minimum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_MIN: usize = 3;

pub static SERDE_FILE_NAME: &str = "messages.ser";
pub static RENDER_FILE_NAME: &str = "messages.sav";

#[get("/view")]
/// A page to view all messages sent by this specific user, uses their ip address to look them ip in the hash map.
fn view(req: SocketAddr, messages: &State<Messages>) -> RawHtml<String> {
    let msg_vec = get_message_list_from_ip(&req, messages);

    let message_list: String = {
        let mut string_list = String::new();

        msg_vec.into_iter().for_each(|msg| {
            // make a vector full of all of the messages this specific user has sent
            let escaped = html_escape::encode_safe(&msg);
            // append each message they sent, after escaping it
            string_list.push_str(&format!("{}<br>", escaped));
            // this text is escaped, but we put a line break after so it has one line per message

            // string_list // return this string, which gets collected as a single string
        });
        string_list
    }; // message list is a string that is pre escaped, has line breaks between each message sent.
    let user_ip = req.ip().to_string();
    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";
    RawHtml(
        html! {
           h1 {"Messages sent:"}
            (format!("IP: {}", user_ip))
            br;
            br;
            (PreEscaped(message_list))
            br;
            (PreEscaped(back_button))
            br;
        }
        .into_string(),
    )
}

#[get("/")]
/// Base page that the web page loads to, contains buttons that take you to various other pages.
fn index(_req: SocketAddr, _messages: &State<Messages>) -> RawHtml<String> {
    // TODO: make these links for buttons open in a new tab, not in current tab.

    RawHtml(html! {
        (DOCTYPE)
        title {"Thank you rocket!"}
        h1 {"Thank you rocket!"}
        p {"Welcome to thank you rocket!"}
        (PreEscaped("<button onclick=\"window.location.href=\'/new\';\">Write a message</button>"))
        br;
        (PreEscaped("<button onclick=\"window.location.href=\'/view\';\">View written messages</button>"))
    }.into_string())
}

#[get("/new")]
/// Page for creating a new message
fn new(_req: SocketAddr, _messages: &State<Messages>) -> RawHtml<String> {
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
    "#
        .parse()
        .unwrap(),
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

    if message.msg.len() < MESSAGE_LENGTH_MIN {
        return Redirect::to(uri!("/too_short")); // early return to tell the user their message is too short
    }

    let mut lock = messages.messages.lock().unwrap();
    let user_ip = &req.ip().to_string();
    match lock.get_mut(user_ip) {
        None => {
            // let mut new_vec = vec![]; // create a new vector and add it to this users ip address
            // new_vec.push(message.msg.to_string()); // eventually push the message they sent, not just underscores
            let msg = Message {
                text: message.msg.to_string(),
                time_stamp: Utc::now(),
            };
            lock.insert(user_ip.to_string(), User::new(msg)); // insert the new vector with the key of the users ip address
        }
        Some(user) => {
            // let time_since_last_post = SystemTime::now().duration_since(user.last_time_post).unwrap().as_secs();
            if user.can_post() {
                if user.is_dupe_message(&message) {
                    return Redirect::to(uri!("/duplicate"));
                } // check if the user is about to post a duplicate message

                // if the last time the user posted was 5 or more seconds ago
                user.push(message.msg.to_string()); // push their new message, this also updates their last time of posting
            } else {
                user.last_time_post = SystemTime::now();
                return Redirect::to(uri!("/slow_down")); // early return and tell the user to slow down
            }
        }
    };

    save_messages(lock);

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

#[get("/too_short")]
/// Route for having the message sent be too long
fn too_short() -> String {
    "That message is too short. :)".to_string()
}

#[get("/duplicate")]
/// Route for having the message sent be too long
fn duplicate() -> String {
    "That message is a duplicate message.".to_string()
}

#[get("/error_message")]
/// Route for having the message contain bad characters
fn error_message() -> String {
    "An unexpected error occurred. ¯\\_(ツ)_/¯".to_string()
}

#[get("/submit_message")]
/// Route for redirecting the user from a bad submit message request
fn submit_message_no_data(_req: SocketAddr, _messages: &State<Messages>) -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}

#[launch]
fn rocket() -> Rocket<Build> {
    // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.

    let load = load_messages();
    println!("Loaded message data: {:?}", load.messages);
    let state = Messages {
        messages: Arc::new(Mutex::new(load.messages)),
        banned_ips: {
            if let Ok(file) = File::open("./banned_ips.txt") {
                let br = BufReader::new(file);
                let lines: Vec<String> = br
                    .lines() // parse the lines out in a super fun way, probably unnecessary as a banned_ips file is unlikely to be full of errors, but this was super fun to make.
                    .filter_map(|line| {
                        // filter out only valid lines
                        match line {
                            Ok(l) => Some(l),
                            Err(_) => None,
                        }
                    })
                    .filter(|line| {
                        // check that there are 4 valid u8 numbers in the ip address
                        // 1.2.3.4 1111.2222.3333.4444
                        // possible inputs, only the left one should be considered possibly valid.
                        let num_len_valid: Vec<&str> = line
                            .split('.') // split the line given by its periods
                            .filter(|num_split| {
                                // only keep lines that are possible to be parsed into a 8u
                                num_split.parse::<u8>().is_ok()
                            })
                            .collect();
                        num_len_valid.len() == 4 // there needs to be exactly 4 valid u8 numbers to allow this given line to be kept.
                    })
                    .collect();
                lines
            } else {
                vec![]
            }
        },
    };

    let metrics_fairing: Metrics = Metrics {
        banned_ips: state.banned_ips.clone(),
    };

    println!("Loaded banned ips: {:?}", state.banned_ips);

    // TODO: embed a previous wasm project e.g. rhythm_rs as dockerfile build time, also use a pattern match to optionally build without it for debug builds. (use rocket::FileServer for this)
    //  use a build.rs buildscript to auto download rhythm_rs and build:
    //  check if directory exists for rhythm_rs
    //  if exists, go to next step, else, git clone it.
    //  trunk build --release in its directory (possibly use a shell script for this if the build script tool in cargo doesnt like multi command chains e.g. a change directory followed by a command.),
    //  then move its contents into where ever the program expects it to be, like /static or something.

    rocket::build()
        .manage(state)
        .mount(
            "/",
            routes![
                index,
                submit_message,
                new,
                submit_message_no_data,
                view,
                slow_down,
                too_long,
                too_short,
                duplicate,
                error_message,
            ],
        )
        .mount("/public", FileServer::from(relative!("static"))) // program crashes if static folder does not exist.
        .attach(metrics_fairing)
}
