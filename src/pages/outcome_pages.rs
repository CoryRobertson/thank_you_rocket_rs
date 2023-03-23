use crate::TYRState;
use crate::POST_COOLDOWN;
use rocket::response::Redirect;
use rocket::State;
use std::net::SocketAddr;
use std::time::SystemTime;

#[get("/slow_down")]
/// Route for requiring the user to slow down their message send rate.
pub fn slow_down(req: SocketAddr, messages: &State<TYRState>) -> String {
    let time_remaining = {
        match messages.messages.read().unwrap().get(&req.ip().to_string()) {
            None => 0,
            Some(user) => match SystemTime::now().duration_since(user.last_time_post) {
                Ok(time) => {
                    if !user.can_post() {
                        POST_COOLDOWN - time.as_secs()
                    } else {
                        0
                    }
                }
                Err(_) => 0,
            },
        }
    };

    format!(
        "\
    Please slow down, you are trying to post too often. :) \n\
    You need to wait {POST_COOLDOWN} seconds between posts.\n\
    Your remaining cooldown is {time_remaining} seconds.\
    "
    )
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
pub fn submit_message_no_data(_req: SocketAddr, _messages: &State<TYRState>) -> Redirect {
    Redirect::to(uri!("/new")) // user some how went to submit message, and there was no form data sent to the server, so we redirect them to the submit page.
}

#[get("/paste_404")]
/// Route for if a paste does not exist
pub fn paste_404() -> String {
    "That paste does not exist.".to_string()
}
