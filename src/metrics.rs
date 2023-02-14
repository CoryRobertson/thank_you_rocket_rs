use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::tokio::spawn;
use rocket::{Build, Data, Orbit, Request, Response, Rocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A struct that can contain things we take metrics on, at the moment it only contains the list of banned ips, but will eventually keep track of how many people have view the page for example
/// Or even keeping a how many unique users have viewed the page.
pub struct Metrics {
    pub banned_ips: Vec<String>,

    pub unique_users: Arc<Mutex<HashMap<String, UserMetric>>>,
}

#[derive(Clone, Serialize, Deserialize)]
/// A serializable version of metrics, for the purpose of saving the metrics to a file.
struct SerializableMetrics {
    request_count: u64,
    users: HashMap<String, UserMetric>,
}

impl Metrics {
    /// Serialized users to a file, saving it to ./output/metrics.ser
    fn serialize_users(&self, path: &PathBuf) -> File {
        let users = self.unique_users.lock().unwrap().clone();
        let users: SerializableMetrics = SerializableMetrics {
            request_count: users.iter().map(|user| user.1.request_count).sum(),
            users,
        };
        let ser = serde_json::to_string(&users).unwrap();
        let mut file = File::create(path).unwrap();
        let _ = file
            .write(ser.as_bytes())
            .expect("Unable to write metrics to serializable file");
        file
    }

    /// Deserializes metrics from a file, and modifies the unique users hashmap in metrics
    fn deserialize_metrics(&self, path: &PathBuf) {
        // only run function if metrics serialization file exists
        if let Ok(mut file) = File::open(path) {
            let mut file_content = String::new();
            // only continue running function if we can successfully read the file content into a string.
            if file.read_to_string(&mut file_content).is_ok() {
                // only continue if we can read the file into a string, and successfully deserialize the file content.
                if let Ok(unique_users) = serde_json::from_str::<SerializableMetrics>(&file_content)
                {
                    let mut lock = self.unique_users.lock().unwrap();
                    for (ip, user) in unique_users.users {
                        lock.insert(ip, user);
                    }
                    // self.request_count.store(unique_users.request_count,Ordering::Relaxed);
                } else {
                    // unable to deserialize metrics file
                    println!(
                        "Unable to deserialize metrics file, a new metrics file will be created."
                    );
                }
            } else {
                // unable to read file to string
                println!(
                    "Unable to read metrics file to string, a new metrics file will be created."
                );
            }
        } else {
            // metrics file does not exist
            println!("Metrics file does not exist, a new one will be created.");
        }
    }
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
            kind: Kind::Request | Kind::Response | Kind::Ignite | Kind::Shutdown,
        }
    }

    /// On ignite, we read the serialized users and input them into this metrics object.
    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        self.deserialize_metrics(&PathBuf::from("./output/metrics.ser"));
        Ok(rocket)
    }

    /// On request, we check the users ip, if it is banned, we change their uri to an error message.
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        match req.remote() {
            None => {
                // if somehow we don't get a remote url, direct them to an error page
                req.set_uri(Origin::try_from("/error_message").unwrap());
            }
            Some(ip) => {
                if is_banned(&ip.ip().to_string(), &self.banned_ips) {
                    // if the user has a valid ip, and is banned, direct them to an error page, and cease function activity.
                    req.set_uri(Origin::try_from("/error_message").unwrap());
                    return;
                } else {
                    // if the user is not banned, then we do metrics on them.
                    let mut lock = self.unique_users.lock().unwrap();
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

    /// On shutdown we save the unique users to file, and save their metrics in a pretty format to a file as well.
    async fn on_shutdown(&self, _rocket: &Rocket<Orbit>) {
        self.serialize_users(&PathBuf::from("./output/metrics.ser"));
        let future = save_metrics(self.unique_users.lock().unwrap().clone());
        future.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    /// At the moment, the only assertion is that a metrics serialized struct will contain the same users
    fn test_save_load_metrics() {
        let metrics = Metrics {
            // none of these matter, as the metrics serialized struct does not load this, instead it loads it through startup.
            banned_ips: vec!["1.23.45.67".to_string(), "98.67.54.32".to_string()],

            unique_users: Arc::new(Mutex::new(Default::default())),
        };
        {
            let mut lock = metrics.unique_users.lock().unwrap();

            lock.insert("44.55.66.77".to_string(), UserMetric { request_count: 3 });
            lock.insert("67.162.11.4".to_string(), UserMetric { request_count: 400 });
            lock.insert("33.44.55.66".to_string(), UserMetric { request_count: 126 });
        }

        let _ = metrics.serialize_users(&PathBuf::from("test_metrics_file.ser"));

        let deser = Metrics {
            banned_ips: vec![],
            unique_users: Arc::new(Mutex::new(Default::default())),
        };

        deser.deserialize_metrics(&PathBuf::from("test_metrics_file.ser"));

        assert_eq!(
            *metrics.unique_users.lock().unwrap(),
            *deser.unique_users.lock().unwrap()
        );

        fs::remove_file("test_metrics_file.ser").expect("Unable to delete test metrics file");
    }
}
