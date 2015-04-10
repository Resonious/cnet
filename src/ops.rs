use std::ptr;
use std::mem::{size_of, transmute};
use std::net::UdpSocket;
use std::net::{SocketAddr};

pub mod in_op {
  pub const NEW_GAME: u8 = 2;
}
pub mod out_op {
  pub const ERROR: u8 = 0;
  pub const GAME_CREATED: u8 = 2;
}

pub struct Packet<'b> {
  pub buf: &'b mut [u8],
  pub pos: usize
}

impl<'b> Packet<'b> {
  pub fn ptr(&self) -> &u8 {
    &self.buf[self.pos]
  }

  pub fn push<T>(&mut self, data: &T) {
    unsafe {
      ptr::copy_nonoverlapping::<T>(data, transmute(&mut self.buf[self.pos]), 1);
    }
    self.pos += size_of::<T>();
  }

  pub fn send_to(&self, socket: &UdpSocket, addr: SocketAddr) {
    socket.send_to(&self.buf[0..self.pos], addr).unwrap();
  }
}

// TODO turn this into a method
#[macro_export]
macro_rules! pull(
  ($packet:expr, $bytes:expr) => ({
    let pos = $packet.pos;
    $packet.pos += $bytes;
    unsafe { &$packet.buf[pos..pos + $bytes] }
  });

  ($packet:expr => $t:ty) => ({
    let pos = $packet.pos;
    let size = size_of::<$t>();
    $packet.pos += size;
    unsafe { *(transmute::<_, &$t>(&$packet.buf[pos..pos + size][0])) }
  });
);

// TODO make push! or a method I dunno.
