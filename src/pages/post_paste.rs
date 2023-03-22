use crate::paste::Paste;
use crate::TYRState;
use maud::{html, PreEscaped};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::State;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;

#[derive(FromForm, Debug, Clone)]
/// Form struct for a message
pub struct NewPaste {
    pub text: String,
}

#[post("/paste/new", data = "<paste>")]
/// Post request handler for creating new pastes.
pub fn new_paste_post(
    paste: Form<NewPaste>,
    req: SocketAddr,
    state: &State<TYRState>,
    jar: &CookieJar,
) -> Redirect {
    println!("{:?}", paste);
    let mut hasher = DefaultHasher::new();
    paste.text.hash(&mut hasher);
    let text_hash = hasher.finish();
    println!("hash: {}", text_hash);
    let mut lock = state.pastes.write().unwrap();
    let paste_struct = Paste::new(paste.text.clone(), &req, jar);
    lock.insert(text_hash, paste_struct);

    let uri = uri!(view_paste(text_hash));
    Redirect::to(uri)
}

#[get("/paste/view/<paste_id>")]
/// Page for viewing created pastes.
pub fn view_paste(paste_id: u64, _req: SocketAddr, state: &State<TYRState>) -> RawHtml<String> {
    let binding = state.pastes.read().unwrap();
    let paste_opt = binding.get(&paste_id);

    //TODO: further test the quality of this escaping, just incase :)

    let escaped = match paste_opt {
        None => "".to_string(),
        Some(text_paste) => {
            let escaped = html_escape::encode_safe(&text_paste.text);
            escaped.replace("\r\n", "<br>").replace('\n', "<br>")
        }
    };

    RawHtml(
        html! {
            (PreEscaped(escaped))
        }
        .into_string(),
    )
}

#[get("/paste/new")]
/// Page for creating a new paste
pub fn new_paste(_req: SocketAddr, _state: &State<TYRState>) -> RawHtml<String> {
    //<input type="text" name="text" id="text">
    RawHtml(
        html! {
        (PreEscaped(r#"
            <form action="/paste/new" method="post">
                <label for="ip">Enter paste</label>
                <br>
                    <textarea rows = "5" cols = "60" name = "text"></textarea>
                <br>
                <input type="submit" value="Submit paste">
            </form>
    "#))
        }
        .into_string(),
    )
}
