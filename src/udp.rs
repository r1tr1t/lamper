use serde_json::{json, Value};
use std::{
    net::{AddrParseError, Ipv4Addr, SocketAddrV4, UdpSocket},
    num::ParseIntError,
    str::FromStr,
};
// use arr_macro::arr;

// cmd types
#[derive(Debug)]
pub enum Cmd {
    OnOff(Turn),
    Brightness(u8),
    Color([u8; 3]),
}

// on, or maybe off
#[derive(Debug)]
pub enum Turn {
    On,
    Off,
}

// cmd error types
#[derive(Debug)]
pub enum CmdErr {
    ParseIntErr,
    SerdeErr,
    InvalidBrightnessErr,
    MiscCmdErr,
    RecvErr,
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
    SerdeErr,
    DevStatusErr,
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

impl From<CmdErr> for InitErr {
    fn from(_: CmdErr) -> Self {
        InitErr::DevStatusErr
    }
}

// socket, address, init state, max brightness
#[derive(Debug)]
pub struct Lamp {
    socket: UdpSocket,
    pub addr: SocketAddrV4,
    pub init: State,
    maxb: u8,
}

#[derive(Debug)]
pub struct State {
    pub pwr: Turn,
    pub bright: u8,
    pub color: [u8; 3],
    pub temp: u16,
}

pub enum CmdValue {
    Single(u8),
    RGB([u8; 3]),
}

impl Lamp {
    fn new(socket: UdpSocket, addr: SocketAddrV4, init: State) -> Self {
        Lamp {
            socket,
            addr,
            init,
            maxb: 0,
        }
    }

    // send any command to lamp over udp
    pub fn send_cmd(&self, cmd: Cmd) -> Result<(), CmdErr> {
        let (command, value) = match cmd {
            Cmd::OnOff(val) => {
                let command = "turn";
                let value = match val {
                    Turn::On => 1,
                    Turn::Off => 0,
                };
                (command, CmdValue::Single(value))
            }
            Cmd::Brightness(val) => {
                if val > 100 {
                    return Err(CmdErr::InvalidBrightnessErr);
                }
                let command = "brightness";
                (command, CmdValue::Single(val))
            }
            Cmd::Color(val) => {
                let command = "colorwc";
                (command, CmdValue::RGB(val))
            }
        };

        let msg = match value {
            CmdValue::Single(val) => serde_json::to_vec(&json!({
                "msg": {
                    "cmd": command,
                    "data": {
                        "value": val
                    }
                }
            }))?,
            CmdValue::RGB(val) => serde_json::to_vec(&json!({
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
            }))?,
        };

        self.socket.send_to(&msg, self.addr)?;

        Ok(())
    }

    pub fn restore(&self) -> Result<(), CmdErr> {
        let pwr = match self.init.pwr {
            Turn::Off => 0,
            Turn::On => 1,
        };
        let bright = self.init.bright;
        let color = self.init.color;
        let temp = self.init.temp;

        let msg = serde_json::to_vec(&json!({
            "msg": {
                "cmd": "turn",
                "data": {
                    "value": pwr
                }
            },
            "msg": {
                "cmd": "brightness",
                "data": {
                    "value": bright
                }
            },
            "msg": {
                "cmd": "colorwc",
                "data": {
                    "color": {
                        "r": color[0],
                        "g": color[1],
                        "b": color[2]
                    },
                    "colorTemInKelvin": temp
                }
            }
        }))?;

        self.socket.send_to(&msg, self.addr)?;
        Ok(())
    }

    // check if still connected
    pub fn check(&self) -> Result<(), CmdErr> {
        match dev_status(&self.socket, &self.addr) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    // set max brightness
    pub fn set_maxb(&mut self, maxb: u8) {
        self.maxb = maxb
    }

    // return max brightness
    pub fn maxb(&self) -> u8 {
        self.maxb
    }
}

// creates udp socket, joins the multicast group, queries device
// returns Lamp struct with socket and ip of first device to respond
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

    socket
        .send_to(&msg, multicast_socket)
        .expect("failed to send to multicast socket");

    let mut buf = [0u8; 256];
    socket.recv_from(&mut buf)?;

    let json = trimmer(&buf);

    let ip = match json["msg"]["data"]["ip"].as_str() {
        Some(ip) => ip,
        None => return Err(InitErr::AddrParseErr),
    };
    let addr = SocketAddrV4::new(Ipv4Addr::from_str(ip)?, 4003);
    let init = dev_status(&socket, &addr)?;

    Ok(Lamp::new(socket, addr, init))
}

// get lamp status
fn dev_status(socket: &UdpSocket, addr: &SocketAddrV4) -> Result<State, CmdErr> {
    let msg = serde_json::to_vec(&json!({
        "msg": {
            "cmd": "devStatus",
            "data": {

            }
        }
    }))?;

    socket.send_to(&msg, addr)?;

    let mut recv_buf = [0u8; 256];
    socket.recv_from(&mut recv_buf).or(Err(CmdErr::RecvErr))?;

    let recv = trimmer(&recv_buf);

    // json! macro can't be used in match statements??
    let pwr = if recv["msg"]["data"]["onOff"] == json!(1) {
        Turn::On
    } else if recv["msg"]["data"]["offOff"] == json!(0) {
        Turn::Off
    } else {
        return Err(CmdErr::RecvErr);
    };

    let bright = match &recv["msg"]["data"]["brightness"] {
        Value::Number(num) => num.as_u64().unwrap_or(0) as u8,
        _ => return Err(CmdErr::RecvErr),
    };

    let r = match &recv["msg"]["data"]["color"]["r"] {
        Value::Number(num) => num.as_u64().unwrap_or(0) as u8,
        _ => return Err(CmdErr::RecvErr),
    };
    let g = match &recv["msg"]["data"]["color"]["g"] {
        Value::Number(num) => num.as_u64().unwrap_or(0) as u8,
        _ => return Err(CmdErr::RecvErr),
    };
    let b = match &recv["msg"]["data"]["color"]["b"] {
        Value::Number(num) => num.as_u64().unwrap_or(0) as u8,
        _ => return Err(CmdErr::RecvErr),
    };

    let color = [r, g, b];

    let temp = match &recv["msg"]["data"]["colorTemInKelvin"] {
        Value::Number(num) => num.as_u64().unwrap_or(0) as u16,
        _ => return Err(CmdErr::RecvErr),
    };

    Ok(State {
        pwr,
        bright,
        color,
        temp,
    })
}

// trims whitespace from response buffer, could be more efficient but don't feel like fixing it
fn trimmer(buf: &[u8]) -> Value {
    let mut end = buf.len() - 1;
    let mut trav: u8 = buf[end];
    while trav == 0 {
        end -= 1;
        trav = buf[end];
    }

    let length = end + 1;
    let mut trim: Vec<u8> = Vec::with_capacity(length);

    for i in buf.iter().take(length) {
        trim.push(*i);
    }

    let json: Value = serde_json::from_slice(&trim).unwrap();

    json
}
