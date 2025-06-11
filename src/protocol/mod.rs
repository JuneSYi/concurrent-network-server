// constants
pub const HANDSHAKE:    u8 = b'*';
pub const START_MARKER: u8 = b'^';
pub const END_MARKER:   u8 = b'$';
pub const PRIME_CMD:    u8 = b'P';

// returns true if this byte is the '^' start marker
#[inline]
pub fn is_start(b: u8) -> bool { b == START_MARKER }

// returns true if this byte is the '$' end marker
#[inline]
pub fn is_end(b: u8) -> bool { b == END_MARKER }

// transforms a byte according to this simple protocol implementation (add 1 mod 256)
#[inline]
pub fn transform(b: u8) -> u8 { b.wrapping_add(1) }