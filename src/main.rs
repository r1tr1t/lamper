extern crate lamper;

use std::{io, thread, sync::mpsc};
use lamper::{audproc, colproc};
use reqwest;
use serde_json::{json};
use tokio;
use colored::*;

async fn power_cycle() {
    println!("Cycle power? [y/n]");
    let mut input = String::new();
    read_line(&mut input);
    loop{
        match input.to_lowercase().as_str() {
            "y" => {
                power("off").await;
                power("on").await;
                break
            },
            "n" => break,
            _ => println!("Please input y or n")
    
        }
    }
}

async fn power(arg: &str) {
    let client = reqwest::Client::new();

    let body = json!({
        "device": "2C:14:C1:33:36:32:42:8E",
        "model": "H6051",
        "cmd": {
            "name": "turn",
            "value": arg
        }
    });


    client
    .put("https://developer-api.govee.com/v1/devices/control")
    .header("Content-Type", "application/json")
    .header("Govee-API-Key", "32f653a8-87cd-476c-b6b3-904fe7f7948d")
    .json(&body)
    .send()
    .await
    .unwrap();

    // println!("{:?}", res);
}

async fn brightness() -> u32 {
    loop {
        let mut uin: u32 = 0;    
        read_u32(&mut uin);
        if uin <= 100 {
            let client = reqwest::Client::new();
    
        let body = json!({
            "device": "2C:14:C1:33:36:32:42:8E",
            "model": "H6051",
            "cmd": {
                "name": "brightness",
                "value": uin
            }
        });
    
    
        client
        .put("https://developer-api.govee.com/v1/devices/control")
        .header("Content-Type", "application/json")
        .header("Govee-API-Key", "32f653a8-87cd-476c-b6b3-904fe7f7948d")
        .json(&body)
        .send()
        .await
        .unwrap();

        break uin
        } else {
            println!("Please input a number between 1 and 100");
            
        }
    }
    // println!("{:?}", res);
}

fn read_line(input:&mut String) {
    io::stdin().read_line(input).expect("Failed to read line");
    *input = input.trim().to_string();
}

fn read_u32(uin:&mut u32) {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    *uin = input.trim().parse::<u32>().expect("Failed to parse input to u32");
}

fn clear() {
    print!("\x1B[2J")
}

#[tokio::main]
async fn main() {
    power_cycle().await;
    // let mut input  = String::new();
    println!("Set Inital Brightness [0-100]:");
    let brightness = brightness().await;

    let (aptx, aprx) = mpsc::channel();
    // let (cptx, cprx) = mpsc::channel();

    let ap = thread::spawn( move ||{
        audproc::start(aptx);
    });

    let cp = thread::spawn(move ||{
        colproc::process(aprx);
    });

    loop {}
    // ap.join().unwrap();
    // cp.join().unwrap();
}
