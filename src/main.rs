#![feature(ip_addr, convert, std_misc)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::string::String;
use std::convert::AsRef;
use std::thread;
use std::mem::transmute;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::duration::Duration;

#[macro_use]
mod testing;

fn start_server(ip: &str) {
  let ip_addr = Ipv4Addr::from_str(ip).unwrap();
  let addr = SocketAddr::new(IpAddr::V4(ip_addr), 34561);

  let socket = match UdpSocket::bind(addr) {
    Ok(s) => s,
    Err(e) => panic!("Failed to bind socket! {}", e)
  };

  println!("I survived! (now listening??)");

  let mut buf = [0u8; 256];
  loop {
    match socket.recv_from(&mut buf) {
      Ok((len, src_addr)) => {
        println!("Fuckin' got it!!!!!!!!!!!!!!!!!!!!!!!");
        if len == 1 && buf[0] == 1 {
          // ping!
          unsafe {
            let mut ms = transmute::<_, *mut i32>(&buf[0]);
            // NOTE random value for now...
            *ms = 15;
            socket.send_to(buf.as_ref(), src_addr).unwrap();
          }
        }
      }

      Err(e) => println!("Failed to receive :(! error: {}", e)
    }
    println!("going again!");
  }
}

fn main() {
  let mut stdin = std::io::stdin();

  println!("Enter ip address!");
  let mut ipstr = String::new();
  stdin.read_line(&mut ipstr).unwrap();

  ipstr.pop();
  start_server(ipstr.as_ref());
}


// NOTE This MUST be run before any server tests!
#[test]
fn start_test_server() {
  thread::spawn(move || {
    start_server("127.0.0.1");
  });
}

#[test]
fn server_can_receive_packet() {
  server_test!((socket, host_addr) {
    let mut buf = [0u8; 256];
    buf[0] = 90;
    unsafe { socket.send_to(transmute(buf.as_ref()), host_addr).unwrap(); };
  });
}

#[test]
fn server_can_be_pinged() {
  server_test!((socket, host_addr) {
    unsafe {
      let mut buf = [1u8];
      socket.send_to(transmute(buf.as_ref()), host_addr).unwrap();

      let mut in_buf = [0u8, 256];
      match socket.recv_from(&mut in_buf) {
        Ok((len, src_addr)) => {
          let ms = *transmute::<_, *const i32>(&in_buf[0]);
          println!("Got a ping of {} ms!", ms);
        }

        Err(e) => panic!("Got error {} when trying to ping!", e)
      }
    }
  });
}
