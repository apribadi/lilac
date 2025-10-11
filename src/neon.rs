pub use core::arch::aarch64;
pub use core::arch::aarch64::uint8x16_t;
pub use core::arch::aarch64::uint8x16x4_t;
pub use core::arch::aarch64::vqtbl4q_u8;
pub use core::arch::aarch64::vld1q_u8;
pub use core::arch::aarch64::vld1q_u8_x4;
pub use core::arch::aarch64::vdupq_n_u8;
pub use core::arch::aarch64::veorq_u8;

pub struct uint8x16x8_t(uint8x16x4_t, uint8x16x4_t);

pub unsafe fn vld1q_u8_x8(a: *const u8) -> uint8x16x8_t {
  let x = unsafe { vld1q_u8_x4(a) };
  let y = unsafe { vld1q_u8_x4(a.wrapping_add(64)) };
  return uint8x16x8_t(x, y);
}

#[target_feature(enable = "neon")]
pub fn vqtbl8q_u8(a: uint8x16x8_t, b: uint8x16_t) -> uint8x16_t {
  let x = vqtbl4q_u8(a.0, b);
  let y = vqtbl4q_u8(a.1, veorq_u8(b, vdupq_n_u8(0b01000000)));
  return veorq_u8(x, y); 
}
