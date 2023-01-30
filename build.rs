use std::fs;
use std::process::Command;

fn main() {

    println!("cargo:rerun-if-changed=rhythm_rs");

    let static_dir = match fs::read_dir("./static") {
        Ok(dir) => { dir }
        Err(_) => {
            match fs::create_dir("./static") {
                Ok(cdir) => { cdir }
                Err(err) => {
                    panic!("Unable to read or create \"./static\" directory relative to program. \n{}", err);
                }
            }
        }
    };

    let rhythm_rs_dir = match fs::read_dir("./rhythm_rs") {
        Ok(dir) => {
            dir
            // rhythm directory exists, could probably do further checking for if it needs to be built
        }
        Err(_) => {
            Command::new("git clone https://github.com/CoryRobertson/rhythm_rs.git")
        }
    }



}

