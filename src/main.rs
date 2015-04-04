#![feature(ip_addr, std_misc)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::string::String;
use std::convert::AsRef;
use std::thread;
use std::ptr;
use std::mem::{zeroed, transmute};

pub mod ops;
use ops::{in_op, out_op};

#[macro_use]
#[cfg(debug_assertions)]
pub mod testing;
#[cfg(debug_assertions)]
pub mod tests;

#[derive(Clone, Copy)]
struct Player {
  id: i16,
  last_received_packet_at: u64, // In nanoseconds
}

#[derive(Clone, Copy)]
struct Game<'a> {
  id: u8,
  name: &'a str,
  players: [Player; 10]
}

impl<'a> Game<'a> {
  fn new(id: u8, name: &'a str) -> Game<'a> {
    Game {
      id: id,
      name: name,
      players: unsafe { zeroed() }
    }
  }
}

const MAX_GAMES: usize = 20;

fn error_response(socket: &UdpSocket, addr: SocketAddr, message: &str) {
  unsafe {
    let message_buf: &[u8] = transmute(message);
    let mut buf = Vec::with_capacity(message_buf.len() + 1);
    buf[0] = out_op::ERROR;
    ptr::copy_nonoverlapping(&message_buf[0], &mut buf[1], message_buf.len());

    socket.send_to(&buf, addr).unwrap();
  }
}

fn start_server(ip: &str) {
  let ip_addr = Ipv4Addr::from_str(ip).unwrap();
  let addr = SocketAddr::new(IpAddr::V4(ip_addr), 34561);

  let socket = match UdpSocket::bind(addr) {
    Ok(s) => s,
    Err(e) => panic!("Failed to bind socket! {}", e)
  };

  println!("I survived! (now listening??)");

  let mut buf = [0u8; 256];
  let mut games = [None; MAX_GAMES];
  loop {
    match socket.recv_from(&mut buf) {
      Err(e) => println!("Failed to receive :(! error: {}", e),

      Ok((len, src_addr)) => {

        // ======================= BASIC PING =================
        if len == 1 && buf[0] == 1 {
          println!("ping!!!");
          let response = [1u8];
          socket.send_to(&response, src_addr).unwrap();
          continue;
        }

        // ==================== OTHER OPERATIONS ==============
        match buf[0] {
          in_op::NEW_GAME => {
            let mut index = -1;
            // TODO grab the requested game name, then check if that name exists.
            // (using this loop)
            for i in 0..MAX_GAMES {
              match games[i] {
                Some(_) => continue,
                None => {
                  index = i;
                  break;
                }
              }
            }

            // TODO inform the client that we have no more room instead of whining.
            if index < 0 { println!("Ran out of room for games!!!!!!!!!"); continue; }

            // TODO actually read packet for name lol.
            games[index] = Some(Game::new(index as u8, "who cares"));

            // TODO accept a packet id so that the client can know what responses mean.
            // (maybe)
            let response = [out_op::GAME_CREATED];
            socket.send_to(&response, src_addr).unwrap();

            println!("MADE GAME");
          }

          _ => {
            // TODO send back an UNKNOWN_OP op
            println!("Unknown opcode {} in a {}-byte packet from {}", buf[0], len, src_addr);
          }
        }
      }
    }
    println!("going again!");
  }
}

#[allow(dead_code)]
fn main() {
  let mut stdin = std::io::stdin();

  println!("Enter ip address!");
  let mut ipstr = String::new();
  stdin.read_line(&mut ipstr).unwrap();

  ipstr.pop();
  start_server(ipstr.as_ref());
}


#[test]
fn a0_start_test_server() {
  thread::spawn(move || {
    start_server("127.0.0.1");
  });
}


