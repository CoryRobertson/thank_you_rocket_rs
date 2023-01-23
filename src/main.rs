#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{Build, Rocket, State};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

struct Messages {
    // hash map consists of the ip address of the user: a vector of the messages they have left
    messages: Arc<Mutex<HashMap<String, User>>>,
}

struct User {
    messages: Vec<String>,
    last_time_post: SystemTime,
}

impl Default for User {
    fn default() -> Self {
        User { messages: vec![], last_time_post: SystemTime::now() }
    }
}

impl User {
    fn new(message_vec: Vec<String>) -> User {
        User {messages: message_vec, last_time_post: SystemTime::now() }
    }

    fn push(&mut self,msg: String) {
        self.messages.push(msg);
    }

}

impl Default for Messages {
    fn default() -> Self {
        Messages {
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[get("/view")]
fn view(req: SocketAddr, messages: &State<Messages>) -> String {
    let user_ip = &req.ip().to_string();
    let msg_vec = match messages.messages.lock().unwrap().get(user_ip) {
        None => { vec![] }
        Some(user) => { user.messages.clone() }
    };

    let user_info = format!("{:?}", msg_vec).to_string();

    user_info
}

#[get("/")]
fn index() -> RawHtml<&'static str> {
    let s = r#"
    <!DOCTYPE html>
        <html>
          <head>
            <title>Title of the document</title>
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
struct NewMessage {
    msg: String,
}

#[get("/new")]
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
fn submit_message(
    message: Form<NewMessage>,
    req: SocketAddr,
    messages: &State<Messages>,
) -> Redirect {
    println!("{:#?}", message.msg);

    let mut lock = messages.messages.lock().unwrap();
    let user_ip = &req.ip().to_string();
    match lock.get_mut(user_ip) {
        None => {
            let mut new_vec = vec![]; // create a new vector and add it to this users ip address
            new_vec.push(message.msg.to_string()); // eventually push the message they sent, not just underscores
            lock.insert(user_ip.to_string(), User::new(new_vec)); // insert the new vector with the key of the users ip address
        }
        Some(user) => {
            user.push(message.msg.to_string());
        }
    };

    // format!("IP: {}, messages: {:?}",user_ip ,msg_vec).to_string()
    Redirect::to(uri!("/"))
}

#[get("/submit_message")]
fn submit_message_no_data() -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}

#[launch]
fn rocket() -> Rocket<Build> {
    // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.
    rocket::build().manage(Messages::default()).mount(
        "/",
        routes![index, submit_message, new, submit_message_no_data, view],
    )
}
