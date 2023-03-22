#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

use crate::metrics::Metrics;
use crate::pages::admin::*;
use crate::pages::error_catch_pages::not_found;
use crate::pages::index::index;
use crate::pages::login::*;
use crate::pages::new::new;
use crate::pages::outcome_pages::*;
use crate::pages::submit_message::submit_message;
use crate::pages::view::view;
use crate::state_management::*;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use std::path::PathBuf;

mod common;
mod message;
mod metrics;
mod pages;
mod state_management;
mod user;
mod verified_guard;
mod paste;

//TODO: make a module similar to pastebin,  should be allow pasting using a web text box through a post request, something like "/paste/new/<paste_form_struct>"
//  also needs a route to read a paste, pastes would be stored in a hashmap where a uuid of some kind is generated with a reference to the paste, and something like
//  "/paste/view/<uuid>" would view them for the user. Make sure to escape text in some way, either by telling browser it should be text only, or by actually escaping the text.
// these pastes should have a date and time they were created, as well as the ip and or the login of the user who created them.

/// The duration in seconds that a user must wait between each message. debug only
#[cfg(debug_assertions)]
pub static POST_COOLDOWN: u64 = 5;

/// The duration in seconds that a user must wait between each message. release only
#[cfg(not(debug_assertions))]
pub static POST_COOLDOWN: u64 = 3600;

/// The duration in seconds that a user is considered "online" from their last time they have been seen on the website.
/// Used to calculate the number of online users.
pub static ONLINE_TIMER: u64 = 600;

/// The maximum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_CAP: usize = 150;

/// The minimum length of a message that can be left by a user.
pub static MESSAGE_LENGTH_MIN: usize = 3;

/// File name for saving the state to the system.
pub static SERDE_FILE_NAME: &str = "state.ser";

/// Rendered version of messages, in a pretty file.
pub static RENDER_FILE_NAME: &str = "messages.sav";

/// Version number for the cargo package version.
pub static VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[launch]
fn rocket() -> Rocket<Build> {
    // using this return type isn't shown in the documentation from my minimal looking, but makes intellij happy.

    let load = load_state_save(&PathBuf::from(format!("./output/{SERDE_FILE_NAME}")));

    println!("Loaded message data: {:?}", load.messages);

    let state = TYRState::from_state_save(load);

    let metrics_fairing: Metrics = Metrics {};

    #[cfg(debug_assertions)]
    println!("Salt: {}", pages::login::SALT.as_str());

    println!("Admin state: {:?}", state.admin_state.read().unwrap());

    println!("Loaded banned ips: {:?}", state.banned_ips.read().unwrap());

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
                login,
                login_post,
                logout,
                admin,
                admin_metrics,
                ban_ip,
                view_cooldown,
                view_hashes,
                view_online,
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
        .attach(AdHoc::on_shutdown("State shutdown save", |rocket| {
            Box::pin(async move {
                println!("Saving state to file system.");
                let state_ref = rocket.state::<TYRState>().unwrap();
                save_program_state(state_ref.into(), &PathBuf::from("./output/state.ser"));
            })
        }))
}
