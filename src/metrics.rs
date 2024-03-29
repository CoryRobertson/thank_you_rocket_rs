use crate::common::PreviousRequestsList;
use crate::state_management::TYRState;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::tokio::spawn;
use rocket::{Data, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::SystemTime;

pub static PREVIOUS_REQUEST_LIST_CAP: usize = 50;

/// A struct handles metrics capturing, this struct is purely a function only implementation struct, and contains no data itself.
/// It takes a reference to the rockets managed state "TYRState" when it needs to modify data.
pub struct Metrics {}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// A struct that contains the data each user will carry as we track their metrics, at the moment
/// simply the number of requests they have sent to the server.
/// A user is unique to each ip, not device or computer.
pub struct UserMetric {
    pub request_count: u64,
    pub logins: Option<Vec<String>>,
    pub last_time_seen: Option<SystemTime>,
    pub last_page_visited: Option<String>,
    pub previous_pages: Option<PreviousRequestsList>,
}

/// Function that checks if the given ip address is banned
fn is_banned(ip: &String, banned_ips: &[String]) -> bool {
    banned_ips.contains(ip)
}

/// Save metrics in a pretty, formatted way, to a human readable file.
async fn save_metrics(metrics: HashMap<String, UserMetric>) {
    let file = File::create("./output/metrics.sav").unwrap();
    let mut buf = BufWriter::new(file);
    let req_count = metrics.iter().map(|user| user.1.request_count).sum::<u64>();
    let _ = buf
        .write(format!("Unique view count: {} \n", metrics.len()).as_bytes())
        .unwrap();
    let _ = buf
        .write(format!("Request count: {req_count} \n").as_bytes())
        .unwrap();

    for (socket_addr, user_metric) in metrics.iter() {
        let _ = buf
            .write(format!("{socket_addr}: \n").as_bytes())
            .expect("Unable to write to file buffer for metrics");
        let _ = buf
            .write(format!("\t{} \n", user_metric.request_count).as_bytes())
            .expect("Unable to write to file buffer for metrics");
    }

    buf.flush().expect("Unable to flush metrics file buffer.");
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
        let state = req.rocket().state::<TYRState>().unwrap();

        // println!("{:?}", req);

        let uri = req.uri();

        // println!("URI: {}", uri);

        match req.remote() {
            None => {
                // if somehow we don't get a remote url, direct them to an error page
                req.set_uri(Origin::try_from("/error_message").unwrap());
            }
            Some(ip) => {
                if is_banned(&ip.ip().to_string(), &state.banned_ips.read().unwrap()) {
                    // if the user has a valid ip, and is banned, direct them to an error page, and cease function activity.
                    req.set_uri(Origin::try_from("/error_message").unwrap());
                    return;
                } else {
                    // if the user is not banned, then we do metrics on them.
                    let mut lock = state.unique_users.write().unwrap();
                    match lock.get_mut(&ip.ip().to_string()) {
                        None => {
                            let mut prq = PreviousRequestsList::new(PREVIOUS_REQUEST_LIST_CAP);
                            prq.push(&uri.to_string());
                            lock.insert(
                                ip.ip().to_string(),
                                UserMetric {
                                    request_count: 1,
                                    logins: Some(vec![]),
                                    last_time_seen: Some(SystemTime::now()),
                                    last_page_visited: Some(uri.to_string()),
                                    previous_pages: Some(prq),
                                },
                            );
                        }
                        Some(metric) => {
                            // when ever we see a user, add one to their request count
                            metric.request_count += 1;
                            // when ever we see a user, update their last time seen.
                            metric.last_time_seen = Some(SystemTime::now());
                            metric.last_page_visited = Some(uri.to_string());

                            match &mut metric.previous_pages {
                                None => {
                                    let mut prq =
                                        PreviousRequestsList::new(PREVIOUS_REQUEST_LIST_CAP);
                                    prq.push(&uri.to_string());
                                    metric.previous_pages = Some(prq);
                                }
                                Some(prev) => {
                                    prev.push(&uri.to_string());
                                }
                            }
                        }
                    };
                    spawn(save_metrics(lock.clone()));
                }
            }
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut Response<'r>) {
        // unimplemented
    }
}
