use chrono::{DateTime, Local};
use rocket::http::CookieJar;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paste {
    pub text: String,
    // #[serde(with = "ts_seconds")]
    pub post_time: DateTime<Local>,
    pub ip_of_poster: String,
    pub login_cookie_of_poster: Option<String>,
    // potentially add a file upload optional field for this struct.
}

impl Paste {
    pub fn new(text: String, req_socket: &SocketAddr, jar: &CookieJar) -> Self {
        Paste {
            text,
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            login_cookie_of_poster: { jar.get("login").map(|cookie| cookie.to_string()) },
        }
    }
}
