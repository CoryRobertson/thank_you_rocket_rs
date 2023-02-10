use crate::message::get_message_list_from_ip;
use crate::TYRState;
use maud::html;
use maud::PreEscaped;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/view")]
/// A page to view all messages sent by this specific user, uses their ip address to look them ip in the hash map.
pub fn view(req: SocketAddr, messages: &State<TYRState>) -> RawHtml<String> {
    let msg_vec = get_message_list_from_ip(&req, messages);

    let message_list: String = {
        let mut string_list = String::new();

        msg_vec.into_iter().for_each(|msg| {
            // make a vector full of all of the messages this specific user has sent
            let escaped = html_escape::encode_safe(&msg);
            // append each message they sent, after escaping it
            string_list.push_str(&format!("{escaped}<br>"));
            // this text is escaped, but we put a line break after so it has one line per message

            // string_list // return this string, which gets collected as a single string
        });
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
