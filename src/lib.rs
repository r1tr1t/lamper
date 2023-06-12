const WINDOW: usize = 2048;
pub const BOLDSTART: &str = "\x1b[1m";
pub const BOLDEND: &str = "\x1b[0m";

pub mod audproc;
pub mod colproc;
pub mod udp;

// TODO: nested enums for errors from each module
pub enum LampErr {}

// catch method to call on any error received
impl LampErr {
    pub fn catch(&self, err: LampErr) {}
}
