use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::thread;
use std::ptr;
use std::str;
// use std::mem::transmute;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::duration::Duration;

use ops::{in_op, out_op, Packet};

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
    let buf = [1u8];
    socket.send_to(&buf, host_addr).unwrap();

    let mut packet = [0u8; 256];
    let ms = Duration::span(|| {
      match socket.recv_from(&mut packet) {
        Ok((len, _src_addr)) => {
          assert!(len == 1,
                  "Length of ping return packet was not 1, it was {}", len);
          assert!(packet[0] == 1,
                  "First byte of ping return packet was not 1, it was {}", packet[0]);
        }

        Err(e) => panic!("Got error {} when trying to ping!", e)
      }
    });
    println!("Ping took {} ms!", ms.num_milliseconds());
  });
}

macro_rules! create_game(
  ($socket:expr, $host_addr:expr, $name:expr, $packet_id:expr
      => ($packet:ident, $len:ident, $src_addr:ident) $logic:block) => ({
    let mut buf = [0u8; 128];
    // u16 packet id
    unsafe {
      let packet_id = $packet_id;
      ptr::copy_nonoverlapping(&packet_id, &mut buf[0], 2);
    }
    // u8 opcode
    buf[2] = in_op::NEW_GAME;
    let name = $name;

    buf[3] = name.len() as u8;
    unsafe {
      ptr::copy_nonoverlapping(&name[0], &mut buf[4], name.len());
    }

    $socket.send_to(&buf, $host_addr).unwrap();

    let mut packet = [0u8; 256];
    match $socket.recv_from(&mut packet) {
      Ok((len, src_addr)) => {
        let mut logic = |$packet: &mut[u8], $len: usize, $src_addr: SocketAddr| $logic;
        logic(&mut packet, len, src_addr);
      }

      Err(e) => panic!("Got error {} when trying to create game!", e)
    }
  });

  ($socket:expr, $host_addr:expr, $name:expr
      => ($packet:ident, $len:ident, $src_addr:ident) $logic:block) => ({
    create_game!($socket, $host_addr, $name, 0 => ($packet, $len, $src_addr) $logic);
  });
);

macro_rules! join_game(
  ($socket:expr, $host_addr:expr, $name:expr, $packet_id:expr
      => ($packet:ident, $len:ident, $src_addr:ident) $logic:block) => ({
    let mut buf = [0u8; 128];
    let mut request = Packet::new(&mut buf);
    // packet id
    request.push::<u16>(&$packet_id);
    // opcode
    request.push::<u8>(&in_op::JOIN_GAME);
    // game name
    let name = $name;
    request.push(&(name.len() as u8));
    request.push_slice(name);

    request.send_to(&$socket, $host_addr);

    let mut response_buf = [0u8; 256];
    match $socket.recv_from(&mut response_buf) {
      Ok((len, src_addr)) => {
        let logic = |$packet: &mut[u8], $len: usize, $src_addr: SocketAddr| $logic;
        logic(&mut response_buf, len, src_addr);
      }

      Err(e) => panic!("Got error {} when trying to join game!", e)
    }
  });
);

macro_rules! assert_packet_id(
  ($got:expr, $expected:expr) => (
    assert!($expected == $got, "Packet id mismatch! Expected {}, got {}", $expected, $got);
  );
);

#[test]
fn server_cannot_create_2_games_with_the_same_name() {
  server_test!((socket, host_addr) {
    create_game!(socket, host_addr, b"Stupid game" => (packet, len, _src_addr) {
      assert_eq!(packet[0], 0);
      assert_eq!(packet[1], 0);
      assert!(packet[2] == out_op::GAME_CREATED,
              "Returned opcode was not GAME_CREATED, rather {}", packet[0]);
    });

    create_game!(socket, host_addr, b"Stupid game" => (packet, len, _src_addr) {
      let mut response = Packet::new(packet);
      let packet_id = response.pull::<u16>();
      assert_eq!(packet_id, 0);
      let opcode = response.pull::<u8>();
      assert!(opcode == out_op::ERROR,
              "Returned opcode was not ERROR, rather {}", opcode);

      assert!(len > 3, "No error message returned. packet length: {}", len);

      let msg_len = response.pull::<u16>() as usize;
      assert!(msg_len > 1, "Message should have a length greater than 1; got {}", msg_len);
      let msg = unsafe {
        match str::from_utf8(response.peek_slice(msg_len)) {
          Ok(s) => s,
          Err(_) => panic!("Error message was not correct utf8 format")
        }
      };
      println!("Error message: {}", msg);
    });
  });
}

#[test]
fn server_gets_a_player_id_or_something_on_game_create() {
  server_test!((socket, host_addr) {
    create_game!(socket, host_addr, b"THE GAME", 12 => (buf, len, _src_addr) {
      let mut packet = Packet::new(buf);
      let packet_id: u16 = packet.pull();
      assert_packet_id!(packet_id, 12);
      assert_eq!(packet.pull::<u8>(), out_op::GAME_CREATED);

      assert!(packet.has::<i16>(len), "Packet does not have an i16 player ID after opcode");
      let player_id: u16 = packet.pull();
      println!("Got a player ID of {}!", player_id);
    });
  });
}

#[test]
fn player_can_join_game() {
  server_test!((socket, host_addr) {
    let mut host_player_id: u16 = 0;

    create_game!(socket, host_addr, b"join me link", 13 => (buf, _len, _src_addr) {
      let mut packet = Packet::new(buf);
      let packet_id: u16 = packet.pull();
      assert_packet_id!(packet_id, 13);
      assert_eq!(packet.pull::<u8>(), out_op::GAME_CREATED);

      host_player_id = packet.pull();
    });

    join_game!(socket, host_addr, b"join me link", 14 => (buf, len, src_addr) {
      let mut packet = Packet::new(buf);
      let packet_id: u16 = packet.pull();
      assert_packet_id!(packet_id, 14);
      assert_eq!(packet.pull::<u8>(), out_op::GAME_JOINED);

      // TODO expect a player list or something?
    });
  });
}
