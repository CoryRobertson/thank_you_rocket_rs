use crate::message::Messages;
use maud::html;
use maud::PreEscaped;
use maud::DOCTYPE;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/")]
/// Base page that the web page loads to, contains buttons that take you to various other pages.
pub fn index(_req: SocketAddr, _messages: &State<Messages>) -> RawHtml<String> {
    // TODO: make these links for buttons open in a new tab, not in current tab.

    RawHtml(html! {
        (DOCTYPE)
        title {"Thank you rocket!"}
        h1 {"Thank you rocket!"}
        p {"Welcome to thank you rocket!"}
        (PreEscaped("<button onclick=\"window.location.href=\'/new\';\">Write a message</button>"))
        br;
        (PreEscaped("<button onclick=\"window.location.href=\'/view\';\">View written messages</button>"))
    }.into_string())
}
