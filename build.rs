use std::fs;
use std::process::Command;

fn main() {

    // let a = cfg!(feature = "rhythm");
    // panic!("a: {}", a);
    // cargo build --features="rhythm" OR cargo build

    // TODO: use features to determine if to build with this script or to simply build just the web server.

    // println!("cargo:rerun-if-changed=rhythm_rs");
    //
    // let static_dir = match fs::read_dir("./static") {
    //     Ok(dir) => { dir }
    //     Err(_) => {
    //         match fs::create_dir("./static") {
    //             Ok(cdir) => { cdir }
    //             Err(err) => {
    //                 panic!("Unable to read or create \"./static\" directory relative to program. \n{}", err);
    //             }
    //         }
    //     }
    // };
    //
    // match fs::read_dir("./rhythm_rs") {
    //     Ok(_) => {
    //         // rhythm directory exists, could probably do further checking for if it needs to be built
    //     }
    //     Err(_) => {
    //         Command::new("git")
    //             .args(["clone", "https://github.com/CoryRobertson/rhythm_rs.git"])
    //             .current_dir("./")
    //             .status()
    //             .expect("git clone rhythm_rs failed unexpectedly. Is git installed?");
    //
    //         Command::new("trunk")
    //             .args(["build", "--release"])
    //             .current_dir("./rhythm_rs/")
    //             .status()
    //             .expect("trunk build failed unexpectedly. Is trunk installed as well as cargo?")
    //
    //     }
    // };
}
