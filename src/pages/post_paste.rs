use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::TYRState;
use rocket::response::content::RawHtml;
use rocket::State;
use rocket::response::Redirect;
use std::net::SocketAddr;
use maud::{html, PreEscaped};
use rocket::form::Form;
use rocket::http::CookieJar;
use crate::paste::Paste;

#[derive(FromForm, Debug, Clone)]
/// Form struct for a message
pub struct NewPaste {
    pub text: String,
}

#[post("/paste/new",data = "<paste>")]
pub fn new_paste_post(paste: Form<NewPaste>, req: SocketAddr, state: &State<TYRState>, jar: &CookieJar) -> Redirect {
    println!("{:?}", paste);
    let mut hasher = DefaultHasher::new();
    paste.text.hash(&mut hasher);
    let text_hash = hasher.finish();
    println!("hash: {}", text_hash);
    let mut lock = state.pastes.write().unwrap();
    let paste_struct = Paste::new(paste.text.clone(),&req,jar);
    lock.insert(text_hash,paste_struct);


    let uri = uri!(view_paste(text_hash));
    Redirect::to(uri)
}

#[get("/paste/view/<paste_id>")]
pub fn view_paste(paste_id: u64, _req: SocketAddr, state: &State<TYRState>) -> RawHtml<String> {

    let binding = state.pastes.read().unwrap();
    let paste_opt = binding.get(&paste_id);
    // TODO: escape the text stored here, as it can easily xss, also, put it in a PreEscaped object in html so it can render new lines, might need to replace all \r\n's with <br>'s

    RawHtml(html! {
        @if let Some(paste) = paste_opt {
            (paste.text)
        }
    }.into_string())
}

#[get("/paste/new")]
/// Page for creating a new paste
pub fn new_paste(_req: SocketAddr, _state: &State<TYRState>) -> RawHtml<String> {
    //<input type="text" name="text" id="text">
    RawHtml(html! {
    (PreEscaped(r#"
            <form action="/paste/new" method="post">
                <label for="ip">Enter paste</label>
                <br>
                    <textarea rows = "5" cols = "60" name = "text"></textarea>
                <br>
                <input type="submit" value="Submit paste">
            </form>
    "#))
    }.into_string())
}
