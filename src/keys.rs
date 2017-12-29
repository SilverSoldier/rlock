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
    INVALID,
}

impl From<u64> for Key{
    fn from(int: u64) -> Key{
        match int {
            0 => Key::KEYPAD,
            1 => Key::FUNCTION,
            2 => Key::MISCFUNCTION,
            3 => Key::PF,
            4 => Key::PRIVATEKEYPAD,
            _ => Key::INVALID,
        }
    }
}

pub fn get_key_type(ksym: KeySym) -> Key {
    match () {
        _ if ksym >= XK_space as u64 && ksym <= XK_equal as u64 => Key::KEYPAD,
        _ if ksym >= XK_F1 as u64 && ksym <= XK_F35 as u64 => Key::FUNCTION,
        _ if ksym >= 0x11000000 && ksym <= 0x1100FFFF => Key::PF,
        _ if ksym >= XK_Select as u64 && ksym <= XK_Break as u64 => Key::MISCFUNCTION,
        _ => Key::INVALID

    }
}
