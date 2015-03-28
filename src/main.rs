#![feature(ip_addr)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::string::String;
use std::borrow::Borrow;

fn main() {
    let mut stdin = std::io::stdin();

    println!("Enter ip address!");
    let mut ipstr = String::new();
    stdin.read_line(&mut ipstr);

    let ip_addr = IpAddr::from_str(ipstr.borrow()).unwrap();

    let addr = SocketAddr::new(ip_addr, 34561);

    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => panic!("Failed to bind socket! {}")
    };

    /*
    let mut buf = [0u8; 256];
    match socket.recv_from(&mut buf) {
    }
    */
}
