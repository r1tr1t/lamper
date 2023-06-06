// todo:
// error handling to mutate conn in audproc::start and colproc::process
// ui:
// maximum brightness
// intensity modes?
// revert to original settings on ending

extern crate lamper;

use lamper::{audproc, colproc, udp};
use std::{
    io::{self, Write},
    num::ParseIntError,
    sync::mpsc,
    thread,
    time::Duration,
};
use udp::{Cmd, Turn};

fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn max_brightness() -> u8 {
    print!("Set maximum brightness(1-100) [100]: ");
    flush();
    loop {
        match read_line() {
            Ok(val) => {
                if val == "\n" {
                    return 100;
                } else if let Ok(num) = val.trim().parse::<u8>() {
                    if num <= 100 && num > 0 {
                        return num;
                    } else {
                        print!("Please enter a value 1-100 or press enter for default [100]: ");
                        flush();
                    }
                } else {
                    print!("Please enter a value 1-100 or press enter for default [100]: ");
                    flush();
                }
            }
            Err(err) => {
                println!("Failed to read line: {}", err);
            }
        }
    }
}

fn clear() {
    print!("\x1B[2J\x1B[H")
}

fn flush() {
    let mut stdout = std::io::stdout();
    stdout.flush().expect("failed to flush stdout");
}

fn main() {
    clear();
    let max_brightness = max_brightness();
}
