#![allow(unused_variables)]
extern crate x11;
extern crate pwd;
extern crate libc;
extern crate crypto;

mod config;

use structs::{
    Lock,
    Xrandr,
    Constructor,
};
mod structs;

use keys::{
    Key,
    get_key_type,
};
mod keys;

use std::ptr;

use std::error::Error;

use std::fs::File;
use std::io::prelude::*;
use std::io;

use std::collections::HashMap;

use std::ffi::CString;

use std::cmp::Ordering;

use crypto::digest::Digest;
use crypto::md5::Md5;

use x11::xlib::*;
use x11::xrandr::*;
use x11::keysym::*;

use libc::{
    group,
    uid_t,
    gid_t,
    usleep,
};

#[derive(Copy, Clone, Debug)]
enum Color {
    INIT,
    INPUT,
    FAILED,
    NUMCOLS
}

fn readinput() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(msg) => panic!("Error reading input: {}", msg.description()),
    }
}

fn getpwfilepath() -> String {

    let username = config::getusername();

    let file_prefix = String::from("/home/");
    let file_suffix = String::from("/.rlock_pwd");
    let path = file_prefix + &username + &file_suffix;

    path
}

/** Function to hash password that the user entered .
 * to_hash: reference to string which needs to be hashed.
 * returns: hashed string using Md5 algortithm
 * */

fn hash(to_hash: &String) -> String {
    let mut digest = Md5::new();
    digest.input_str(&to_hash);
    digest.result_str()
}

fn createpwfile() -> String {
    /* Prompt user for password */
    print!("Enter password for screen lock: ");
    std::io::stdout().flush().unwrap();

    /* TODO: Input field should not display whatever is entered */
    let pwd = readinput();

    /* Prompt user to re-enter password */
    println!("Re-enter password to verify: ");
    std::io::stdout().flush().unwrap();

    let pwd_verify = readinput();

    /* Verify both hashes */

    if pwd != pwd_verify {
        println!("Passwords do not match!. Try again.");
        createpwfile();
    }

    /* Removing the newline character which was also included */
    let mut pwd_in_bytes = pwd.into_bytes();
    pwd_in_bytes.pop();

    let input_pwd = String::from_utf8(pwd_in_bytes).unwrap();

    let pwd_hash = hash(&input_pwd);
    println!("{}", pwd_hash);

    /* Write to the pwd file */
    let path = getpwfilepath();

    match File::create(path.clone()) {
        Ok(f) => {
            let mut file = f;
            match file.write_all(pwd_hash.as_bytes()){
                Ok(_) => {
                    println!("Successfully wrote to {}", path);
                    pwd_hash
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

pub fn getvalue(key: u32, map: HashMap<u32, String>) -> String {

    match map.get(&key) {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}

fn lockscreen(dpy: *mut Display, rr: Xrandr, screen: i32) -> (Lock, bool) {
    println!("Entered lockscreen fn");
    if dpy.is_null() || screen < 0 {
        return (Lock::new(), false);
    } 

    let mut screen_def_return = XColor::new();
    let mut exact_def_return = XColor::new();
    let mut ptgrab = -1;
    let mut kbgrab = -1;

    let mut lock = Lock::new();
    lock.screen = screen;

    let colors = config::readconfig();

    // println!("{:?}", getvalue(0, colors.clone()).as_ptr());

    unsafe {
        lock.root = XRootWindow(dpy, screen);
        for i in 0..Color::NUMCOLS as u32 {
            XAllocNamedColor(
                dpy,
                XDefaultColormap(dpy, screen),
                getvalue(i, colors.clone()).as_ptr() as *const i8,
                &mut screen_def_return,
                &mut exact_def_return);
            println!("Reached checkpoint 5");
            lock.colors.push(screen_def_return.pixel);
            println!("{:?}", lock);
        }
        /* init */

        let mut wa = XSetWindowAttributes::new();
        wa.override_redirect = 1;
        wa.background_pixel = lock.colors[0 /* init */];
        lock.win = XCreateWindow(
            dpy, /* Display */
            lock.root, /* Parent window */
            0, /* x */
            0, /* y */
            XDisplayWidth(dpy, lock.screen) as u32, /* width */
            XDisplayHeight(dpy, lock.screen) as u32, /* height */
            0, /* border_width */
            XDefaultDepth(dpy, lock.screen), /* depth */
            CopyFromParent as u32, /* Class */
            XDefaultVisual(dpy, lock.screen), /* Visual */
            CWOverrideRedirect | CWBackPixel, /* ValueMask */
            &mut wa /* XSetWindowAttributes */);

        let curs = vec![0; 8]; 

        lock.pmap = XCreateBitmapFromData(dpy, lock.win, curs.as_ptr(), 8, 8);

        let invisible = XCreatePixmapCursor(dpy, lock.pmap, lock.pmap, &mut screen_def_return, &mut screen_def_return, 0, 0);

        XDefineCursor(dpy, lock.win, invisible);

        /* Try to grab mouse and keyboard for 600ms */

        for i in 0..6 {
            if ptgrab != GrabSuccess {

                ptgrab = XGrabPointer(
                    dpy,
                    lock.root,
                    0,
                    ButtonPressMask as u32,
                    GrabModeAsync,
                    GrabModeAsync,
                    0,
                    invisible,
                    CurrentTime)
                
            }

            if kbgrab != GrabSuccess {

                kbgrab = XGrabKeyboard(
                    dpy,
                    lock.root,
                    1,
                    GrabModeAsync,
                    GrabModeAsync,
                    CurrentTime)

            }

            /* Input is grabbed, we can lock the screen */

            // if ptgrab == GrabSuccess && kbgrab == GrabSuccess {
            
                // XMapRaised(dpy, lock.win);
                if rr.active != 0 {
                    XRRSelectInput(dpy, lock.win, RRScreenChangeNotifyMask);
                }

                XSelectInput(dpy, lock.root, SubstructureNotifyMask);
                return (lock, true);
            // }

            /* Retry on AlreadyGrabbed, fail on other errors */
            if (ptgrab != AlreadyGrabbed && ptgrab != GrabSuccess) ||
                (kbgrab != AlreadyGrabbed && kbgrab != GrabSuccess) {
                break;
            }

            usleep(100000);
        }
    }

    /* couldn't grab all input: fail out */
    if ptgrab != GrabSuccess {
        println!("slock: unable to grab mouse pointer for screen: {}", screen);
    }

    if kbgrab != GrabSuccess {
        println!("slock: unable to grab keyboard for screen: {}", screen);
    }

    (Lock::new(), false)
}

fn readpw(dpy: *mut Display, rr: Xrandr, locks: Vec<Lock>, nscreens: i32, actual_hash: String) {
    let mut rre: XRRScreenChangeNotifyEvent;
    let mut running = true;
    let mut ev = XEvent::new();
    let mut num: i32 ; /* Number of characters entered currently */
    let mut passwd = Vec::new();
    let mut failure = false;

    unsafe {
        while running && (XNextEvent(dpy, &mut ev) == 0) {

            let mut ksym: KeySym = 0;

            if ev.get_type() == KeyPress {

                println!("Password {:?}", passwd);

                let buf_raw = CString::new("").unwrap().into_raw();
                num = XLookupString(&mut ev.key, buf_raw, 32, &mut ksym, ptr::null_mut() as *mut XComposeStatus);
                let buf = CString::from_raw(buf_raw);
                println!("Key pressed: buf: {:?}, num: {:?}, ksym: {}", buf, num, ksym);
                let key_type = get_key_type(ksym);

                /* If key typed is one of the extras ignore it */
                let exclude = vec![Key::FUNCTION, Key::KEYPAD, Key::MISCFUNCTION, Key::PF, Key::PRIVATEKEYPAD];
                match exclude.into_iter().find(|x| x.clone() == Key::from(ksym)) {
                    Some(_) => continue,
                    None => {},
                }

                println!("Key pressed: buf: {:?}, num: {:?}, ksym: {}", buf, num, ksym);

                match ksym as u32 {
                    XK_Return => {
                        /* User has finished typing the password */
                        let passwd_string = String::from_utf8(passwd).unwrap();

                        /* Hash password */
                        let input_hash = hash(&passwd_string);

                        /* Compare with actual hash (read from file) */
                        match input_hash.cmp(&actual_hash) {
                            Ordering::Equal => running = false,
                            _ => {
                                /* If wrong, set as failed */
                                XBell(dpy, 100);
                                failure = true;
                            }
                        }
                        passwd = Vec::new();
                        break;
                    },
                    XK_Escape => {
                        /* Clear password typed until now */
                        passwd = Vec::new();
                    },
                    XK_BackSpace => {
                        /* Remove last entered character */
                        passwd.pop();
                    },
                    _ => {
                        /* All other characters can be counted as a password character */
                        if num != 0 {
                            let buf_slice = buf.to_bytes();
                            passwd.extend_from_slice(buf_slice);
                        }
                    }

                }
            }
        }
    }
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

        println!("Reached checkpoint 3");
        /* XRRQueryExtension returns event and error base codes */
        let mut evbase: i32 = 0;
        let mut errbase: i32 = 0;
        rr.active = XRRQueryExtension(dpy, &mut evbase, &mut errbase);
        rr.evbase = evbase;
        rr.errbase = errbase;

        /* Get number of screens from dpy and blank them */
        nscreens = XScreenCount(dpy);
        let mut locks: Vec<Lock> = Vec::new();
        let mut nlocks = 0;

        for s in 0..nscreens {

            let (lock, success) = lockscreen(dpy, rr, s);
            println!("Reached checkpoint 4");
            if success {
                locks.push(lock);
                nlocks += 1;
            }
            else {
                break;
            }
        }

        XSync(dpy, 0);
        println!("Reached checkpoint 6");

        /* Check if all screens were locked */
        if nlocks != nscreens {
            panic!("Could not lock all screens");
        }

        /* run post-lock command */
        /* TODO: understand why slock code has fork */

        readpw(dpy, rr, locks, nscreens, hash);

        println!("Exited readpw");

    }
}
