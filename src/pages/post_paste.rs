use crate::TYRState;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;
use maud::html;

#[get("/paste/new")]
/// Page for creating a new paste
pub fn new_paste(_req: SocketAddr, state: &State<TYRState>) -> RawHtml<String> {
    RawHtml(html! {

    }.into_string())
}
