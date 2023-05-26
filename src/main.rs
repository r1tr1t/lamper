// todo:
// error handling to mutate conn in audproc::start and colproc::process
// ui:
// maximum brightness
// intensity modes?
// revert to original settings on ending

extern crate lamper;

use std::{io, thread, sync::mpsc};
use lamper::{audproc, colproc, udp};
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
    let init = udp::init();
    let mut conn;
    println!("{:?}", init);
    let lamp = match init {
        Ok(lamp) => {conn = true; lamp},
        Err(_) => panic!("Failed to initialize")
    };
    let initstate = lamp.dev_status().expect("failed to get lamp status");
    
    let (aptx, aprx) = mpsc::channel();
    let (cptx, cprx) = mpsc::channel();

    let ap = thread::spawn( move ||{
        audproc::start(aptx, &conn);
    });

    let cp = thread::spawn(move ||{
        colproc::process(aprx, cptx, &conn);
    });

    while conn {
        let (brightness, rgb) = cprx.recv().expect("failed to receive from cp");
        lamp.send_cmd(Cmd::Brightness(brightness)).expect("failed to send packet to lamp");
        lamp.send_cmd(Cmd::Color(rgb)).expect("failed to send packet to lamp");
    }

    ap.join().unwrap();
    cp.join().unwrap();
    std::process::exit(0);
}
