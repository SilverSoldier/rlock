/* Module for handling the command line arguments facility */
extern crate docopt;

/* Defining a USAGE string */
pub const USAGE: &'static str = "
Program.

Usage: program [options]

Options:
    -h, --help          Show this message.
    -p, --password      Change password.
    -k, --keypad        Lock only keypad.
";
