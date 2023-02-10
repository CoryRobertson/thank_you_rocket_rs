// TODO: implement a random uuid as a password, generated at runtime, navigating to this page displays all messages sent and stored in the state.
//  password would use the uuid crate, and have a page that is of low route priority, and takes in any string, validates the priority, then displays the content, if not, displays the 404 error
//  this will use a request guard!
//  this will also use a specific page that stores the key as a cookie?

// TODO: on first run of program (potentially use a file existing or not as a marker? maybe with password hash stored inside?) ask the first person to connect to "/admin" to type in a password, since
//  they are the first person there, take what ever they type, store its hash, and use that for login.

// TODO: store the fact that a user is an admin by using a cookie? if this is the correct way to do this.

// TODO: create a request guard that required admin rights to go to any page with this request guard.

// most of this is written on https://api.rocket.rs/v0.5-rc/rocket/request/trait.FromRequest.html !
// structure will be mostly: user connects to admin only page (a page with the admin request guard) -> request guard checks users cookies for a hashed password that fits
// the admins password -> if cookie is correct, let them in, if not, redirect them somewhere else or just 404 them or something.
// potentially put all of this in some module, but maybe not?

use crate::state_management::TYRState;
use maud::{html, PreEscaped};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::response::content::RawHtml;
use rocket::{request, Request, State};

pub struct IsAdminGuard;

impl Default for IsAdminGuard {
    fn default() -> Self {
        IsAdminGuard {}
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IsAdminGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(login_cookie) = req.cookies().get("login") {
            let outcome: &State<TYRState> = req.guard::<&State<TYRState>>().await.unwrap();
            if outcome
                .admin_state
                .read()
                .unwrap()
                .admin_hashes
                .contains(&login_cookie.value().to_string())
            {
                return Outcome::Success(IsAdminGuard::default());
            }
        }
        Outcome::Forward(())
    }
}

#[get("/admin")]
pub fn admin(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {

    let messages = state.messages.read().unwrap().clone();
    let message_list = {
        let mut output = String::new();
        for (ip,user) in messages {
            output.push_str(&format!("[{}]:<br>", ip));
            user.messages.iter().for_each(|message| {
                let escaped = html_escape::encode_safe(&message.text);
                output.push_str(&format!("{} : {} <br>", message.time_stamp,escaped));
            });
        }
        output
    };

    RawHtml(
        html! {
            p {"you are an admin!"}
            (PreEscaped(message_list))
        }
        .into_string(),
    )
}
