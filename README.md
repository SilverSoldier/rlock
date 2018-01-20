# rlock
Clone of [slock](https://tools.suckless.org/slock/), a simple X display locker (by suckless), written in rust.

## Features
+ Keyboard-only lock command line option: Locks keyboard while still displaying monitor

+ Custom password: Unlike slock, which uses the user's system password to exit the lock, rlock allows to set a custom password stored locally, with command line option to modify.

## Usage
### Dowload rust
For unix, simply run 
```
curl https://sh.rustup.rs -sSf | sh
```
to get the latest version of rust

### Clone repository
```
git clone https://github.com/SilverSoldier/rlock.git
```

### Compile and build binary using cargo
```
cargo build
cargo run
```

## Usage
Add `/target/debug/rlock` to `~/bin`, if `~/bin` is in your `PATH`

Use `-h` or `--help` for the command line arguments
