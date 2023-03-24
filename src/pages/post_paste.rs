use crate::pages::outcome_pages::paste_404;
use crate::paste::{Paste, PasteContents};
use crate::verified_guard::GetVerifiedGuard;
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
    pub custom_url: Option<String>,
}

#[post("/paste/new", data = "<paste>")]
/// Post request handler for creating new pastes.
pub fn new_paste_post(
    paste: Form<NewPaste>,
    req: SocketAddr,
    state: &State<TYRState>,
    jar: &CookieJar,
    is_verified: GetVerifiedGuard,
) -> Redirect {
    let mut hasher = DefaultHasher::new();
    paste.text.hash(&mut hasher);
    let text_hash = hasher.finish();
    let mut lock = state.pastes.write().unwrap();
    let paste_struct = Paste::new(paste.text.clone(), &req, jar);
    // custom url is either the forms given custom url, or the text hash if no custom url is given.
    let custom_url = paste.custom_url.clone().unwrap_or(text_hash.to_string());
    let url_already_exists = { lock.iter().map(|(id, _)| id).any(|id| id == &custom_url) }; // variable for if the given custom url already exists

    if is_verified.0 && !url_already_exists {
        // if the user is both verified, and this given custom url does not exist.
        lock.insert(custom_url.clone(), paste_struct);
        let uri = uri!(view_paste(custom_url));
        Redirect::to(uri)
    } else {
        lock.insert(text_hash.to_string(), paste_struct);
        let uri = uri!(view_paste(text_hash.to_string()));
        Redirect::to(uri)
    }
}

#[get("/paste/view/<paste_id>")]
/// Page for viewing created pastes.
pub fn view_paste(paste_id: String, _req: SocketAddr, state: &State<TYRState>) -> RawHtml<String> {
    let binding = state.pastes.read().unwrap();
    let paste_opt = binding.get(&paste_id);

    //TODO: further test the quality of this escaping, just incase :)

    let escaped = match paste_opt {
        None => paste_404(),
        Some(text_paste) => {
            match &text_paste.content {
                PasteContents::File => {
                    "FILE PASTE, NO DISPLAY YET".to_string()
                }
                PasteContents::PlainText(text) => {
                    let escaped = html_escape::encode_safe(&text);
                    escaped.replace("\r\n", "<br>").replace('\n', "<br>")
                }
            }

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
pub fn new_paste(
    _req: SocketAddr,
    _state: &State<TYRState>,
    is_verified: GetVerifiedGuard,
) -> RawHtml<String> {
    if is_verified.0 {
        RawHtml(
            html! {
            (PreEscaped(r#"
            <form action="/paste/new" method="post">
                <label for="ip">Enter paste</label>
                <br>
                    <textarea rows = "5" cols = "60" name = "text"></textarea>
                    <br>
                    <p>Custom url: </p>
                    <input type="text" name="custom_url" id="custom_url">
                <br>
                <br>
                <input type="submit" value="Submit paste">
            </form>
    "#))
            }
            .into_string(),
        )
    } else {
        // user is not verified
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
}
