pub const fn usize_u32_saturating_cast(x: usize) -> u32 {
  return if (x as u32) as usize == x { x as u32 } else { u32::MAX };
}
