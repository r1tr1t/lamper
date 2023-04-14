use std::net::{Ipv4Addr, UdpSocket, SocketAddrV4};
use serde_json::{Result, Value};
use arr_macro::arr;

pub fn init() -> Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:4002").expect("failed to bind");

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

    return Ok(ip)
}

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