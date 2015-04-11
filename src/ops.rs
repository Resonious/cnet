use std::ptr;
use std::mem::{size_of, transmute, uninitialized};
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
  pub fn new(buf: &mut [u8]) -> Packet {
    Packet { buf: buf, pos: 0 }
  }

  pub fn ptr(&self) -> &u8 {
    &self.buf[self.pos]
  }

  pub fn push<T>(&mut self, data: &T) {
    unsafe {
      ptr::copy_nonoverlapping::<T>(data, transmute(&mut self.buf[self.pos]), 1);
    }
    self.pos += size_of::<T>();
  }

  pub fn pull<T>(&mut self) -> T {
    unsafe {
      let mut value: T = uninitialized();
      ptr::copy_nonoverlapping::<T>(transmute(&self.buf[self.pos]), &mut value, 1);
      self.pos += size_of::<T>();
      value
    }
  }

  // Doesn't actually copy anything like pull does
  pub unsafe fn peek<T>(&mut self) -> &T {
    let pos = self.pos;
    self.pos += size_of::<T>();
    transmute::<_, &T>(&self.buf[pos])
  }

  // pub fn pull_slice(&mut self, bytes: usize) -> Box<[u8]> {
    // TODO implement this if necessary
  // }

  // Doesn't actually copy (or allocate) anything like pull_slice does
  pub unsafe fn peek_slice(&mut self, bytes: usize) -> &'static [u8] {
    let pos = self.pos;
    self.pos += bytes;
    transmute(&self.buf[pos..pos+bytes])
  }

  pub fn has<T>(&self, size: usize) -> bool {
    size - self.pos >= size_of::<T>()
  }

  pub fn send_to(&self, socket: &UdpSocket, addr: SocketAddr) {
    socket.send_to(&self.buf[0..self.pos], addr).unwrap();
  }
}
