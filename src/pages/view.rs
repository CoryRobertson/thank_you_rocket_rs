use crate::TYRState;
use maud::html;
use maud::PreEscaped;
use rocket::http::CookieJar;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/view")]
/// A page to view all messages sent by this specific user, uses their ip address to look them ip in the hash map.
pub fn view(req: SocketAddr, state: &State<TYRState>, jar: &CookieJar) -> RawHtml<String> {
    let logged_in = jar.get("login");
    let message_list: String = {
        let mut string_list = String::new();
        // if the user is logged in, we need to render all messages that have the same hash
        // else, we need to render all messages with no hash.
        if let Some(cookie) = logged_in {
            state
                .messages
                .read()
                .unwrap()
                .iter()
                .map(|(_, user)| &user.messages)
                .for_each(|messages| {
                    for msg in messages {
                        if let Some(hash) = &msg.user_hash {
                            if &cookie.value().to_string() == hash {
                                let escaped = html_escape::encode_safe(&msg.text);
                                string_list.push_str(&format!("{escaped}<br>"));
                            }
                        }
                    }
                });
        } else {
            match state.messages.read().unwrap().get(&req.ip().to_string()) {
                None => {}
                Some(user) => {
                    for message in &user.messages {
                        if message.user_hash.is_none() {
                            // render all messages that this ip sent that don't have a hash.
                            let escaped = html_escape::encode_safe(&message.text);
                            string_list.push_str(&format!("{escaped}<br>"));
                        }
                    }
                }
            }
        };
        string_list
    }; // message list is a string that is pre escaped, has line breaks between each message sent.

    let user_ip = req.ip().to_string();

    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";

    RawHtml(
        html! {
           h1 {"TYRState sent:"}
            (format!("IP: {user_ip}"))
            br;
            br;
            (PreEscaped(message_list))
            br;
            (PreEscaped(back_button))
            br;
        }
        .into_string(),
    )
}
