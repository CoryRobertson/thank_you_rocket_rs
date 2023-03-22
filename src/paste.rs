use std::net::SocketAddr;
use chrono::{DateTime, Local};
use rocket::http::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
    text: String,
    // #[serde(with = "ts_seconds")]
    post_time: DateTime<Local>,
    ip_of_poster: String,
    login_cookie_of_poster: Option<String>,
}

impl Paste {
    pub fn new(text: String, req_socket: &SocketAddr, jar: &CookieJar) -> Self {

        Paste{
            text,
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            login_cookie_of_poster: {
                if let Some(cookie) = jar.get("login") {
                    Some(cookie.to_string())
                } else {
                    None
                }
            },
        }
    }
}