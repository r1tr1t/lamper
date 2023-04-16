// todo:
// finish error propegation
// finish cmd fn
// make the cmd fn ideal for spamming large amounts of color commands

use std::{net::{Ipv4Addr, UdpSocket, SocketAddrV4, AddrParseError}, str::FromStr};
use serde_json::{Value};
// use arr_macro::arr;

// cmd types
pub enum Cmd {
    OnOff(Turn),
    Brightness(u8),
    DevStatus,
    Color([u8; 3])
}

// cmd success types
pub enum CmdSuccess {
    Success,
    Status(Turn, Cmd, Cmd)
}

// on, or maybe off
pub enum Turn {
    On,
    Off
}

// cmd error types
pub enum CmdErr {

}

// init error types
pub enum InitErr {
    AddrParseErr,
    MulticastTtlErr
}

impl From<AddrParseError> for InitErr {
    fn from(err: AddrParseError) -> Self {
        InitErr::AddrParseErr
    }
}

impl From<std::io::Error> for InitErr {
    fn from(err: std::io::Error) -> Self {
        InitErr::MulticastTtlErr
    }
}

// creates udp socket, joins the multicast group, queries device
// returns the socket and the ip of the first device to respond
pub fn init() -> Result<(UdpSocket, SocketAddrV4), InitErr> {
    let socket = UdpSocket::bind("0.0.0.0:4002").expect("failed to bind");
    socket.set_multicast_ttl_v4(1)?;

    let multicast_addr = Ipv4Addr::from([239, 255, 255, 250]);
    let port = 4001;
    let multicast_socket = SocketAddrV4::new(multicast_addr, port);

    let msg = r#"{"msg": {"cmd" : "scan", "data" : {"account_topic" : "reserve"}}}"#;
    let bytes = msg.as_bytes();

    socket.send_to(bytes, multicast_socket).expect("failed to send to multicast socket");

    let mut buf = [0u8; 256];
    socket.recv_from(&mut buf).unwrap();

    let json = trimmer(&buf);

    let ip = json["msg"]["data"]["ip"].to_string();
    let socketaddr = SocketAddrV4::new(Ipv4Addr::from_str(&ip)?, 4003);

    return Ok((socket, socketaddr))
}

// WIP perform all possible commands over udp
pub fn cmd(cmd: Cmd, socket: UdpSocket) -> Result<CmdSuccess, CmdErr> {
    match cmd {
        Cmd::OnOff(val) => {todo!()},
        Cmd::Brightness(val) => {todo!()},
        Cmd::Color(val) => {todo!()},
        Cmd::DevStatus => {todo!()}
    }
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
