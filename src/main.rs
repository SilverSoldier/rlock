extern crate x11;
extern crate pwd;
extern crate libc;

use std::ptr;
use std::process::exit;
use std::ffi::{ CStr, CString };

use x11::xlib::{
    Window,
    Pixmap,
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

fn gethash() -> *mut c_char {
    let mut hash: *mut c_char;
    let pw: *mut passwd = ptr::null_mut();

    /* Check if the current user has a password entry */
    unsafe {
        let uid = getuid();
        let errno = __errno_location();
        let pw = getpwuid(uid);
        if pw.is_null() {
            if let Some(0i32) = errno.as_ref().map(|x| *x) {
                // die("slock: getpwuid: %s\n", strerror(errno));
                die(&CStr::from_ptr(strerror(*errno)).to_string_lossy().into_owned())
                    // die(strerror(*errno));
            }
            else {
                die("slock: cannot retrieve password entry\n");
            }

        }
        hash = (*pw).pw_passwd;
        if strcmp(hash, CString::new("x").unwrap().as_ptr()) != 0 {
            let sp: *mut spwd;
            sp = getspnam((*pw).pw_name);
            if sp.is_null() {
                die("slock: getspnam: cannot retrieve shadow entry.\n Make sure to suid or sgid slock.\n");
            }
            else {
                hash = (*sp).sp_pwdp;
            }
        }

    }

    hash
}

fn main() {
}
