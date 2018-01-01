extern crate x11;
extern crate pwd;
extern crate libc;
extern crate crypto;
extern crate docopt;

mod config;

use structs::{
    Lock,
    Xrandr,
    Constructor,
};
mod structs;

use keys::{
    get_key_type,
};
mod keys;

use arg::USAGE;
mod arg;

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
    usleep,
};

use docopt::Docopt;

#[derive(Copy, Clone, Debug)]
enum Color {
    INIT,
    INPUT,
    FAILED,
    NUMCOLS
}

fn read_input() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(msg) => panic!("Error reading input: {}", msg.description()),
    }
}

fn get_pwfile_path() -> String {

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

fn create_pwfile() -> String {
    /* Prompt user for password */
    print!("Enter password for screen lock: ");
    std::io::stdout().flush().unwrap();

    /* TODO: Input field should not display whatever is entered */
    let pwd = read_input();

    /* Prompt user to re-enter password */
    print!("Re-enter password to verify: ");
    std::io::stdout().flush().unwrap();

    let pwd_verify = read_input();

    /* Verify both passwords */
    if pwd != pwd_verify {
        println!("Passwords do not match!. Try again.");
        create_pwfile();
    }

    /* Removing the newline character which is also included */
    let mut pwd_in_bytes = pwd.into_bytes();
    pwd_in_bytes.pop();
    let input_pwd = String::from_utf8(pwd_in_bytes).unwrap();

    /* Hash password */
    let pwd_hash = hash(&input_pwd);

    /* Write to the pw file */
    let path = get_pwfile_path();

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

/** Function to read password from /home/user/.rlock_pwd if it exists
 * else prompt user and create file_suffix
 * force_create_file: bool If true, then prompt user even if file exists
 */
fn getpw(force_create_file: bool) -> String {
    /* Read password from /home/user/.rlock_pwd file */

    match (File::open(get_pwfile_path()), force_create_file) {
        (Ok(f), false) => { 
            let mut file = f;
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => contents,
                Err(msg) => {
                    panic!("Cannot read contents of file. {}", msg);
                }
            }
        },
        /* Create file in case it does not exist */
        (Err(_), _) | (_, true) => {
            println!("Creating password file ... ");
            create_pwfile()
        }
    }
}

pub fn getvalue(key: u32, map: HashMap<u32, String>) -> String {

    match map.get(&key) {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}

fn lockscreen(dpy: *mut Display, rr: Xrandr, screen: i32, colors: HashMap<u32, String>, keyboard_only: bool) -> (Lock, bool) {
    if dpy.is_null() || screen < 0 {
        return (Lock::new(), false);
    } 

    let mut screen_def_return = XColor::new();
    let mut exact_def_return = XColor::new();
    let mut ptgrab = -1;
    let mut kbgrab = -1;

    let mut lock = Lock::new();
    lock.screen = screen;

    // println!("{:?}", getvalue(0, colors.clone()).as_ptr());

    unsafe {
        lock.root = XRootWindow(dpy, screen);
        let default_cmap = XDefaultColormap(dpy, screen);
        for i in 0..Color::NUMCOLS as u32 {
            // println!("{:?}", getvalue(i, colors.clone()));
            let err = XAllocNamedColor(
                dpy,
                default_cmap,
                getvalue(i, colors.clone()).as_ptr() as *const i8,
                &mut screen_def_return,
                &mut exact_def_return);
            lock.colors.push(screen_def_return.pixel);
            // println!("Err: {}, {:?}", err, screen_def_return);
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

        for _ in 0..6 {
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

            if ptgrab == GrabSuccess && kbgrab == GrabSuccess {
            
                if !keyboard_only {
                    XMapRaised(dpy, lock.win);
                }
                if rr.active != 0 {
                    XRRSelectInput(dpy, lock.win, RRScreenChangeNotifyMask);
                }

                XSelectInput(dpy, lock.root, SubstructureNotifyMask);
                return (lock, true);
            }

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

fn readpw(dpy: *mut Display, rr: Xrandr, locks: Vec<Lock>, actual_hash: String, keyboard_only: bool) {
    let mut running = true;
    let mut ev = XEvent::new();
    let mut passwd = Vec::new();
    let mut failure = false;
    let mut old_color = Color::INIT;

    unsafe {
        while running && (XNextEvent(dpy, &mut ev) == 0) {

            let mut ksym: KeySym = 0;

            if ev.get_type() == KeyPress {

                let buf_raw = CString::new("").unwrap().into_raw();
                let num = XLookupString(&mut ev.key, buf_raw, 32, &mut ksym, ptr::null_mut() as *mut XComposeStatus);
                let buf = CString::from_raw(buf_raw);
                // let key_type = get_key_type(ksym);

                /* If key typed is one of the extras ignore it */
                match get_key_type(ksym) {
                    Ok(_) => continue,
                    Err(_) => {}
                };

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
                        };
                        passwd = Vec::new(); 
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

                if !keyboard_only {
                    let color = match passwd.is_empty() {
                        false => Color::INPUT,
                        _ => {
                            match failure {
                                true => Color::FAILED,
                                false => Color::INIT
                            }
                        }
                    };
                    if running && (old_color as u32 != color as u32) {
                        for lock in locks.iter() {
                            XSetWindowBackground(dpy, lock.win, *lock.colors.get(color as usize).unwrap());
                            // XSetWindowBackground(dpy, lock.win, lock.colors[color]);
                            XClearWindow(dpy, lock.win);
                        }
                        old_color = color;
                    }
                    else if (rr.active != 0) && (ev.get_type() == rr.evbase + RRScreenChangeNotify) {
                        let rre = XRRScreenChangeNotifyEvent::from(ev);
                        for lock in locks.iter() {
                            if lock.win == rre.window {
                                match rre.rotation as i32{
                                    RR_Rotate_90 | RR_Rotate_270 => XResizeWindow(dpy, lock.win, rre.height as u32, rre.width as u32),
                                    _ => XResizeWindow(dpy, lock.win, rre.width as u32, rre.height as u32)
                                };
                                XClearWindow(dpy, lock.win);
                                break;
                            }
                        }
                    }
                    else {
                        for lock in locks.iter() {
                            XRaiseWindow(dpy, lock.win);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut rr = Xrandr::new();
    let hash: String;
    let dpy: *mut Display;
    let nscreens: i32;

    /* TODO: add command line arguments functionality to change password */

    let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    // println!("{:?}", args.get_bool("-p"));

    hash = getpw(args.get_bool("-p"));
    let keyboard_only = args.get_bool("-k");
    if keyboard_only {
        println!("rlock is running");
        println!("Enter password to exit");
    }

    unsafe {
        dpy = XOpenDisplay(ptr::null());

        /* XRRQueryExtension returns event and error base codes */
        rr.active = XRRQueryExtension(dpy, &mut rr.evbase, &mut rr.errbase);

        /* Get number of screens from dpy and blank them */
        nscreens = XScreenCount(dpy);
        let mut locks: Vec<Lock> = Vec::new();
        let colors = config::readconfig();

        for s in 0..nscreens {

            let (lock, success) = lockscreen(dpy, rr, s, colors.clone(), keyboard_only);
            match success {
                true => locks.push(lock),
                false => break
            };
        }

        XSync(dpy, 0);

        /* Check if all screens were locked */
        if locks.len() as i32 != nscreens {
            panic!("Could not lock all screens");
        }

        /* run post-lock command */
        /* TODO: understand why slock code has fork */

        readpw(dpy, rr, locks, hash, keyboard_only);

        /* run post-lock command */
        /* TODO: understand why slock code has fork */


    println!("Exited readpw");
}
}
