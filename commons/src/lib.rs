pub mod graphics;
pub mod math;

pub fn add_u32(value: u32, delta: i32) -> Option<u32> {
    let new_value = value as i32 + delta;
    if new_value < 0 {
        None
    } else {
        Some(new_value as u32)
    }
}
