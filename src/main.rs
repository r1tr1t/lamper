use lamper::{
    audproc, colproc,
    udp::{self, InitErr, Lamp},
    LampErr, {BOLDEND, BOLDSTART, CMDDELAY},
};
use std::{
    io::{self, Write},
    sync::{mpsc, Arc, RwLock},
    thread,
    time::Duration,
};
use udp::{Cmd, Turn};

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
                if val.is_empty() {
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

fn connect() -> (Lamp, bool) {
    let catch = |err: InitErr| -> bool {
        match err {
            InitErr::DevStatusErr => {
                println!("Error retrieving device status, retry connection? [Y/n]");
                match read_line() {
                    Ok(val) => {
                        if val.is_empty() || val == "y" || val == "Y" {
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

    // establish connection with device
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
        let res = res.unwrap();
        if let Err(err) = res.send_cmd(Cmd::OnOff(Turn::On)) {
            println!("Failed to turn lamp on: {:?}", err);
        }
        return (res, true);
    }
}

fn run(conn: Arc<RwLock<bool>>, lamp: Lamp) {
    // conn atomics
    let apconn = Arc::clone(&conn);
    let cpconn = Arc::clone(&conn);
    // channels
    let (aptx, aprx) = mpsc::channel();
    let (cptx, cprx) = mpsc::channel();

    // threads
    let ap = thread::spawn(|| match audproc::start(aptx, apconn) {
        Ok(_) => {}
        Err(err) => err.catch(),
    });

    let cp = thread::spawn(|| match colproc::process(aprx, cptx, cpconn) {
        Ok(_) => {}
        Err(err) => err.catch(),
    });

    let mut check: u8 = 0;
    loop {
        match cprx.recv() {
            Ok(val) => {
                if check < 255 {
                    let (brightness, rgb) = val;
                    match lamp.send_cmd(Cmd::Brightness(brightness)) {
                        Ok(_) => {}
                        Err(err) => println!("Error sending brightness: {:?}", err),
                    }
                    thread::sleep(Duration::from_millis(CMDDELAY as u64));
                    match lamp.send_cmd(Cmd::Color(rgb)) {
                        Ok(_) => {}
                        Err(err) => println!("Error sending color: {:?}", err),
                    }
                    check += 1;
                } else {
                    loop {
                        match lamp.check() {
                            Ok(_) => {
                                check = 0;
                                *conn.write().unwrap() = true;
                                break;
                            }
                            Err(_) => {
                                *conn.write().unwrap() = false;
                                println!("No response from device, retry connection? [Y/n]");
                                match read_line() {
                                    Ok(val) => {
                                        if val.is_empty() || val == "y" || val == "Y" {
                                            continue;
                                        } else if val == "n" || val == "N" {
                                            break;
                                        }
                                    }
                                    Err(err) => {
                                        println!("Failed to read line: {}", err);
                                        continue;
                                    }
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(CMDDELAY as u64));
                    }
                    if !*conn.read().unwrap() {
                        break;
                    }
                }
            }
            Err(err) => LampErr::from(err).catch(),
        }
    }

    ap.join().unwrap();
    cp.join().unwrap();
    exit(lamp);
}

// clear terminal
fn clear() {
    print!("\x1B[2J\x1B[H")
}

// print line
fn line() {
    println!("---------------------------------");
}

// flush output (this and the 2 prior should probably be macros when I get to it)
fn flush() {
    let mut stdout = std::io::stdout();
    stdout.flush().expect("failed to flush stdout");
}

// read a line from stdin
fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn exit(lamp: Lamp) {
    std::thread::sleep(Duration::from_secs(2));
    lamp.restore()
        .expect("Could not restore lamp to initial settings");
    std::process::exit(0);
}

fn main() {
    clear();
    let (mut lamp, conn) = connect();
    let conn = Arc::new(RwLock::new(conn));
    line();
    lamp.set_maxb(max_brightness());
    line();
    run(conn, lamp);
}
