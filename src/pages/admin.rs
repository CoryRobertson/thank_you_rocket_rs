use crate::common::is_ip_valid;
use crate::metrics::UserMetric;
use crate::paste::PasteContents;
use crate::state_management::{save_program_state, TYRState};
use crate::user::User;
use crate::{ONLINE_TIMER, POST_COOLDOWN};
use chrono_tz::US::Pacific;
use maud::{html, PreEscaped};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{request, Request, State};
use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
                return Outcome::Success(Self(login_cookie.value().to_string()));
            }
        }
        Outcome::Forward(())
    }
}

#[get("/admin/metrics")]
/// Admin only page for viewing metrics of the site. Includes ip of user, request count, and last page visited.
pub fn admin_metrics(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let unique_users_lock = state.unique_users.read().unwrap();

    let total_requests = unique_users_lock
        .iter()
        .map(|(_, user)| user.request_count)
        .sum::<u64>();
    let unique_users_count = unique_users_lock.len();

    let metrics_string = {
        let mut output = String::new();
        let mut user_metrics_vector = unique_users_lock
            .iter()
            .collect::<Vec<(&String, &UserMetric)>>();

        user_metrics_vector.sort_by(|entry, second_entry| {
            second_entry
                .1
                .request_count
                .partial_cmp(&entry.1.request_count)
                .unwrap()
        });

        for (ip, user_metric) in user_metrics_vector {
            let last_page_visited = match &user_metric.last_page_visited {
                None => "".to_string(),
                Some(url) => url.to_string(),
            };
            //"/admin/metrics/<ip_address>"
            let link_to_metrics = format!("<a href=\"/admin/metrics/{0}\">{0}</a>", ip);

            output.push_str(&format!(
                "[{}]: {} : {} <br>",
                link_to_metrics, user_metric.request_count, last_page_visited
            ));
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

#[get("/admin/view_hashes")]
/// A page that displays all ip addresses and their given login hashes they have ever used.
pub fn view_hashes(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let read_lock = state.unique_users.read().unwrap();

    let mut login_hashes_string = String::new();

    let mut logins: Vec<(&String, &UserMetric)> = vec![];

    for thing in read_lock.iter() {
        logins.push(thing);
    }

    logins.sort_by(|thing, thing2| match (&thing.1.logins, &thing2.1.logins) {
        (Some(logins1), Some(logins2)) => logins1.len().partial_cmp(&logins2.len()).unwrap(),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    });

    logins.reverse();

    for (ip, user) in logins {
        let hashes = {
            match &user.logins {
                None => "".to_string(),
                Some(logins_vec) => {
                    let formatted = format!("{:?}", logins_vec); // convert vector to a string, does not need to be pretty.
                    let escaped = html_escape::encode_safe(&formatted); // escape it, just encase :)
                    escaped.to_string()
                }
            }
        };
        login_hashes_string.push_str(&format!("{}: {} <br>", ip, hashes));
    }

    let back_button = "<button onclick=\"window.location.href=\'/admin\';\">Go back</button>";

    RawHtml(
        html! {
            (PreEscaped(back_button))
            br;
            br;
            (PreEscaped(login_hashes_string))
        }
        .into_string(),
    )
}

#[get("/admin/view_cooldown")]
/// An admin only page that displays all users who are currently on cooldown.
pub fn view_cooldown(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let read_lock = state.messages.read().unwrap();
    let users_on_cooldown = read_lock
        .iter()
        .filter(|(_, user)| !user.can_post())
        .collect::<Vec<(&String, &User)>>();

    let mut cooldown_users_string = String::new();
    for (ip, user) in users_on_cooldown {
        let time_left_cooldown = match SystemTime::now().duration_since(user.last_time_post) {
            Ok(time) => POST_COOLDOWN - time.as_secs(),
            Err(_) => 0,
        };
        cooldown_users_string.push_str(&format!("{}: {} <br>", &ip, time_left_cooldown));
    }

    let back_button = "<button onclick=\"window.location.href=\'/admin\';\">Go back</button>";

    RawHtml(
        html! {
            (PreEscaped(back_button))
            br;
            br;
            p {"[IP: Time Left]"}
            (PreEscaped(cooldown_users_string))
        }
        .into_string(),
    )
}

#[get("/admin/view_online")]
/// An admin only page that displays all users who are currently on cooldown, as well as the last navigated page for that user.
pub fn view_online(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let read_lock = state.unique_users.read().unwrap();
    let users_online = read_lock
        .iter()
        .filter(|user| user.1.last_time_seen.is_some())
        .filter(|user| {
            SystemTime::now()
                .duration_since(user.1.last_time_seen.unwrap())
                .unwrap_or_default()
                .as_secs()
                <= ONLINE_TIMER
        })
        .collect::<Vec<(&String, &UserMetric)>>();

    let mut online_users_string = String::new();
    for (ip, user) in users_online {
        let last_page_visited = match &user.last_page_visited {
            None => "".to_string(),
            Some(url) => url.to_string(),
        };

        online_users_string.push_str(&format!(
            "{}: {} : {} <br>",
            &ip,
            SystemTime::now()
                .duration_since(user.last_time_seen.unwrap())
                .unwrap()
                .as_secs(),
            last_page_visited,
        ));
    }

    let back_button = "<button onclick=\"window.location.href=\'/admin\';\">Go back</button>";

    RawHtml(
        html! {
            (PreEscaped(back_button))
            br;
            br;
            p {"[IP: Last Time Seen]"}
            (PreEscaped(online_users_string))
        }
        .into_string(),
    )
}

#[get("/admin/view_pastes")]
pub fn view_pastes_admin(_is_admin: IsAdminGuard, state: &State<TYRState>) -> RawHtml<String> {
    let mut paste_list = String::new();

    let pastes = state.pastes.read().unwrap();

    for (paste_id, paste) in pastes.iter() {
        let id_escaped_paste = html_escape::encode_safe(&paste_id);
        let deletion_link_for_paste = format!(
            "<a href=\"/paste/view/{0}/delete\">DELETE</a>",
            id_escaped_paste
        );
        let link_to_paste = format!("<a href=\"/paste/view/{0}\">-{0}-</a>", id_escaped_paste);
        match &paste.content {
            PasteContents::File(path) => {
                let (file_content, file_name) = match File::open(path).ok() {
                    None => (
                        "File un-readable. Error occurred.".to_string(),
                        "NO FILE NAME GIVEN",
                    ),
                    Some(mut file) => {
                        let mut file_contents = String::new();
                        file.read_to_string(&mut file_contents).unwrap_or_default();
                        file_contents.truncate(150);
                        (
                            file_contents,
                            path.file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default(),
                        )
                    }
                };
                paste_list.push_str(&format!(
                    "[{}] : {} : {} : {} <br>",
                    link_to_paste, file_name, file_content, deletion_link_for_paste
                ));
            }
            PasteContents::PlainText(paste_text) => {
                // let paste_text = paste.text.clone();

                let escaped = html_escape::encode_safe(&paste_text); // escape the paste

                let final_text = {
                    if escaped.len() <= 150 {
                        escaped
                    } else {
                        std::borrow::Cow::Borrowed(&escaped[0..150])
                    }
                };

                paste_list.push_str(&format!(
                    "[{}] : {} : {} <br>",
                    link_to_paste, final_text, deletion_link_for_paste
                ));
            }
        }
    }

    let back_button = "<button onclick=\"window.location.href=\'/admin\';\">Go back</button>";

    RawHtml(
        html! {
            (PreEscaped(back_button))
            br;
            br;
            (PreEscaped(paste_list))

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
                    message.time_stamp.with_timezone(&Pacific),
                    hashed,
                    escaped
                ));
            });
        }
        output
    };
    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";
    let metrics_button =
        "<button onclick=\"window.location.href=\'/admin/metrics\';\">Metrics</button>";
    let view_cooldown_button = "<button onclick=\"window.location.href=\'/admin/view_cooldown\';\">View Cooldowns</button>";
    let view_hashes_button =
        "<button onclick=\"window.location.href=\'/admin/view_hashes\';\">View Hashes</button>";
    let view_online_button = "<button onclick=\"window.location.href=\'/admin/view_online\';\">View Online Users</button>";
    let view_pastes_button =
        "<button onclick=\"window.location.href=\'/admin/view_pastes\';\">View Pastes</button>";
    let banned_ips = format!("{:?}", state.banned_ips.read().unwrap());

    let verified_list = match &state.admin_state.read().unwrap().verified_list {
        None => "".to_string(),
        Some(list) => {
            format!("{:?}", list)
        }
    };

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

                    <input type="radio" id="ban" name="ip_action" value="Ban" checked>
                    <label for="ban">Ban</label><br>
                    <input type="radio" id="unban" name="ip_action" value="Unban">
                    <label for="unban">Unban</label><br>
                    <input type="radio" id="reset_cooldown" name="ip_action" value="ResetCooldown">
                    <label for="reset_cooldown">Reset Cooldown</label><br>
                    <input type="radio" id="add_verified" name="ip_action" value="AddVerified">
                    <label for="add_verified">Add Verified</label><br>
                    <input type="radio" id="remove_verified" name="ip_action" value="RemoveVerified">
                    <label for="remove_verified">Remove Verified</label><br>


                    <input type="submit" value="Submit Ip">
                </form>
                "#
            ))

            br;
            ("Verified list:")
            br;
            (verified_list)
            br;
            br;
            ("Banned ips:")
            br;
            (banned_ips)
            br;
            br;
            (PreEscaped(back_button))
            (PreEscaped(metrics_button))
            (PreEscaped(view_cooldown_button))
            (PreEscaped(view_hashes_button))
            (PreEscaped(view_online_button))
            (PreEscaped(view_pastes_button))
            br;
            br;
            (PreEscaped(message_list))
        }
        .into_string(),
    )
}

#[derive(FromFormField, Debug, Clone)]
/// Enum for determining the action to go through with, used for submitting an ip address in a form.
pub enum IpAction {
    Ban,
    Unban,
    ResetCooldown,
    AddVerified,
    RemoveVerified,
}

#[derive(FromForm, Debug, Clone)]
/// Struct for the form used when handling an ip address.
pub struct Ip {
    pub ip: String,
    pub ip_action: IpAction,
}

#[post("/admin/ban_ip", data = "<ip>")]
/// Route for banning an ip, requires an admin cookie, and a form submission containing an ip address.
pub fn ban_ip(_is_admin: IsAdminGuard, state: &State<TYRState>, ip: Form<Ip>) -> Redirect {
    match ip.ip_action {
        IpAction::Ban => {
            if is_ip_valid(&ip.ip) {
                state.banned_ips.write().unwrap().push(ip.ip.clone());
            }
        }
        IpAction::Unban => {
            if is_ip_valid(&ip.ip) {
                let banned_ips = { state.banned_ips.read().unwrap().clone() };
                for (index, loop_ip) in banned_ips.iter().enumerate() {
                    if loop_ip.eq(&ip.ip) {
                        state.banned_ips.write().unwrap().remove(index);
                        break;
                    }
                }
            }
        }
        IpAction::ResetCooldown => {
            if is_ip_valid(&ip.ip) {
                match state.messages.write().unwrap().get_mut(&ip.ip) {
                    None => {}
                    Some(user) => {
                        user.last_time_post = UNIX_EPOCH;
                    }
                }
            }
        }
        IpAction::AddVerified => {
            let mut lock = state.admin_state.write().unwrap();
            match lock.verified_list.as_mut() {
                None => {
                    lock.verified_list = Some(vec![ip.ip.to_string()]);
                }
                Some(list) => {
                    list.push(ip.ip.to_string());
                }
            }
        }
        IpAction::RemoveVerified => {
            let mut lock = state.admin_state.write().unwrap();
            match lock.verified_list.as_mut() {
                None => {}
                Some(list) => {
                    for (index, ip_in_list) in list.clone().iter().enumerate() {
                        if ip_in_list.eq(&ip.ip.to_string()) {
                            list.remove(index);
                        }
                    }
                }
            }
        }
    }
    save_program_state(state, &PathBuf::from("./output/state.ser"));

    Redirect::to(uri!("/admin"))
}

/// Returns true if the user is an admin.
/// Requirements for this are the state holding the login cookie of the user in the admin_hashes vector.
pub fn check_is_admin(state: &State<TYRState>, jar: &CookieJar) -> bool {
    return match jar.get("login") {
        None => false,
        Some(cookie) => state
            .admin_state
            .read()
            .unwrap()
            .admin_hashes
            .contains(&cookie.value().to_string()),
    };
}

#[get("/admin/metrics/<ip_address>")]
pub fn view_metrics_ip(
    _is_admin_guard: IsAdminGuard,
    ip_address: String,
    state: &State<TYRState>,
) -> RawHtml<String> {
    if let Some(metric) = state.unique_users.read().unwrap().get(&ip_address) {
        let request_count = metric.request_count;
        let last_time_seen_seconds = match metric.last_time_seen {
            None => 0,
            Some(time) => SystemTime::now()
                .duration_since(time)
                .unwrap_or_default()
                .as_secs(),
        };
        let last_page_visited = match &metric.last_page_visited {
            None => "No last page known.",
            Some(last_page) => last_page,
        };
        let previous_pages: String = match &metric.previous_pages {
            None => "".to_string(),
            Some(pages) => {
                let mut list = String::new();
                list.push_str("<br>");
                pages
                    .get_list()
                    .iter()
                    .map(|non_escaped_uri| html_escape::encode_safe(&non_escaped_uri).to_string())
                    .for_each(|previous_request_uri| {
                        list.push_str(&format!("{} <br>", previous_request_uri))
                    });
                list
            }
        };

        RawHtml(
            html! {
                "Request count: " (request_count)
                br;
                br;
                "Last time seen: " (last_time_seen_seconds)
                br;
                br;
                "Last page visited: " (last_page_visited)
                br;
                br;
                "Previous requests: " (PreEscaped(previous_pages))
                br;
                br;
            }
            .into_string(),
        )
    } else {
        RawHtml(
            html! {
                p {"Metric not found."}
            }
            .into_string(),
        )
    }
}

#[get("/paste/view/<paste_id>/delete")]
/// Route for deleting a paste, this is forceful and requires administration rights.
/// The paste is either deleted, or if it does not exist, a 404 error is returned.
pub fn force_delete_paste(
    paste_id: u64,
    state: &State<TYRState>,
    _is_admin_guard: IsAdminGuard,
) -> Redirect {
    let mut lock = state.pastes.write().unwrap();
    return match lock.remove(&paste_id.to_string()) {
        None => Redirect::to(uri!("/paste_404")),
        Some(_paste) => Redirect::to(uri!("/admin")),
    };
}
