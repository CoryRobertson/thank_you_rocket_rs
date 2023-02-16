use crate::common::is_ip_valid;
use crate::state_management::{save_program_state, TYRState};
use maud::{html, PreEscaped};
use rocket::form::Form;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{request, Request, State};
use std::path::PathBuf;

#[derive(Default)]
/// Request guard that requires an admin cookie.
pub struct IsAdminGuard(String);

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
                return Outcome::Success(IsAdminGuard(login_cookie.value().to_string()));
            }
        }
        Outcome::Forward(())
    }
}

#[get("/admin/metrics")]
/// Admin only page for viewing metrics of the site.
pub fn admin_metrics(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let unique_users_lock = state.unique_users.read().unwrap();

    let total_requests = unique_users_lock
        .iter()
        .map(|(_, user)| user.request_count)
        .sum::<u64>();
    let unique_users_count = unique_users_lock.len();

    let metrics_string = {
        let mut output = String::new();
        for (ip, user_metric) in unique_users_lock.iter() {
            output.push_str(&format!("[{}]: {} <br>", ip, user_metric.request_count));
        }
        output
    };

    let back_button = "<button onclick=\"window.location.href=\'/admin\';\">Go back</button>";

    RawHtml(
        html! {
            (PreEscaped(back_button))
            p { "Total requests: " (total_requests) }
            p { "Unique users: " (unique_users_count) }
            p { (PreEscaped(metrics_string)) }
        }
        .into_string(),
    )
}

#[get("/admin")]
/// Admin only page for displaying all messages sent to the server, as well as a few tools.
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
    let metrics_button =
        "<button onclick=\"window.location.href=\'/admin/metrics\';\">Metrics</button>";
    let banned_ips = format!("{:?}", state.banned_ips.read().unwrap());

    // TODO: add ip input field for admin resetting cooldown for a given ip address, should probably just set their last post time to unix epoch? or possibly set a boolean on their user?

    RawHtml(
        html! {
            p {"you are an admin!"}
            (PreEscaped(
                r#"
                <form action="/admin/ban_ip" method="post">
                    <label for="ip">Enter ip</label>
                    <br>
                    <input type="text" name="ip" id="ip">
                    <br>
                    <p>
                        Check to ban, uncheck to unban:
                        <input type="checkbox" name="ban_check_box" id="ban_check_box" value="true">
                    </p>
                    <input type="submit" value="Submit Ip">
                </form>
                "#
            ))
            br;
            (banned_ips)
            br;
            br;
            (PreEscaped(back_button))
            (PreEscaped(metrics_button))
            br;
            br;
            (PreEscaped(message_list))
        }
        .into_string(),
    )
}

#[derive(FromForm, Debug, Clone)]
/// Struct for the form used when handling an ip address.
pub struct Ip {
    pub ip: String,
    pub ban_check_box: bool,
}

#[post("/admin/ban_ip", data = "<ip>")]
/// Route for banning an ip, requires an admin cookie, and a form submission containing an ip address.
pub fn ban_ip(_is_admin: IsAdminGuard, state: &State<TYRState>, ip: Form<Ip>) -> Redirect {
    if is_ip_valid(&ip.ip) {
        if ip.ban_check_box {
            state.banned_ips.write().unwrap().push(ip.ip.clone());
        } else {
            let banned_ips = { state.banned_ips.read().unwrap().clone() };
            for (index,loop_ip) in banned_ips.iter().enumerate() {
                if loop_ip.eq(&ip.ip) {
                    state.banned_ips.write().unwrap().remove(index);
                }
            }
        }
        save_program_state(state, &PathBuf::from("./output/state.ser"));
    }
    Redirect::to(uri!("/admin"))
}
