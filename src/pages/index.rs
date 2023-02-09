use crate::TYRState;
use crate::VERSION;
use maud::html;
use maud::PreEscaped;
use maud::DOCTYPE;
use rocket::http::{CookieJar};
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/")]
/// Base page that the web page loads to, contains buttons that take you to various other pages.
pub fn index(
    _req: SocketAddr,
    _messages: &State<TYRState>,
    jar: &CookieJar,
) -> RawHtml<String> {
    // TODO: make these links for buttons open in a new tab, not in current tab.

    let version_number_test = format!("v{}", VERSION.unwrap_or("UNKNOWN VERSION"));


    let login_info: String = match jar.get("login") {
        None => {
            format!("Not logged in.")
        }
        Some(login_cookie) => {
            format!("Logged in, hash: {}", login_cookie.value())
        }
    };

    RawHtml(html! {
        (DOCTYPE)
        title {"Thank you rocket!"}
        h1 {"Thank you rocket!"}
        p {"Welcome to thank you rocket, my home page!"}
        p {"You can write a message, viewable only to people from the same ip address as the user who sent the message, and myself, the website host."}
        p {"Feel free to write a message if anything I have made was interesting to you, or if I helped in any sort of way. :)"}

        a href="/login" {"login"}
        br;
        br;
        a href="/logout" {"logout"}
        br;
        br;

        p { (login_info) }

        (PreEscaped("<button onclick=\"window.location.href=\'/new\';\">Write a message</button>"))
        br;
        (PreEscaped("<button onclick=\"window.location.href=\'/view\';\">View written messages</button>"))
        br;
        h3 {"Browser Capable Projects:"}
        a href="/rhythm_rs" {"Rhythm Rs"}
        br;
        a href="/discreet_math_fib" {"Fibonacci Series"}
        br;
        br;
        a href="https://github.com/CoryRobertson" {"github.com/CoryRobertson"}
        br;

        (PreEscaped("<style>
        .version-footer {
            height: 30px;
            position: fixed;
            bottom:0%;
            width:99%;
            text-align: right;
            opacity: 0.25;
        }</style>"
        ))

        div."version-footer" { (version_number_test) }

    }.into_string())
}
