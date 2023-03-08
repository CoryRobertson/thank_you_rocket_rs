use crate::metrics::UserMetric;
use crate::user::User;
use chrono::{DateTime, Datelike, Local, Timelike};
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
/// A serializable version of the TYRState struct, used only for saving.
/// Content in this state are persisted between launches.
pub struct StateSave {
    pub messages: HashMap<String, User>,
    pub banned_ips: Option<Vec<String>>,
    pub admin_state: Option<AdminState>,
    pub unique_users: Option<HashMap<String, UserMetric>>,
}

/// The state struct for the rocket web frame work.
/// Content in this struct but not in StateSave are not persisted.
#[derive(Debug, Clone)]
pub struct TYRState {
    // hash map consists of the ip address as a key, and the user struct itself.
    pub messages: Arc<RwLock<HashMap<String, User>>>,
    pub banned_ips: Arc<RwLock<Vec<String>>>, // vector full of all of the banned ips read from file at startup
    pub admin_state: Arc<RwLock<AdminState>>,
    pub unique_users: Arc<RwLock<HashMap<String, UserMetric>>>,
}

impl TYRState {
    /// Reads a StateSave object, producing a TYRState object.
    pub fn from_state_save(state_save: StateSave) -> Self {
        Self {
            messages: Arc::new(RwLock::new(state_save.messages)),
            banned_ips: Arc::new(RwLock::new(state_save.banned_ips.unwrap_or_default())),
            admin_state: Arc::new(RwLock::new(state_save.admin_state.unwrap_or_default())),
            unique_users: Arc::new(RwLock::new(state_save.unique_users.unwrap_or_default())),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
/// A struct that stores if an admin has been created, and a vector of hashes of passwords that an admin can use to login.
pub struct AdminState {
    pub admin_created: bool,
    pub admin_hashes: Vec<String>,
}

impl Default for TYRState {
    /// Default message struct is just an empty hash map.
    fn default() -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            banned_ips: Arc::new(RwLock::new(Default::default())),
            admin_state: Arc::from(RwLock::from(AdminState::default())),
            unique_users: Arc::new(Default::default()),
        }
    }
}

/// Loads all messages from the system, outputs a new state if no messages were found.
pub fn load_state_save(path: &PathBuf) -> StateSave {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(err) => {
            println!("Didnt find serde file name. {err}");
            return StateSave {
                messages: HashMap::new(),
                banned_ips: None,
                admin_state: None,
                unique_users: None,
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
                banned_ips: None,
                admin_state: None,
                unique_users: None,
            };
        }
    }

    match serde_json::from_str::<StateSave>(&s) {
        Ok(state) => state,
        Err(err) => {
            println!("ERROR UNABLE TO READ STATE SAVE FROM \"messages.ser\", using default message list \n {err}");
            StateSave {
                messages: HashMap::new(),
                banned_ips: None,
                admin_state: None,
                unique_users: None,
            }
        }
    }
}

/// Saves all messages to the system in a file.
pub fn save_program_state(messages: &State<TYRState>, path: &PathBuf) {
    match fs::read_dir(path.parent().unwrap()) {
        Ok(_) => {
            // output dir exists
        }
        Err(_) => {
            match fs::create_dir(path.parent().unwrap()) {
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
            messages: messages.messages.read().unwrap().clone(),
            banned_ips: Some(messages.banned_ips.read().unwrap().clone()),
            admin_state: Some(messages.admin_state.read().unwrap().clone()),
            unique_users: Some(messages.unique_users.read().unwrap().clone()),
        };

        let ser = serde_json::to_string(&state_save).unwrap();

        let mut ser_file = File::create(path).unwrap();

        ser_file.write_all(ser.as_ref()).unwrap();
    }

    let file_name = { format!("{}/messages.sav", path.parent().unwrap().to_str().unwrap()) };

    // block for rendering out the user data into a pretty file for the host :)
    let file = File::create(file_name).unwrap();
    let mut bw = BufWriter::new(file);
    for (ip, user) in messages.messages.read().unwrap().iter() {
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

#[cfg(test)]
mod test {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_state_management() {
        let state = TYRState {
            messages: Arc::new(Default::default()),
            banned_ips: Arc::new(Default::default()),
            admin_state: Arc::new(Default::default()),
            unique_users: Arc::new(Default::default()),
        };
        state.admin_state.write().unwrap().admin_created = true;
        state
            .admin_state
            .write()
            .unwrap()
            .admin_hashes
            .push("lmao not a real hash".to_string());
        state.unique_users.write().unwrap().insert(
            "this ip".to_string(),
            UserMetric {
                request_count: 44,
                logins: None,
                last_time_seen: None,
            },
        );
        state.unique_users.write().unwrap().insert(
            "this ip2".to_string(),
            UserMetric {
                request_count: 55,
                logins: None,
                last_time_seen: None,
            },
        );
        state
            .banned_ips
            .write()
            .unwrap()
            .push("1.2.3.4".to_string());
        state
            .banned_ips
            .write()
            .unwrap()
            .push("5.6.7.8".to_string());
        state.messages.write().unwrap().insert(
            "4.1.2.3".to_string(),
            User {
                messages: vec![],
                last_time_post: SystemTime::now(),
            },
        );
        state
            .messages
            .write()
            .unwrap()
            .get_mut("4.1.2.3")
            .unwrap()
            .push("lmao".to_string(), None);
        let rocket = rocket::build().manage(state.clone());
        save_program_state(
            State::get(&rocket).unwrap(),
            &PathBuf::from("./test/test_state.ser"),
        );

        let loaded_state =
            TYRState::from_state_save(load_state_save(&PathBuf::from("./test/test_state.ser")));

        assert_eq!(
            state.admin_state.read().unwrap().clone(),
            loaded_state.admin_state.read().unwrap().clone()
        );
        assert_eq!(
            state.unique_users.read().unwrap().clone(),
            loaded_state.unique_users.read().unwrap().clone()
        );

        // check the text of each message, this is because system times when serialized get rounded partially.
        for (ip, _) in state.messages.read().unwrap().iter() {
            assert_eq!(
                state
                    .messages
                    .read()
                    .unwrap()
                    .get(ip)
                    .unwrap()
                    .messages
                    .iter()
                    .map(|msg| msg.text.to_string())
                    .collect::<Vec<String>>(),
                loaded_state
                    .messages
                    .read()
                    .unwrap()
                    .get(ip)
                    .unwrap()
                    .messages
                    .iter()
                    .map(|msg| msg.text.to_string())
                    .collect::<Vec<String>>()
            );
        }
        assert_eq!(
            state.banned_ips.read().unwrap().clone(),
            loaded_state.banned_ips.read().unwrap().clone()
        );

        fs::remove_dir_all(PathBuf::from("./test")).unwrap();
    }
}
