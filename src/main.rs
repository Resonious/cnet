#![feature(ip_addr, std_misc, convert)]

extern crate time;

use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::str;
use std::string::String;
use std::convert::AsRef;
use std::thread;
use std::ptr;
use std::mem::{zeroed, transmute, size_of};
use time::precise_time_ns;

#[macro_use]
pub mod ops;
use ops::{in_op, out_op, Packet};

#[macro_use]
#[cfg(debug_assertions)]
pub mod testing;
#[cfg(debug_assertions)]
pub mod tests;

#[derive(Clone, Copy)]
struct Player {
  id: u16,
  last_received_packet_at: u64, // In nanoseconds
}

impl Player {
  fn new(game_id: u8, id: u8) -> Player {
    let id_parts = [game_id, id];
    Player {
      id: unsafe { *transmute::<_, &u16>(&id_parts[0]) },
      last_received_packet_at: precise_time_ns()
    }
  }
}

#[derive(Clone, Copy)]
struct Game {
  id: u8,
  name: &'static str,
  players: [Option<Player>; 10]
}

impl Game {
  fn new(id: u8, name: &'static str) -> Game {
    let mut game = Game {
      id: id,
      name: name,
      players: [None; 10]
    };
    game.players[0] = Some(Player::new(id, 0));
    game
  }
}

const MAX_GAMES: usize = 20;
const MAX_GAME_NAME_SIZE: usize = 15;

struct Server<'a> {
  socket:     &'a UdpSocket,

  game_names: &'a mut [[u8; MAX_GAME_NAME_SIZE]; MAX_GAMES],
  games:      &'a mut [Option<Game>; MAX_GAMES],
}

struct PacketInfo<'a, 'b> {
  src_addr:  SocketAddr,
  packet_id: u16,
  opcode:    u8,
  len:       usize,
  request:   &'a mut Packet<'a>,
  response:  &'b mut Packet<'b>
}

impl<'a, 'b> PacketInfo<'a, 'b> {
  pub fn packets(&mut self) -> (&'static mut Packet<'static>, &'static mut Packet<'static>) {
    // HACK holy hell
    unsafe {
      let mut request = transmute::<_, *mut usize>(&self.request);
      let mut response = transmute::<_, *mut usize>(&self.response);
      (transmute(*request), transmute(*response))
    }
  }
}

fn error_response(server: &Server, packet: &mut PacketInfo, message: &str) {
  unsafe {
    let message_buf: &[u8] = transmute(message);
    let message_len = message_buf.len() as u16;

    let ref mut buf = packet.response;
    buf.push(&packet.packet_id);
    buf.push(&out_op::ERROR);
    buf.push(&message_len);
    buf.push_slice(message_buf);

    buf.send_to(server.socket, packet.src_addr);
    // socket.send_to(&buf[0..buf_len], addr).unwrap();
  }
}

fn unknown_op(server: &mut Server, packet: &mut PacketInfo) {
  println!("Unknown opcode {} from {}", packet.opcode, packet.src_addr);
}

fn create_game(server: &mut Server, packet: &mut PacketInfo) {
  let (mut request, mut response) = packet.packets();

  let name_len = request.pull::<u8>() as usize;

  if name_len > MAX_GAME_NAME_SIZE {
    let msg = format!("Game name too long! Must be <= {} characters.", MAX_GAME_NAME_SIZE);
    error_response(server, packet, msg.as_str());
    return;
  }

  let name_pos = request.pos;
  let name = unsafe {
    match str::from_utf8(request.peek_slice(name_len)) {
      Ok(s) => s,
      Err(_) => {
        error_response(server, packet, "Invalid game name");
        return;
      }
    }
  };

  let mut index: isize = -1;
  for i in 0..MAX_GAMES {
    match server.games[i] {
      Some(game) => {
        println!("CHECKING IF \"{}\" == \"{}\"", game.name, name);
        if game.name == name {
          let msg = format!("Game name {} already taken", name);
          error_response(server, packet, msg.as_str());
          return;
        }
      }

      None => {
        index = i as isize;
        break;
      }
    }
  }

  if index < 0 {
    error_response(server, packet, "Server is too full to accept another game");
    return;
  }
  let index = index as usize;

  unsafe {
    ptr::copy_nonoverlapping(&request.buf[name_pos], &mut server.game_names[index][0], name_len);
    server.games[index] = Some(
      Game::new(
        index as u8,
        // here we trust that game_names[index] will not change
        transmute::<_, &'static str>(&server.game_names[index][0..name_len])
      )
    );
  }
  let player = server.games[index].unwrap().players[0].unwrap();

  response.push(&packet.packet_id);
  response.push(&out_op::GAME_CREATED);
  response.push(&player.id);
  response.send_to(&server.socket, packet.src_addr);

  println!("MADE GAME");

}

fn start_server(ip: &str) {
  let ip_addr = Ipv4Addr::from_str(ip).unwrap();
  let addr = SocketAddr::new(IpAddr::V4(ip_addr), 34561);

  let socket = match UdpSocket::bind(addr) {
    Ok(s) => s,
    Err(e) => panic!("Failed to bind socket! {}", e)
  };

  println!("I survived! (now listening??)");

  let mut in_buf = [0u8; 1024];
  let mut out_buf = [0u8; 1024];

  let mut game_names = [[0u8; MAX_GAME_NAME_SIZE]; MAX_GAMES];
  let mut games: [Option<Game>; MAX_GAMES] = [None; MAX_GAMES];

  let mut server = Server {
    socket: &socket,

    game_names: &mut game_names,
    games: &mut games
  };

  let create_game = create_game;
  let mut operations = [None; 255];
  operations[in_op::NEW_GAME as usize] = Some(&create_game);

  'receiving: loop {
    match socket.recv_from(&mut in_buf) {
      Err(e) => println!("Failed to receive :(! error: {}", e),

      Ok((len, src_addr)) => {

        // ======================= BASIC PING =================
        if len == 1 && in_buf[0] == 1 {
          println!("ping!!!");
          let response = [1u8];
          socket.send_to(&response, src_addr).unwrap();
          continue 'receiving;
        }

        // ==================== OTHER OPERATIONS ==============
        if len < 3 {
          println!("Packet too short to be anything!");
          continue 'receiving;
        }

        let mut packet   = Packet { buf: &mut in_buf,  pos: 0 };
        let mut response = Packet { buf: &mut out_buf, pos: 0 };

        let packet_id: u16 = packet.pull();
        let opcode:    u8  = packet.pull();

        println!("\n********* processing packet id {} opcode {}", packet_id, opcode);
        {
          let mut packet_info = PacketInfo {
            src_addr: src_addr,
            packet_id: packet_id,
            opcode: opcode,
            len: len,
            request: &mut packet,
            response: &mut response
          };
          match operations[opcode as usize] {
            Some(function) => function(&mut server, &mut packet_info),
            None           => unknown_op(&mut server, &mut packet_info)
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


