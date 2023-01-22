#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use rocket::{State};

struct Messages {
    messages: Arc<
        Mutex<
            HashMap<String,Vec<String>> // hash map consists of the ip address of the user: a vector of the messages they have left
        >
    >,
}



impl Default for Messages {
    fn default() -> Self {
        Messages{ messages: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[get("/")]
fn index(req: SocketAddr, messages: &State<Messages>) -> &'static str {
    let mut lock = messages.messages.lock().unwrap();
    let user_ip = &req.ip().to_string();
    match lock.get_mut(user_ip) {
        None => {
            let mut new_vec = vec![]; // create a new vector and add it to this users ip address
            new_vec.push("____".to_string()); // eventually push the message they sent, not just underscores
            lock.insert(user_ip.to_string(), new_vec); // insert the new vector with the key of the users ip address
        }
        Some(msg_vec) => {
            msg_vec.push("____".to_string());
        }
    }
    for (ip,msg_vec) in lock.iter() {
        println!("{}: {:?}", ip, msg_vec);
    }
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {

    rocket::build()
        .manage(Messages::default())
        .mount("/", routes![index])
}
