use crate::state_management::TYRState;
use maud::{html, PreEscaped};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::response::content::RawHtml;
use rocket::{request, Request, State};

#[derive(Default)]
pub struct IsAdminGuard;

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
                // if the login cookie hash is one of the admin hashes, allow the user to proceed.
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
        for (ip, user) in messages {
            output.push_str(&format!("[{ip}]:<br>"));
            user.messages.iter().for_each(|message| {
                let escaped = html_escape::encode_safe(&message.text);
                let hashed = {
                    match message.user_hash {
                        None => "",
                        Some(_) => "#",
                    }
                };
                output.push_str(&format!(
                    "{} :{}: {} <br>",
                    message.time_stamp, hashed, escaped
                ));
            });
        }
        output
    };
    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";

    RawHtml(
        html! {
            p {"you are an admin!"}
            (PreEscaped(back_button))
            br;
            br;
            (PreEscaped(message_list))
        }
        .into_string(),
    )
}
