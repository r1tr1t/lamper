// todo:
// error catch fn to make handle various errors and give the option to exit
// ui:
// maximum brightness
// intensity modes?
// revert to original settings on ending

extern crate lamper;

use lamper::{
    audproc, colproc,
    udp::{self, InitErr, Lamp},
    {BOLDEND, BOLDSTART},
};
use std::{
    io::{self, Write},
    net::AddrParseError,
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

// set the max brightness level
fn max_brightness() -> u8 {
    print!(
        "{}Set maximum brightness(1-100){} [100]: ",
        BOLDSTART, BOLDEND
    );
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

fn connect() -> Lamp {
    let catch = |err: InitErr| -> bool {
        match err {
            InitErr::DevStatusErr => {
                println!("Error retrieving device status, retry connection? [Y/n]");
                match read_line() {
                    Ok(val) => {
                        if val == "\n" || val == "y" || val == "Y" {
                            true
                        } else {
                            !(val == "n" || val == "N")
                        }
                    }
                    Err(err) => {
                        println!("Failed to read line: {}", err);
                        false
                    }
                }
            }
            _ => false,
        }
    };

    loop {
        println!(
            "{}Searching for Govee device on current network...{}",
            BOLDSTART, BOLDEND
        );
        let res = match udp::init() {
            Ok(lamp) => Some(lamp),
            Err(err) => match catch(err) {
                true => {
                    continue;
                }
                false => {
                    println!("Unrecoverable error, exiting...");
                    std::thread::sleep(Duration::from_secs(2));
                    std::process::exit(0);
                }
            },
        };
        if let Some(ref lamp) = res {
            println!("{}Govee device found:{}", BOLDSTART, BOLDEND);
            println!("{}IP:{} {}", BOLDSTART, BOLDEND, &lamp.addr);
            let pwr = match &lamp.init.pwr {
                Turn::On => "On",
                Turn::Off => "Off",
            };
            let r = &lamp.init.color[0];
            let g = &lamp.init.color[1];
            let b = &lamp.init.color[2];
            println!("{}Initial State:{}\nPower: {}\nBrightness: {}\nColor(RGB): {}, {}, {}\nColor(Kelvin): {}", BOLDSTART, BOLDEND, pwr, &lamp.init.bright, r, g, b, &lamp.init.temp);
        }
        return res.unwrap();
    }
}

fn clear() {
    print!("\x1B[2J\x1B[H")
}

fn line() {
    println!("---------------------------------");
}

fn flush() {
    let mut stdout = std::io::stdout();
    stdout.flush().expect("failed to flush stdout");
}

fn main() {
    clear();
    connect();
    line();
    let max_brightness = max_brightness();
    clear();
}
