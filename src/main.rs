extern crate lamper;

use std::{io, thread, sync::mpsc};
use lamper::{audproc, colproc, udp};
use reqwest;
use serde_json::{json};
use udp::{Cmd, Turn};


fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}

fn read_u32() -> u32 {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let uin = input.trim().parse::<u32>().expect("Failed to parse input to u32");
    uin
}

fn clear() {
    print!("\x1B[2J")
}

fn main() {
    let lamp = udp::init().expect("fuck if I know");
    println!("{:?}", lamp);

    let result = lamp.send_cmd(Cmd::Brightness(15)).unwrap();
    println!("{:?}", result);

    // let (aptx, aprx) = mpsc::channel();
    // let (cptx, cprx) = mpsc::channel();

    // let ap = thread::spawn( move ||{
    //     audproc::start(aptx);
    // });

    // let cp = thread::spawn(move ||{
    //     colproc::process(aprx);
    // });

    // ap.join().unwrap();
    // cp.join().unwrap();
}
