use std::net::UdpSocket;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::thread;
use std::ptr;
use std::mem::transmute;
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
        let logic = |$packet: &[u8], $len: usize, $src_addr: SocketAddr| $logic;
        logic(&packet, len, src_addr);
      }

      Err(e) => panic!("Got error {} when trying to create game!", e)
    }
  });

  ($socket:expr, $host_addr:expr, $name:expr
      => ($packet:ident, $len:ident, $src_addr:ident) $logic:block) => ({
    create_game!($socket, $host_addr, $name, 0 => ($packet, $len, $src_addr) $logic);
  });
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
      assert_eq!(packet[0], 0);
      assert_eq!(packet[1], 0);
      assert!(packet[2] == out_op::ERROR,
              "Returned opcode was not ERROR, rather {}", packet[2]);
    });
  });
}

#[test]
fn server_gets_a_player_id_or_something_on_game_create() {
  server_test!((socket, host_addr) {
    create_game!(socket, host_addr, b"THE GAME", 12 => (buf, len, _src_addr) {
      let mut packet = unsafe { Packet { buf: transmute(buf), pos: 0 } };
      let packet_id: u16 = packet.pull();
      assert_eq!(packet_id, 12);
      assert_eq!(packet.pull::<u8>(), out_op::GAME_CREATED);

      // TODO look for player id
    });
  });
}
