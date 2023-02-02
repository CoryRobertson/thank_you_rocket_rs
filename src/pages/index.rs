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
        p {"Welcome to thank you rocket, my home page!"}
        p {"You can write a message, viewable only to people from the same ip address as the user who sent the message, and myself, the website host."}
        p {"Feel free to write a message if anything I have made was interesting to you, or if I helped in any sort of way. :)"}
        // br;
        (PreEscaped("<button onclick=\"window.location.href=\'/new\';\">Write a message</button>"))
        br;
        (PreEscaped("<button onclick=\"window.location.href=\'/view\';\">View written messages</button>"))
        br;
        h3 {"Browser Capable Projects:"}
        // br;
        a href="/rhythm_rs" {"Rhythm Rs"}
        br;
        a href="/discreet_math_fib" {"Fibonacci Series"}
        br;
        br;
        a href="https://github.com/CoryRobertson" {"github.com/CoryRobertson"}

    }.into_string())
}
