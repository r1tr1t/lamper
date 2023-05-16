// todo:
// remove the devstatus command and staus brance of CmdSuccess enum to eliminate the need for a response

use std::{net::{Ipv4Addr, UdpSocket, SocketAddrV4, AddrParseError}, str::FromStr, num::ParseIntError};
use serde_json::{Value, json};
// use arr_macro::arr;

// cmd types
#[derive(Debug)]
pub enum Cmd {
    OnOff(Turn),
    Brightness(u8),
    DevStatus,
    Color([u8; 3])
}

// cmd success types
#[derive(Debug)]
pub enum CmdSuccess {
    Success,
    Status(Turn, Cmd, Cmd)
}

// on, or maybe off
#[derive(Debug)]
pub enum Turn {
    On,
    Off
}

// cmd error types
#[derive(Debug)]
pub enum CmdErr {
    ParseIntErr,
    SerdeErr,
    InvalidBrightnessErr,
    MiscCmdErr
}

impl From<std::io::Error> for CmdErr {
    fn from(_: std::io::Error) -> Self {
        CmdErr::MiscCmdErr
    }
}

impl From<ParseIntError> for CmdErr {
    fn from(_: ParseIntError) -> Self {
        CmdErr::ParseIntErr
    }
}

impl From<serde_json::Error> for CmdErr {
    fn from(_: serde_json::Error) -> Self {
        CmdErr::SerdeErr
    }
}

// init error types
#[derive(Debug)]
pub enum InitErr {
    AddrParseErr,
    MiscInitErr,
    SerdeErr
}

impl From<AddrParseError> for InitErr {
    fn from(_: AddrParseError) -> Self {
        InitErr::AddrParseErr
    }
}

impl From<std::io::Error> for InitErr {
    fn from(_: std::io::Error) -> Self {
        InitErr::MiscInitErr
    }
}

impl From<serde_json::Error> for InitErr {
    fn from(_: serde_json::Error) -> Self {
        InitErr::SerdeErr
    }
}

#[derive(Debug)]
pub struct Lamp {
    socket: UdpSocket, 
    addr: SocketAddrV4
}

pub enum CmdValue{
    Single(u8),
    RGB([u8; 3])
}

impl Lamp {
    fn new(socket: UdpSocket, addr: SocketAddrV4) -> Self {
        Lamp {socket, addr}
    }

    pub fn send_cmd(&self, cmd: Cmd) -> Result<CmdSuccess, CmdErr> {
        let (command, value) = match cmd {
            Cmd::OnOff(val) => {
                let command = "turn";
                let value = match val {
                    Turn::On => {1},
                    Turn::Off => {0},
                };
                (command, Some(CmdValue::Single(value)))
            },
            Cmd::Brightness(val) => {
                if val > 100 {return Err(CmdErr::InvalidBrightnessErr)}
                let command = "brightness";
                (command, Some(CmdValue::Single(val)))
            },
            Cmd::Color(val) => {
                let command = "colorwc";
                (command, Some(CmdValue::RGB(val)))
            },
            Cmd::DevStatus => {
                let command = "devStatus";
                (command, None)
            }
        };

        let msg = match value {
            Some(CmdValue::Single(val)) => {serde_json::to_vec(&json!({
                "msg": {
                    "cmd": command,
                    "data": {
                        "value": val
                    }
                }
            }))?},
            Some(CmdValue::RGB(val)) => {serde_json::to_vec(&json!({
                "msg": {
                    "cmd": command,
                    "data": {
                        "color": {
                            "r": val[0],
                            "g": val[1],
                            "b": val[2]
                        }
                    }
                }
            }))?},
            None => {serde_json::to_vec(&json!({
                "msg": {
                    "cmd": command,
                    "data": {
                        
                    }
                }
            }))?},
        };
        

        self.socket.send_to(&msg, self.addr)?;

        let mut buf = [0u8; 256];
        self.socket.recv_from(&mut buf)?;

        let rx = match buf.len() {
            0 => CmdSuccess::Success,
            _ => {
                let json = trimmer(&buf);
                let power = match json["msg"]["data"]["onOff"].as_str() {
                    Some("0") => Turn::Off,
                    Some("1") => Turn::On,
                    Some(_) => Turn::Off,
                    None => Turn::Off
                };
                let brightness = match json["msg"]["data"]["brightness"].as_str() {
                    Some(val) => Cmd::Brightness(val.parse::<u8>()?),
                    None => Cmd::Brightness(0)
                };
                let color = match &json["msg"]["data"]["color"] {
                    rgb => {
                        let r = rgb["r"].as_u64().unwrap() as u8;
                        let g = rgb["g"].as_u64().unwrap() as u8;
                        let b = rgb["b"].as_u64().unwrap() as u8;
                        Cmd::Color([r, g, b])
                    },
                    _ => Cmd::Color([0, 0, 0])
                };
                
                CmdSuccess::Status(power, brightness, color)
            }
        };

        Ok(rx)
    }
}

// creates udp socket, joins the multicast group, queries device
// returns the socket and the ip of the first device to respond
pub fn init() -> Result<Lamp, InitErr> {
    let socket = UdpSocket::bind("0.0.0.0:4002").expect("failed to bind");
    socket.set_multicast_ttl_v4(1)?;

    let multicast_addr = Ipv4Addr::from([239, 255, 255, 250]);
    let port = 4001;
    let multicast_socket = SocketAddrV4::new(multicast_addr, port);

    let msg = serde_json::to_vec(&json!({
        "msg": {
            "cmd": "scan",
            "data": {
                "account_topic": "reserve"
            }
        }
    }))?;


    socket.send_to(&msg, multicast_socket).expect("failed to send to multicast socket");

    let mut buf = [0u8; 256];
    socket.recv_from(&mut buf)?;

    let json = trimmer(&buf);

    let ip = match json["msg"]["data"]["ip"].as_str() {
        Some(ip) => ip,
        None => return Err(InitErr::AddrParseErr)
    };
    let addr = SocketAddrV4::new(Ipv4Addr::from_str(ip)?, 4003);

    Ok(Lamp::new(socket, addr))
}

// trims whitespace from response buffer
fn trimmer(buf: &[u8]) -> Value {
    let mut end = buf.len() - 1;
    let mut trav: u8 = buf[end];
    while trav == 0 {
        end -= 1;
        trav = buf[end];
    }
    
    let length = end + 1;
    let mut trim: Vec<u8> = Vec::with_capacity(length);

    for i in 0..length {
        trim.push(buf[i]);
    }

    let json: Value = serde_json::from_slice(&trim).unwrap();

    json
}
