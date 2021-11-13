use image::{Rgba, Pixel, DynamicImage, GenericImageView, GenericImage};
use std::collections::{BTreeMap};
use crate::image_utils::rgb_diff;

#[derive(Copy, Clone, Debug)]
struct RGBA {
    r: u64,
    g: u64,
    b: u64,
    p: u64,
    total_record: u64,
}

impl RGBA {
    pub fn new() -> RGBA {
        RGBA {
            r: 0,
            g: 0,
            b: 0,
            p: 0,
            total_record: 0,
        }
    }

    pub fn set_val(&mut self, val: Rgba<u8>) {
        self.total_record += 1;
        self.r += val[0] as u64;
        self.g += val[1] as u64;
        self.b += val[2] as u64;
        self.p += val[3] as u64;
    }

    pub fn avg_rgb(&self) -> Rgba<u8> {
        Rgba::from_channels(
            (self.r / self.total_record) as u8,
            (self.g / self.total_record) as u8,
            (self.b / self.total_record) as u8,
            (self.p / self.total_record) as u8,
        )
    }
}

pub(crate) fn avg(input: &Vec<DynamicImage>) -> DynamicImage {
    let mut width_total: u64 = 0;
    let mut height_total: u64 = 0;
    for img in input {
        width_total += img.width() as u64;
        height_total += img.height() as u64;
    }
    let width = (width_total / input.len() as u64) as u32;
    let height = (height_total / input.len() as u64) as u32;
    // println!("width = {}, height = {}", width, height);

    let mut points = vec![vec![RGBA::new(); height as usize]; width as usize];

    for img in input {
        let mut fixed = img.clone();
        if (img.width() != width) || (img.height() != height) {
            fixed = fixed.thumbnail(width, height);
        }
        for i in 0..width {
            for j in 0..height {
                let val = fixed.get_pixel(i as u32, j as u32);
                points[i as usize][j as usize].set_val(val);
            }
        }
    }

    let mut first_avg_img = vec![vec![Rgba::from([0, 0, 0, 0]); height as usize]; width as usize];

    for i in 0..width {
        for j in 0..height {
            first_avg_img[i as usize][j as usize] = points[i as usize][j as usize].avg_rgb();
        }
    }

    let mut output: DynamicImage = DynamicImage::new_rgba8(width as u32, height as u32);

    for i in 0..width {
        for j in 0..height {
            let mut top_point: BTreeMap<u64, Rgba<u8>> = BTreeMap::new();
            let mut index = 0;
            for img in input {
                let rgb_diff = rgb_diff(img.get_pixel(i, j), first_avg_img[i as usize][j as usize]);
                top_point.insert((((rgb_diff as u64) << 32) + index) as u64, img.get_pixel(i, j));
                index += 1;
            }
            let avg_point_size = (input.len() as f64 * 0.85) as u32;
            let mut avg_point_index = 0;
            let mut rgba = RGBA::new();
            for key in top_point.keys() {
                rgba.set_val(*top_point.get(key).unwrap());
                avg_point_index += 1;
                if avg_point_index >= avg_point_size {
                    break;
                }
            }
            output.put_pixel(i, j, rgba.avg_rgb());
        }
    }
    output
}


#[cfg(test)]
mod tests {
    use crate::image_avg_merger::{RGBA, avg};
    use image::{Rgba, Pixel};

    #[test]
    fn test_rgba() {
        let mut rgba = RGBA::new();
        rgba.set_val(Rgba::from_channels(123, 123, 0, 0));
        println!("{}", rgba.r)
    }


    #[test]
    fn test_avg() {
        let mut input = vec![];
        let img = image::open("./src/images/0.jpg").unwrap();
        input.push(img);
        let img = image::open("./src/images/1.jpg").unwrap();
        input.push(img);
        let img = image::open("./src/images/2.jpg").unwrap();
        input.push(img);
        let img = image::open("./src/images/3.jpg").unwrap();
        input.push(img);
        let output = avg(&input);
        output.save("./src/output.jpg").unwrap();
    }
}