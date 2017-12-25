use std::collections::HashMap;

use std::fs::File;
use std::io::prelude::*;
use std::io;

use std::ffi::{ CStr, CString };

use libc::{
    getenv,
};

macro_rules! map (
    {$($key:expr => $value:expr), + } => {
        {
            let mut m = HashMap::new();
            $(
                m.insert($key, $value);
             )+
            m
        }
    };
);

pub fn getusername() -> String {
    let username: String;
    unsafe{
        let name = getenv(CString::new("USER").unwrap().as_ptr());
        username= CStr::from_ptr(name).to_string_lossy().into_owned();
    }
    username
}

fn createdefaultconfig() -> HashMap<u32, String> {
    /* Create the default config */

    let colors = map!{ 
        0 /* Init */ => "black".to_string(),
        1 /* Input */ => "blue".to_string(),
        2 /* Failed */ => "red".to_string()
    };

    /* TODO:  Write to file */

    colors
}

pub fn readconfig() -> HashMap<u32, String> {

    let file_prefix = String::from("/home/");
    let file_suffix = String::from("/.rlock_config");

    let username = getusername();

    let path = file_prefix + &username + &file_suffix;

    match File::open(path) {
        Ok(f) => { 
            let mut file = f;
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    // contents
                    /* TODO: handle file contents and parsing */
                    createdefaultconfig()
                },
                Err(msg) => {
                    panic!("Cannot read contents of file. {}", msg);
                }
            }
        },

        /* Create file in case it does not exist */
        Err(_) => {
            createdefaultconfig()
        }
    }
}
