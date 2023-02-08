#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

use crate::message::TYRState;
use crate::metrics::Metrics;
use crate::pages::error_catch_pages::not_found;
use crate::pages::index::index;
use crate::pages::new::new;
use crate::pages::outcome_pages::*;
use crate::pages::submit_message::submit_message;
use crate::pages::view::view;
use crate::state_management::load_messages;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

mod message;
mod metrics;
mod pages;
mod state_management;
mod user;

/// The duration in seconds that a user must wait between each message. debug only
#[cfg(debug_assertions)]
pub static POST_COOLDOWN: u64 = 5;

/// The duration in seconds that a user must wait between each message. release only
#[cfg(not(debug_assertions))]
pub static POST_COOLDOWN: u64 = 3600;

/// The maximum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_CAP: usize = 150;

/// The minimum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_MIN: usize = 3;

pub static SERDE_FILE_NAME: &str = "messages.ser";
pub static RENDER_FILE_NAME: &str = "messages.sav";

pub static VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

// TODO: implement a random uuid as a password, generated at runtime, navigating to this page displays all messages sent and stored in the state.
//  password would use the uuid crate, and have a page that is of low route priority, and takes in any string, validates the priority, then displays the content, if not, displays the 404 error
//  this will use a request guard!
//  this will also use a specific page that stores the key as a cookie?

#[launch]
fn rocket() -> Rocket<Build> {
    // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.

    let load = load_messages();
    println!("Loaded message data: {:?}", load.messages);
    let state = TYRState {
        messages: Arc::new(RwLock::new(load.messages)),
        banned_ips: {
            if let Ok(file) = File::open("./banned_ips.txt") {
                let br = BufReader::new(file);
                let lines: Vec<String> = br
                    .lines() // parse the lines out in a super fun way, probably unnecessary as a banned_ips file is unlikely to be full of errors, but this was super fun to make.
                    .filter_map(|line| {
                        // filter out only valid lines
                        match line {
                            Ok(l) => Some(l),
                            Err(_) => None,
                        }
                    })
                    .filter(|line| {
                        // check that there are 4 valid u8 numbers in the ip address
                        // 1.2.3.4 1111.2222.3333.4444
                        // possible inputs, only the left one should be considered possibly valid.
                        let num_len_valid: Vec<&str> = line
                            .split('.') // split the line given by its periods
                            .filter(|num_split| {
                                // only keep lines that are possible to be parsed into a 8u
                                num_split.parse::<u8>().is_ok()
                            })
                            .collect();
                        num_len_valid.len() == 4 // there needs to be exactly 4 valid u8 numbers to allow this given line to be kept.
                    })
                    .collect();
                lines
            } else {
                vec![]
            }
        },
        admin_uuid_page: Uuid::new_v4(),
    };

    let metrics_fairing: Metrics = Metrics {
        banned_ips: state.banned_ips.clone(),
        unique_users: Arc::new(Mutex::new(Default::default())),
        // TODO: if metrics are needed on any pages, clone the arc that is here into the state before we build the rocket.
    };

    println!("Loaded banned ips: {:?}", state.banned_ips);

    rocket::build()
        .manage(state)
        .mount(
            "/",
            routes![
                index,
                submit_message,
                new,
                submit_message_no_data,
                view,
                slow_down,
                too_long,
                too_short,
                duplicate,
                error_message,
            ],
        )
        .register("/", catchers![not_found])
        .mount("/static", FileServer::from("./static"))
        .mount("/rhythm_rs", FileServer::from("./rhythm_rs_dist")) // program crashes if static folder does not exist.
        .mount(
            "/discreet_math_fib",
            FileServer::from("./discreet_math_fib_dist"),
        ) // program crashes if static folder does not exist.
        .attach(metrics_fairing)
}
