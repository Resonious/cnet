#![feature(ip_addr, std_misc)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::string::String;
use std::convert::AsRef;
use std::thread;
use std::mem::{zeroed, transmute};
use std::ptr;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::duration::Duration;
use std::time;

#[macro_use]
mod testing;

#[derive(Clone, Copy)]
struct Player {
  // assigned, given
  id: (i8, i8),
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

mod in_op {
  pub const NEW_GAME: u8 = 2;
}
mod out_op {
  pub const GAME_CREATED: u8 = 2;
}
const MAX_GAMES: usize = 20;

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


// =========================================== TESTS ========================================

// NOTE This MUST be run before any server tests!
#[test]
fn a0_start_test_server() {
  thread::spawn(move || {
    start_server("127.0.0.1");
  });
}

#[test]
fn server_can_receive_packet() {
  server_test!((socket, host_addr) {
    let mut buf = [0u8; 256];
    buf[0] = 90;
    socket.send_to(&buf, host_addr).unwrap();
  });
}

#[test]
fn server_can_be_pinged() {
  server_test!((socket, host_addr) {
    let mut buf = [1u8];
    socket.send_to(buf.as_ref(), host_addr).unwrap();

    let mut in_buf = [0u8; 256];
    let ms = Duration::span(|| {
      match socket.recv_from(&mut in_buf) {
        Ok((len, _src_addr)) => {
          assert!(len == 1,
                  "Length of ping return packet was not 1, it was {}", len);
          assert!(in_buf[0] == 1,
                  "First byte of ping return packet was not 1, it was {}", in_buf[0]);
        }

        Err(e) => panic!("Got error {} when trying to ping!", e)
      }
    });
    println!("Ping took {} ms!", ms.num_milliseconds());
  });
}

#[test]
fn server_can_create_a_game() {
  server_test!((socket, host_addr) {
    let mut buf = [0u8; 128];
    buf[0] = in_op::NEW_GAME;
    let name = b"Stupid game";

    buf[1] = name.len() as u8;
    unsafe {
      ptr::copy(&name[0], &mut buf[2], name.len());
    }

    socket.send_to(&buf, host_addr).unwrap();

    let mut in_buf = [0u8; 256];
    match socket.recv_from(&mut in_buf) {
      Ok((len, _src_addr)) => {
        assert!(len == 1, "length was fucking {}", len);
        assert!(in_buf[0] == out_op::GAME_CREATED,
                "Returned opcode was not GAME_CREATED, rather {}", in_buf[0]);
      }

      Err(e) => panic!("Got error {} when trying to create game!", e)
    }
  });
}
