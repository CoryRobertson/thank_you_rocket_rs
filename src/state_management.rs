use crate::user::User;
use crate::{RENDER_FILE_NAME, SERDE_FILE_NAME};
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
/// A serializable version of the TYRState struct, used only for saving.
pub struct StateSave {
    pub(crate) messages: HashMap<String, User>,
}

/// The state struct for the rocket web frame work.
pub struct TYRState {
    // hash map consists of the ip address as a key, and the user struct itself.
    pub messages: Arc<RwLock<HashMap<String, User>>>,
    pub banned_ips: Vec<String>, // vector full of all of the banned ips read from file at startup
    pub admin_state: Arc<RwLock<AdminState>>,
}

#[derive(Default)]
pub struct AdminState {
    pub admin_created: bool,
    pub admin_hashes: Vec<String>,
}

impl Default for TYRState {
    /// Default message struct is just an empty hash map.
    fn default() -> Self {
        TYRState {
            messages: Arc::new(RwLock::new(HashMap::new())),
            banned_ips: vec![],
            admin_state: Arc::from(RwLock::from(AdminState::default())),
        }
    }
}

/// Loads all messages from the system, outputs a new state if no messages were found.
pub fn load_messages() -> StateSave {
    let file_name = format!("./output/{SERDE_FILE_NAME}");
    let mut file = match File::open(file_name) {
        Ok(f) => f,
        Err(err) => {
            println!("Didnt find serde file name. {err}");
            return StateSave {
                messages: HashMap::new(),
            };
        }
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Ok(_) => {}
        Err(err) => {
            println!("Unable to read to string from save file. {err}");
            return StateSave {
                messages: HashMap::new(),
            };
        }
    }

    match serde_json::from_str::<StateSave>(&s) {
        Ok(state) => state,
        Err(err) => {
            println!("ERROR UNABLE TO READ STATE SAVE FROM \"messages.ser\", using default message list \n {err}");
            StateSave {
                messages: HashMap::new(),
            }
        }
    }
}

/// Saves all messages to the system in a file.
pub fn save_messages(messages: HashMap<String, User>) {
    if !messages.is_empty() {
        match fs::read_dir("./output") {
            Ok(_) => {
                // output dir exists
            }
            Err(_) => {
                match fs::create_dir("./output") {
                    Ok(_) => {
                        // output dir now exists
                    }
                    Err(err) => {
                        panic!("{err} \n unable to create output dir, check file permissions?")
                    }
                }
            }
        }

        {
            // block of code to save the serializable state of the program, useful for allowing users to never lose their messages.
            let state_save = StateSave {
                messages: messages.clone(),
            };

            let ser = serde_json::to_string(&state_save).unwrap();

            let ser_file_name = format!("./output/{SERDE_FILE_NAME}");

            let mut ser_file = File::create(ser_file_name).unwrap();

            ser_file.write_all(ser.as_ref()).unwrap();
        }

        let file_name = { format!("./output/{RENDER_FILE_NAME}") };

        // block for rendering out the user data into a pretty file for the host :)
        let file = File::create(file_name).unwrap();
        let mut bw = BufWriter::new(file);
        for (ip, user) in messages.iter() {
            let messages = &user.messages;
            let _ = bw.write(format!("{ip}:\n").as_bytes()).unwrap();
            for msg in messages {
                let date: DateTime<Local> = DateTime::from(msg.time_stamp);
                let am_pm = match date.hour12().0 {
                    true => "PM",
                    false => "AM",
                };
                let time_format = format!(
                    "{}:{:02}:{:02}{}",
                    date.hour12().1,
                    date.minute(),
                    date.second(),
                    am_pm
                );
                let time_stamp_text = format!(
                    "{}-{}-{}: {}",
                    date.year(),
                    date.month(),
                    date.day(),
                    time_format,
                );
                let _ = bw
                    .write(format!("\t[ {} ]: {}\n", time_stamp_text, msg.text).as_bytes())
                    .unwrap();
            }
        }
        let _ = bw.flush();
    }
}
