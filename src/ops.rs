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
}

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
