/* Module containing helper function associated with keys */
use x11::keysym::*;
use x11::xlib::KeySym;

#[derive(PartialEq, Clone)]
pub enum Key {
    KEYPAD,
    FUNCTION,
    MISCFUNCTION,
    PF,
    PRIVATEKEYPAD,
}

pub fn get_key_type(ksym: KeySym) -> Result<Key, &'static str> {
    match () {
        _ if ksym >= XK_space as u64 && ksym <= XK_equal as u64 => Ok(Key::KEYPAD),
        _ if ksym >= XK_F1 as u64 && ksym <= XK_F35 as u64 => Ok(Key::FUNCTION),
        _ if ksym >= 0x11000000 && ksym <= 0x1100FFFF => Ok(Key::PRIVATEKEYPAD),
        _ if ksym >= XK_Select as u64 && ksym <= XK_Break as u64 => Ok(Key::MISCFUNCTION),
        _ if ksym >= XK_F1 as u64 && ksym <= XK_F4 as u64 => Ok(Key::PF),
        _ => Err("Not an extra key or not a valid key")
    }
}
