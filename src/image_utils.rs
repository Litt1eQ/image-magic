use image::{Rgba};

pub fn rgb_diff(left: Rgba<u8>, right: Rgba<u8>) -> i32 {
    (left[0] as i32 - right[0] as i32).abs() + (left[1] as i32 - right[1] as i32).abs() + (left[2] as i32 - right[2] as i32).abs()
}

// pub fn mask_merge(rgb_left: i32, rgb_right: i32, left_ratio: f32) -> i32 {
//     let r: u32 = ((((rgb_left as u32) >> 24) & 0xFF) as f32 * left_ratio + ((rgb_right as u32) >> 24) as f32 * (1.0 - left_ratio)) as u32;
//     let g: u32 = ((((rgb_left as u32) >> 16) & 0xFF) as f32 * left_ratio + ((rgb_right as u32) >> 16) as f32 * (1.0 - left_ratio)) as u32;
//     let b: u32 = ((((rgb_left as u32) >> 8) & 0xFF) as f32 * left_ratio + ((rgb_right as u32) >> 8) as f32 * (1.0 - left_ratio)) as u32;
//     let p: u32 = ((((rgb_left as u32) >> 0) & 0xFF) as f32 * left_ratio + ((rgb_right as u32) >> 0) as f32 * (1.0 - left_ratio)) as u32;
//     return (r.wrapping_shl(24) | g.wrapping_shl(16) | b.wrapping_shl(8) | p) as i32;
// }


#[cfg(test)]
mod test {
    #[test]
    fn test_rgb_diff() {}

    #[test]
    fn test_mark_merge() {}
}