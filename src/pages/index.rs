use crate::pages::login::login;
use crate::TYRState;
use crate::VERSION;
use maud::html;
use maud::PreEscaped;
use maud::DOCTYPE;
use rocket::form::validate::Contains;
use rocket::http::CookieJar;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/")]
/// Base page that the web page loads to, contains buttons that take you to various other pages.
pub fn index(_req: SocketAddr, state: &State<TYRState>, jar: &CookieJar) -> RawHtml<String> {
    // TODO: make these links for buttons open in a new tab, not in current tab.

    if !state.admin_state.read().unwrap().admin_created {
        return login();
    } // if no admin exists, force the first user to login.

    let is_admin;

    let version_number_test = format!("v{}", VERSION.unwrap_or("UNKNOWN VERSION"));

    let login_info: String = match jar.get("login") {
        None => {
            is_admin = false;
            "Not logged in.".to_string()
        }
        Some(logged_in_cookie) => {
            is_admin = state
                .admin_state
                .read()
                .unwrap()
                .admin_hashes
                .contains(logged_in_cookie.value().to_string());
            if is_admin {
                "Logged in as admin.".to_string()
            } else {
                "Logged in.".to_string()
            }
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
        @if is_admin {
            a href="/admin" {"Admin Panel"}
        }
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
