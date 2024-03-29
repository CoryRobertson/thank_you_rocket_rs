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
use crate::pages::post_paste::*;
use crate::pages::submit_message::submit_message;
use crate::pages::view::view;
use crate::state_management::*;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use std::fs;
use std::path::PathBuf;

// TODO: implement the usage of smol db ?

mod common;
mod message;
mod metrics;
mod pages;
mod paste;
mod state_management;
mod user;
mod verified_guard;

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

/// The maximum length of a paste that can be left by a user.
pub static PASTE_LENGTH_CAP: usize = 2000;

/// The minimum length of a paste that can be left by a user.
pub static PASTE_LENGTH_MIN: usize = 10;

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

    fs::create_dir_all("./output/file_uploads/").unwrap();

    #[cfg(debug_assertions)]
    println!("Salt: {}", pages::login::SALT.as_str());

    println!("Admin state: {:?}", state.admin_state.read().unwrap());

    println!("Loaded banned ips: {:?}", state.banned_ips.read().unwrap());

    println!("Pastes: {:?}", state.pastes.read().unwrap());

    // TODO: make the program periodically save its state even if its not shutting down, most likely through a second thread that carries a reference to the state.

    // TODO: make the same thread that saves program state periodically also clean up old pastes, maybe of age > 30 days?

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
                error_message_specific,
                login,
                login_post,
                logout,
                admin,
                admin_metrics,
                ban_ip,
                view_cooldown,
                view_hashes,
                view_online,
                new_paste,
                new_paste_post,
                view_paste,
                paste_404,
                force_delete_paste,
                view_pastes_admin,
                upload,
                download_file_paste,
                upload_multipart,
                view_metrics_ip,
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
