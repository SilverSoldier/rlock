extern crate x11;
extern crate pwd;
extern crate libc;
extern crate crypto;

use std::ptr;
use std::process::exit;
use std::ffi::{ CStr, CString };
use std::error::Error;

use std::fs::File;
use std::io::prelude::*;
use std::io;

use crypto::digest::Digest;
use crypto::md5::Md5;

use x11::xlib::{
    Window,
    Pixmap,
    Display,
};

use libc::{
    strerror,
    c_char,
    group,
    uid_t,
    gid_t,
};

enum Color {
    INIT,
    INPUT,
    FAILED,
    NUMCOLS
}

struct Lock {
    screen: u32,
    root: Window,
    win: Window,
    pmap: Pixmap,
    colors: Vec<u64>,
}

struct Xrandr {
    active: u32,
    evbase: u32,
    errbase: u32,
}

fn readinput() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(msg) => panic!("Error reading input: {}", msg.description()),
    }
}

fn createpwfile() -> String {
    /* Prompt user for password */
    print!("Enter password for screen lock: ");
    std::io::stdout().flush().unwrap();

    /* TODO: Input field should not display whatever is entered */
    let pwd = readinput();

    /* Prompt user to re-enter password */
    print!("Re-enter password to verify: ");
    std::io::stdout().flush().unwrap();

    let pwd_verify = readinput();

    /* Verify both hashes */

    if pwd != pwd_verify {
        println!("Passwords do not match!. Try again.");
        createpwfile();
    }

    /* Hash password that the user entered */
    let mut digest = Md5::new();
    digest.input_str(&pwd);
    let hash = digest.result_str();
    // print!("Hash: {}", hash);

    /* Write to the pwd file */
    let username: String;
    unsafe{
        let name = libc::getenv(CString::new("USER").unwrap().as_ptr());
        username= CStr::from_ptr(name).to_string_lossy().into_owned();
    }

    let file_prefix = String::from("/home/");
    let file_suffix = String::from("/.rlock_pwd");
    let path = file_prefix + &username + &file_suffix;

    match File::create(path.clone()) {
        Ok(f) => {
            let mut file = f;
            match file.write_all(hash.as_bytes()){
                Ok(_) => {
                    println!("Successfully wrote to {}", path);
                    hash
                },
                Err(msg) => {
                    panic!("Error writing to file: {}",  msg);
                }
            }
            /* Return the hash to the getpw function */
        },
        Err(msg) => {
            panic!("Error creating file {}: {}", path, msg);
        }
    }
}

fn getpw() -> String {
    /* Read password from ~/.rlock_pwd file */

    let file: File;
    match File::open("~/.rlock_pwd") {
        Ok(f) => { 
            file = f;
        },
        /* Create file in case it does not exist */
        Err(_) => {
            println!("No existing password file! Creating file ... ");
            createpwfile();
        }
    };

    "Hello".to_string()
}

fn main() {
    let rr: Xrandr;
    let grp: *mut group;
    let duid: uid_t;
    let dgid: gid_t;
    let hash: String;
    let dpy: Display;
    let (s, nlocks, nscreens): (u32, u32, u32);

    /* Omitting code from original slock which parses arguments to give version info */

    hash = getpw();
}
