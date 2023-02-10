use crate::state_management::TYRState;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use rocket::State;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(FromForm, Debug, Clone)]
/// Form struct for a message
pub struct NewMessage {
    pub msg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A message is a struct that contains the time they sent that individual message, as well as the text of the message itself.
pub struct Message {
    pub text: String,
    #[serde(with = "ts_seconds")]
    pub time_stamp: DateTime<Utc>,
}

/// A function that outputs a vector of all the messages sent by a given ip address
pub fn get_message_list_from_ip(req: &SocketAddr, messages: &State<TYRState>) -> Vec<String> {
    let user_ip = &req.ip().to_string();
    let msg_vec = match messages.messages.read().unwrap().get(user_ip) {
        None => {
            vec![]
        }
        Some(user) => user.messages.clone(),
    };
    msg_vec.into_iter().map(|msg| msg.text).collect()
}
