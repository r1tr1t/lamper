use std::time::Duration;

const WINDOW: usize = 2048;
pub const BOLDSTART: &str = "\x1b[1m";
pub const BOLDEND: &str = "\x1b[0m";

pub mod audproc;
pub mod colproc;
pub mod udp;

// misc errors for audproc and colproc
pub enum LampErr {
    PAErr,
    SendErr,
}

// catch method to call on any error received
impl LampErr {
    pub fn catch(&self) {
        match self {
            Self::PAErr => {
                println!("Unrecoverable Pulse Audio error, exiting...");
                std::thread::sleep(Duration::from_secs(1));
                std::process::exit(0);
            }
            Self::SendErr => {
                println!("Unrecoverable error sending data between threads, exiting...");
                std::thread::sleep(Duration::from_secs(1));
                std::process::exit(0);
            }
        }
    }
}
