extern crate x11;
extern crate pwd;
extern crate libc;

use std::ptr;
use std::process::exit;
use std::ffi::{ CStr, CString };
use std::io::prelude;

use x11::xlib::{
    Window,
    Pixmap,
    Display,
};

use libc::{
    getuid,
    getpwuid,
    __errno_location,
    strerror,
    passwd,
    c_char,
    strcmp,
    spwd,
    getspnam,
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

fn die(errstr: &str) {
    println!("{}", errstr);
    exit(1);
}

/* Abandoned in order to use a shorter user defined password */
fn gethash() -> *mut c_char {
    let mut hash: *mut c_char;
    let pw: *mut passwd = ptr::null_mut();

    /* Check if the current user has a password entry */
    unsafe {
        /* Get user id of user who requested */
        let uid = getuid();
        // let uid = 1020;
        let errno = __errno_location();

        /* Get entry of this user from /etc/passwd */
        let pw = getpwuid(uid);

        /* Check if entry existed or not */
        if pw.is_null() {

            /* If there is no errno print default message */
            if let Some(0i32) = errno.as_ref().map(|x| *x) {
                die("slock: cannot retrieve password entry\n");
            }
            else {
                die(&CStr::from_ptr(strerror(*errno)).to_string_lossy().into_owned())
            }
        }

        println!("Reached checkpoint 2");
        
        hash = (*pw).pw_passwd;

        /* Entry has x in passwd field if there exists a password for that user */
        if strcmp(hash, CString::new("x").unwrap().as_ptr()) == 0 {
            println!("Reached checkpoint 3");
            let sp: *mut spwd;
            println!("Name: {:?}",(*pw).pw_name);
            sp = getspnam((*pw).pw_name);
            println!("sp: {:?}",sp);
            if sp.is_null() {
                die("slock: getspnam: cannot retrieve shadow entry.\nMake sure to suid or sgid slock.\n");
            }
            else {
                hash = (*sp).sp_pwdp;
                print!("{:?}", hash);
            }
        }

    }

    hash
}

fn getpw(){
    /* Read password from ~/.rlock_pwd file */

    /* Introduce functionality to create file in case it was not created */
}

fn main() {
    let rr: Xrandr;
    let pwd: *mut passwd;
    let grp: *mut group;
    let duid: uid_t;
    let dgid: gid_t;
    let hash: *mut c_char;
    let dpy: Display;
    let (s, nlocks, nscreens): (u32, u32, u32);

    /* Omitting code from original slock which parses arguments to give version info */

    hash = gethash();
    unsafe {
        let printable_string = CStr::from_ptr(hash).to_string_lossy().into_owned();
        println!("{}", printable_string);
    }
}
