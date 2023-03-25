use chrono::{DateTime, Local};
use rocket::http::CookieJar;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PasteContents {
    File(PathBuf),
    PlainText(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paste {
    pub content: PasteContents, // TODO: replace this text field with a content field that is a enum that contains either a string of text called plaintext or a file of data called a file.
    // #[serde(with = "ts_seconds")]
    pub post_time: DateTime<Local>,
    pub ip_of_poster: String,
    pub login_cookie_of_poster: Option<String>,
    // potentially add a file upload optional field for this struct.
}

impl Paste {
    pub fn new(text: String, req_socket: &SocketAddr, jar: &CookieJar) -> Self {
        Paste {
            content: PasteContents::PlainText(text),
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            login_cookie_of_poster: { jar.get("login").map(|cookie| cookie.to_string()) },
        }
    }
    pub fn new_file_paste(file_path: PathBuf, req_socket: &SocketAddr, jar: &CookieJar) -> Self {
        Paste{
            content: PasteContents::File(file_path),
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            login_cookie_of_poster: { jar.get("login").map(|cookie| cookie.to_string()) },
        }
    }
}
