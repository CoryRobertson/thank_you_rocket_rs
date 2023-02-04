use crate::message::{Message, Messages, NewMessage};
use crate::state_management::save_messages;
use crate::user::User;
use crate::{MESSAGE_LENGTH_CAP, MESSAGE_LENGTH_MIN};
use chrono::Utc;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::State;
use std::net::SocketAddr;

#[post("/submit_message", data = "<message>")]
/// Route for submitting a message, requires post request data that can fill out the form of a new message, verifies the message for various indicators that it shouldn't be saved.
pub fn submit_message(
    message: Form<NewMessage>,
    req: SocketAddr,
    messages: &State<Messages>,
) -> Redirect {

    let user_ip = &req.ip().to_string();

    if !message.msg.is_ascii() {
        return Redirect::to(uri!("/error_message")); // only allow user to use ascii text in their message
    }

    if message.msg.len() > MESSAGE_LENGTH_CAP {
        return Redirect::to(uri!("/too_long")); // early return and tell the user to write shorter messages
    }

    if message.msg.len() < MESSAGE_LENGTH_MIN {
        return Redirect::to(uri!("/too_short")); // early return to tell the user their message is too short
    }

    {
        let lock = messages.messages.read().unwrap();
        match lock.get(user_ip) {
            None => {
                // if the user does not exist, then they are allowed to post.
            }
            Some(user) => {
                if !user.can_post() {
                    return Redirect::to(uri!("/slow_down"));
                }

                if user.is_dupe_message(&message) {
                    return Redirect::to(uri!("/duplicate"));
                }
            }
        }
    } // block for locking in read mode, the message list to check if the user is able to post, or if their message is a duplicate.

    {
        let mut lock = messages.messages.write().unwrap();
        match lock.get_mut(user_ip) {
            None => {
                let msg = Message {
                    text: message.msg.to_string(),
                    time_stamp: Utc::now(),
                }; // message object used for pushing to the user
                lock.insert(user_ip.to_string(), User::new(msg)); // insert the new vector with the key of the users ip address
            }
            Some(user) => {
                user.push(message.msg.to_string()); // push their new message, this also updates their last time of posting
            }
        };
    } // block for locking the message block in write mode.

    let lock = messages.messages.read().unwrap();
    save_messages(lock.clone());

    Redirect::to(uri!("/"))
}
