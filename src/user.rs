use crate::message::{Message, NewMessage};
use crate::POST_COOLDOWN;
use chrono::Utc;
use rocket::form::Form;
use rocket::http::Cookie;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// A user struct is a the value portion of a hashmap with a key of an ip address, struct contains a timestamp of the time they last posted, and a vector of all their messages.
pub struct User {
    pub messages: Vec<Message>,
    pub last_time_post: SystemTime,
}

impl Default for User {
    /// Default user is a timestamp that is taken immediately and an empty message struct.
    fn default() -> Self {
        Self {
            messages: vec![],
            last_time_post: SystemTime::now(),
        }
    }
}

impl User {
    /// Create a new user from a list of messages, time of last post established
    pub(crate) fn new(message: Message) -> Self {
        Self {
            messages: vec![message],
            last_time_post: SystemTime::now(),
        }
    }
    /// Add a new message to a user, and update their last time of posting
    pub(crate) fn push(&mut self, msg: String, hash: Option<&Cookie>) {
        let time = Utc::now();

        let message: Message = Message {
            text: msg,
            time_stamp: time,
            user_hash: { hash.map(|cookie| cookie.value().to_string()) },
        };
        self.messages.push(message);
        self.last_time_post = SystemTime::now();
    }
    /// Returns true if the user can post, and false if the user can not post.
    pub(crate) fn can_post(&self) -> bool {
        match SystemTime::now().duration_since(self.last_time_post) {
            Ok(dur) => dur.as_secs() >= POST_COOLDOWN,
            Err(_) => false,
        }
    }

    /// Returns true if the user has already sent this message before, only checks text
    /// Returns false if the user has not sent this message
    pub(crate) fn is_dupe_message(&self, msg: &Form<NewMessage>) -> bool {
        self.messages
            .iter()
            .map(|msg| &msg.text)
            .any(|x| x == &msg.msg)
    }
}
