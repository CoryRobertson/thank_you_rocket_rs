use maud::{html, PreEscaped};
use rocket::response::content::RawHtml;

#[catch(404)]
pub fn not_found() -> RawHtml<String> {
    let error_text =
        "lmao dunno what page this was meant to be, but hopefully you are happy with this cat!";
    RawHtml(
        html! {
            (error_text)
            br;
            br;
            (PreEscaped("<img src=\"static/cat.jpg\" alt=\"Cat\" />"))
        }
        .into_string(),
    )
}
