use std::collections::HashMap;

use std::fs::File;
use std::io::prelude::*;

use std::ffi::{ CStr, CString };

use libc::{
    getenv,
};

const config_msg: &'static str = "
Do not edit/remove this line. Change color for each screen by editing only the right hand side of following lines. If file is not parseable, will revert to default config.
";

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
        username = CStr::from_ptr(name).to_string_lossy().into_owned();
    }
    username
}

fn create_color_map(init: &str, input: &str, failed: &str) -> HashMap<u32, String> {
    map!{ 
        0 /* Init */ => init.to_string(),
        1 /* Input */ => input.to_string(),
        2 /* Failed */ => failed.to_string()
    }
}

fn create_default_config() -> HashMap<u32, String> {
    /* Create the default config */
    println!("Used default config");
    create_color_map("black", "#006400", "#8B0000")
}

pub fn parse_contents(mut contents: String) -> HashMap<u32, String> {
    /* Remove the message from the file contents and then separate using
     * whitespaces */
    let config = contents.split_off(config_msg.len() - 1);
    let mut iter = config.split_whitespace();
    iter.next();
    match iter.next() {
        Some(init_col) => {
            iter.next();
            match iter.next() {
                Some(inp_col) => {
                    iter.next();
                    match iter.next() {
                        Some(fail_col) => {
                            return create_color_map(init_col, inp_col, fail_col)
                        },
                        None => {}
                    }
                },
                None => {}
            }
        },
        None => {}
    }
    create_default_config()
}

pub fn read_config() -> HashMap<u32, String> {

    let file_prefix = String::from("/home/");
    let file_suffix = String::from("/.rlock_config");

    let username = getusername();

    let path = file_prefix + &username + &file_suffix;

    match File::open(path) {
        Ok(f) => { 
            println!("Reading from config");
            let mut file = f;
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => return parse_contents(contents),
                Err(_) => {}
            }
        },

        Err(_) => {
            /* TODO: Create file in case it does not exist */
        }
    }
    create_default_config()
}
