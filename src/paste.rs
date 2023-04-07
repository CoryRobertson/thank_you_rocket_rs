use chrono::{DateTime, Local};
use rocket::http::CookieJar;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PasteContents {
    File(PathBuf),
    PlainText(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paste {
    pub content: PasteContents,
    // #[serde(with = "ts_seconds")]
    pub post_time: DateTime<Local>,
    pub ip_of_poster: String,

    // metrics of the paste, potentially will be used to allow old pastes or pastes with non-recent views or downloads to be culled.
    pub view_count: u32,
    pub download_count: u32,
    pub time_of_last_download: DateTime<Local>,
    pub time_of_last_view: DateTime<Local>,

    // login cookie stored just in case we later want to allow a paste to be private and viewable only to specified hashes.
    pub login_cookie_of_poster: Option<String>,
    // potentially add a file upload optional field for this struct.
}

impl Paste {
    pub fn new(text: String, req_socket: &SocketAddr, jar: &CookieJar) -> Self {
        Paste {
            content: PasteContents::PlainText(text),
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            view_count: 0,
            download_count: 0,
            time_of_last_download: Local::now(),
            time_of_last_view: Local::now(),
            login_cookie_of_poster: { jar.get("login").map(|cookie| cookie.to_string()) },
        }
    }
    pub fn new_file_paste(file_path: PathBuf, req_socket: &SocketAddr, jar: &CookieJar) -> Self {
        Paste {
            content: PasteContents::File(file_path),
            post_time: Local::now(),
            ip_of_poster: req_socket.ip().to_string(),
            view_count: 0,
            download_count: 0,
            time_of_last_download: Local::now(),
            time_of_last_view: Local::now(),
            login_cookie_of_poster: { jar.get("login").map(|cookie| cookie.to_string()) },
        }
    }
}
