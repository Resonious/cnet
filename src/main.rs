#![feature(ip_addr, std_misc, convert)]

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::str;
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
struct Game {
  id: u8,
  name: &'static str,
  players: [Player; 10]
}

impl Game {
  fn new(id: u8, name: &'static str) -> Game {
    Game {
      id: id,
      name: name,
      players: unsafe { zeroed() }
    }
  }
}

const MAX_GAMES: usize = 20;
const MAX_GAME_NAME_SIZE: usize = 15;

fn error_response(socket: &UdpSocket, addr: SocketAddr, message: &str) {
  unsafe {
    let message_buf: &[u8] = transmute(message);
    let buf_len = message_buf.len() + 1;

    // TODO perhaps instead of allocating here, keep ahold of an out_buf or something
    // and pass it here.

    let mut buf = Vec::with_capacity(buf_len);
    buf.set_len(buf_len);
    let mut buf = buf.into_boxed_slice();

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
  let mut game_names = [[0u8; MAX_GAME_NAME_SIZE]; MAX_GAMES];
  let mut games: [Option<Game>; MAX_GAMES] = [None; MAX_GAMES];
  'receiving: loop {
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
            let name_len = buf[1] as usize;
            let name = match str::from_utf8(&buf[2..name_len+2]) {
              Ok(s) => s,
              Err(e) => {
                error_response(&socket, src_addr, "Invalid game name");
                continue 'receiving;
              }
            };
            println!("Making a game with a name of {}", name);

            let mut index: isize = -1;
            for i in 0..MAX_GAMES {
              match games[i] {
                Some(game) => {
                  println!("CHECKING IF \"{}\" == \"{}\"", game.name, name);
                  if game.name == name {
                    let msg = format!("Game name {} already taken", name);
                    error_response(&socket, src_addr, msg.as_str());
                    continue 'receiving;
                  }
                }

                None => {
                  index = i as isize;
                  break;
                }
              }
            }

            if index < 0 {
              error_response(&socket, src_addr, "Server is too full to accept another game");
              continue 'receiving;
            }
            let index = index as usize;

            unsafe {
              ptr::copy_nonoverlapping(&buf[2], &mut game_names[index][0], name_len);
              games[index] = Some(
                Game::new(
                  index as u8,
                  transmute::<_, &'static str>(&game_names[index][0..name_len]))
              );
            }

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


