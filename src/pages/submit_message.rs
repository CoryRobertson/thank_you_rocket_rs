use crate::message::{Message, Messages, NewMessage};
use crate::state_management::save_messages;
use crate::user::User;
use crate::{MESSAGE_LENGTH_CAP, MESSAGE_LENGTH_MIN};
use chrono::Utc;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::State;
use std::net::SocketAddr;
use std::time::SystemTime;

#[post("/submit_message", data = "<message>")]
/// Route for submitting a message, requires post request data that can fill out the form of a new message, verifies the message for various indicators that it shouldn't be saved.
pub fn submit_message(
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
