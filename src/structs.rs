/* To implement the trait new for all the required structs */

use x11::xlib::*;

pub trait Constructor {
    fn new() -> Self;
}

#[derive(Debug)]
pub struct Lock {
    pub screen: i32,
    pub root: Window,
    pub win: Window,
    pub pmap: Pixmap,
    pub colors: Vec<u64>,
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
pub struct Xrandr {
    pub active: i32,
    pub evbase: i32,
    pub errbase: i32,
}

impl Xrandr {
    pub fn new() -> Xrandr {
        Xrandr { active: 0, evbase: 0, errbase: 0 }
    }
}

impl Constructor for XColor {
    fn new() -> XColor {
        XColor { pixel: 0, red: 0, green: 0, blue: 0, flags: 0, pad: 0 }
    }
}

impl Constructor for XSetWindowAttributes {
    fn new() -> XSetWindowAttributes {
        XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0,
            border_pixmap: 0,
            border_pixel: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            colormap: 0,
            cursor: 0,
        }
    }
}

impl Constructor for XEvent {
    fn new() -> XEvent {
        XEvent { pad: [0; 24] }
    }
}

