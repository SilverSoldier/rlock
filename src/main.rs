#![allow(unused_variables)]
extern crate x11;
extern crate pwd;
extern crate libc;
extern crate crypto;

mod config;

use std::ptr;
// use std::process::exit;
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
    XOpenDisplay,
    XScreenCount,
    XSync,
    XAllocNamedColor,
    XDefaultColormap,
    XRootWindow,
    XColor,
};

use x11::xrandr::{
    XRRQueryExtension,
};

use libc::{
    // strerror,
    // c_char,
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
    screen: i32,
    root: Window,
    win: Window,
    pmap: Pixmap,
    colors: Vec<u64>,
}

impl Lock {
    pub fn new() -> Lock {
        Lock { 
            screen: 0,
            root: 0,
            win: 0,
            pmap: 0,
            colors: Vec::new()
        }
    }
}

#[derive(Copy, Clone)]
struct Xrandr {
    active: i32,
    evbase: *mut i32,
    errbase: *mut i32,
}

impl Xrandr {
    pub fn new() -> Xrandr {
        Xrandr { active: 0, evbase: ptr::null_mut(), errbase: ptr::null_mut() }
    }
}

fn readinput() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(msg) => panic!("Error reading input: {}", msg.description()),
    }
}

fn getpwfilepath() -> String {
    let username: String;
    unsafe{
        let name = libc::getenv(CString::new("USER").unwrap().as_ptr());
        username= CStr::from_ptr(name).to_string_lossy().into_owned();
    }

    let file_prefix = String::from("/home/");
    let file_suffix = String::from("/.rlock_pwd");
    let path = file_prefix + &username + &file_suffix;

    path
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
    print!("Hash: {}", hash);

    /* Write to the pwd file */
    let path = getpwfilepath();

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
    /* Read password from /home/USER/.rlock_pwd file */

    match File::open(getpwfilepath()) {
        Ok(f) => { 
            let mut file = f;
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    println!("{}", contents);
                    contents
                },
                Err(msg) => {
                    panic!("Cannot read contents of file. {}", msg);
                }
            }
        },
        /* Create file in case it does not exist */
        Err(_) => {
            println!("No existing password file! Creating file ... ");
            createpwfile()
        }
    }
}

fn lockscreen(dpy: *mut Display, rr: Xrandr, screen: i32) -> (Lock, bool) {
    if dpy.is_null() || screen < 0 {
        return (Lock::new(), false);
    } 

    let screen_def_return: *mut XColor = ptr::null_mut();
    let exact_def_return: *mut XColor = ptr::null_mut();

    let mut lock: Lock;
    lock.screen = screen;
    unsafe {
        lock.root = XRootWindow(dpy, screen);
        for i in 0..Color::NUMCOLS as u32 {
            XAllocNamedColor(
                dpy,
                XDefaultColormap(dpy, screen),
                config::getcolors(i).as_ptr() as *const i8,
                screen_def_return,
                exact_def_return);
        }
    }
    return (Lock::new(), false);
}

fn main() {
    let mut rr = Xrandr::new();
    let grp: *mut group;
    let duid: uid_t;
    let dgid: gid_t;
    let hash: String;
    let dpy: *mut Display;
    let nscreens: i32;

    /* TODO: add command line arguments functionality to change password */

    hash = getpw();

    unsafe {
        dpy = XOpenDisplay(ptr::null());

        /* XRRQueryExtension returns event and error base codes */
        let evbase: *mut i32 = ptr::null_mut();
        let errbase: *mut i32  = ptr::null_mut();
        rr.active = XRRQueryExtension(dpy, evbase, errbase);
        rr.evbase = evbase;
        rr.errbase = errbase;

        /* Get number of screens from dpy and blank them */
        nscreens = XScreenCount(dpy);
        let mut locks: Vec<Lock> = Vec::new();
        let mut nlocks = 0;

        for s in 0..nscreens {
            let (lock, success) = lockscreen(dpy, rr, s);
            if success {
                locks.push(lock);
                nlocks += 1;
            }
            else {
                break;
            }
        }

        XSync(dpy, 0);

        /* Check if all screens were locked */
        if nlocks != nscreens {
            panic!("Could not lock all screens");
        }
    }
}
