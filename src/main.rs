#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use rocket::{Build, Rocket, State};
use rocket::form::{Form, Strict};
use rocket::http::RawStr;
use rocket::request::FromRequest;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;

struct Messages {
    messages: Arc<
        Mutex<
            HashMap<String,Vec<String>> // hash map consists of the ip address of the user: a vector of the messages they have left
        >
    >,
}



impl Default for Messages {
    fn default() -> Self {
        Messages{ messages: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[get("/")]
fn index(req: SocketAddr, messages: &State<Messages>) -> String {
    let mut lock = messages.messages.lock().unwrap();
    let user_ip = &req.ip().to_string();
    let msg_vec = match lock.get_mut(user_ip) {
        None => {
            let mut new_vec = vec![]; // create a new vector and add it to this users ip address
            new_vec.push("____".to_string()); // eventually push the message they sent, not just underscores
            lock.insert(user_ip.to_string(), new_vec); // insert the new vector with the key of the users ip address
            lock.get(&user_ip.to_string()).unwrap()
        }
        Some(msg_vec) => {
            msg_vec.push("____".to_string());
            msg_vec
        }
    };
    format!("{}: {:?}",user_ip , msg_vec).to_string()
}

#[derive(FromForm,Debug)]
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

                <input type="submit" value="Create Account">
            </form>
        </body>
    </html>
    "#)
}

#[post("/submit_message", data = "<message>")]
fn submit_message(message: Form<NewMessage>) -> String {
    println!("{:#?}", message.msg);

    message.msg.to_string()

}

#[get("/submit_message")]
fn submit_message_no_data() -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}


#[launch]
fn rocket() -> Rocket<Build> { // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.
    rocket::build()
        .manage(Messages::default())
        .mount("/", routes![index, submit_message,new,submit_message_no_data])
}
