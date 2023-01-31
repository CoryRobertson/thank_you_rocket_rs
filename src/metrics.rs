use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::{Data, Request};

/// A struct that can contain things we take metrics on, at the moment it only contains the list of banned ips, but will eventually keep track of how many people have view the page for example
/// Or even keeping a how many unique users have viewed the page.
pub struct Metrics {
    pub banned_ips: Vec<String>,
}

/// Function that checks if the given ip address is banned
fn is_banned(ip: &String, banned_ips: &Vec<String>) -> bool {
    banned_ips.contains(ip)
}

#[rocket::async_trait]
impl Fairing for Metrics {
    fn info(&self) -> Info {
        Info {
            name: "Metrics",
            kind: Kind::Request | Kind::Response,
        }
    }

    /// On request, we check the users ip, if it is banned, we change their uri to an error message.
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        match req.remote() {
            None => {
                req.set_uri(Origin::try_from("/error_message").unwrap());
            }
            Some(ip) => {
                if is_banned(&ip.ip().to_string(), &self.banned_ips) {
                    req.set_uri(Origin::try_from("/error_message").unwrap());
                }
            }
        }
    }
}
