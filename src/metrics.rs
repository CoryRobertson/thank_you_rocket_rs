use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::tokio::spawn;
use rocket::{Data, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, RwLock};

/// A struct that can contain things we take metrics on, at the moment it only contains the list of banned ips, but will eventually keep track of how many people have view the page for example
/// Or even keeping a how many unique users have viewed the page.
pub struct Metrics {
    pub banned_ips: Arc<RwLock<Vec<String>>>,
    pub unique_users: Arc<RwLock<HashMap<String, UserMetric>>>,
}



#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// A struct that contains the data each user will carry as we track their metrics, at the moment
/// simply the number of requests they have sent to the server.
pub struct UserMetric {
    pub request_count: u64,
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

    // On ignite, we read the serialized users and input them into this metrics object.
    // async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
    //     self.deserialize_metrics(&PathBuf::from("./output/metrics.ser"));
    //     Ok(rocket)
    // }

    /// On request, we check the users ip, if it is banned, we change their uri to an error message.
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        match req.remote() {
            None => {
                // if somehow we don't get a remote url, direct them to an error page
                req.set_uri(Origin::try_from("/error_message").unwrap());
            }
            Some(ip) => {
                if is_banned(&ip.ip().to_string(), &self.banned_ips.read().unwrap()) {
                    // if the user has a valid ip, and is banned, direct them to an error page, and cease function activity.
                    req.set_uri(Origin::try_from("/error_message").unwrap());
                    return;
                } else {
                    // if the user is not banned, then we do metrics on them.
                    let mut lock = self.unique_users.write().unwrap();
                    match lock.get_mut(&ip.ip().to_string()) {
                        None => {
                            lock.insert(ip.ip().to_string(), UserMetric { request_count: 1 });
                        }
                        Some(metric) => {
                            metric.request_count += 1;
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

    // On shutdown we save the unique users to file, and save their metrics in a pretty format to a file as well.
    // async fn on_shutdown(&self, _rocket: &Rocket<Orbit>) {
    //     self.serialize_users(&PathBuf::from("./output/metrics.ser"));
    //     let future = save_metrics(self.unique_users.read().unwrap().clone());
    //     future.await;
    // }
}
