#![feature(ip_addr, convert)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::string::String;
use std::borrow::Borrow;
use std::convert::AsRef;

fn start_server(ip: &str) {
  let ip_addr = Ipv4Addr::from_str(ip).unwrap();
  let addr = SocketAddr::new(IpAddr::V4(ip_addr), 34561);

  let mut socket = match UdpSocket::bind(addr) {
    Ok(s) => s,
    Err(e) => panic!("Failed to bind socket! {}")
  };

  println!("I survived! (now listening??)");

  let mut buf = [0u8; 256];
  match socket.recv_from(&mut buf) {
    Ok((len, src_addr)) => {
      println!("Fuckin' got it!!!!!!!!!!!!!!!!!!!!!!!");
    }

    Err(e) => println!("Failed to receive :(! error: {}", e)
  }
}

fn main() {
  let mut stdin = std::io::stdin();

  println!("Enter ip address!");
  let mut ipstr = String::new();
  stdin.read_line(&mut ipstr);

  ipstr.pop();
  start_server(ipstr.as_ref());
}
