use crate::message::Messages;
use rocket::response::Redirect;
use rocket::State;
use std::net::SocketAddr;

#[get("/slow_down")]
/// Route for requiring the user to slow down their message send rate.
pub fn slow_down() -> String {
    "Please slow down, you are trying to post too often :)".to_string()
}

#[get("/too_long")]
/// Route for having the message sent be too long
pub fn too_long() -> String {
    "That message is too long, please try to make it shorter :)".to_string()
}

#[get("/too_short")]
/// Route for having the message sent be too long
pub fn too_short() -> String {
    "That message is too short. :)".to_string()
}

#[get("/duplicate")]
/// Route for having the message sent be too long
pub fn duplicate() -> String {
    "That message is a duplicate message.".to_string()
}

#[get("/error_message")]
/// Route for having the message contain bad characters
pub fn error_message() -> String {
    "An unexpected error occurred. ¯\\_(ツ)_/¯".to_string()
}

#[get("/submit_message")]
/// Route for redirecting the user from a bad submit message request
pub fn submit_message_no_data(_req: SocketAddr, _messages: &State<Messages>) -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}
