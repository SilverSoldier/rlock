use std::collections::HashMap;

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


pub fn getcolors(key: u32) -> String {
    // match colorname.get(u8) {
    //     Some(color) => color
    // }
    let colorname = map!{ 1 => "one", 2 => "two" };

    match colorname.get(&key) {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}
