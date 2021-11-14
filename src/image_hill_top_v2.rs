use pyo3::{prelude::*};

use std::f64::consts::{SQRT_2, PI};
use image::{DynamicImage, GenericImageView, ImageBuffer, GrayImage};
use crate::image_utils::rgb_diff;
use std::cmp::{min, max};

#[pyclass]
#[derive(Copy, Clone, Debug)]
pub struct Point {
    x: usize,
    y: usize,
    weight: usize,
}

impl Point {
    fn new(x: usize, y: usize, weight: usize) -> Point {
        Point { x, y, weight }
    }
}

#[pymethods]
impl Point {
    pub fn get_x(&self) -> PyResult<u32> {
        PyResult::Ok(self.x as u32)
    }

    pub fn get_y(&self) -> PyResult<u32> {
        PyResult::Ok(self.y as u32)
    }
}

pub struct HilltopParamAndResult {
    background_image: DynamicImage,
    challenge_image: DynamicImage,
    ch_size: u32,
    top_n: usize,
    avg_diff: u32,
}

impl HilltopParamAndResult {
    pub fn new(background_image: DynamicImage, challenge_image: DynamicImage,
               ch_size: u32, top_n: usize) -> HilltopParamAndResult {
        HilltopParamAndResult {
            background_image,
            challenge_image,
            ch_size,
            top_n,
            avg_diff: 0,
        }
    }
}

struct XY {
    x: usize,
    y: usize,
    weight: u64,
}

impl XY {
    pub fn new() -> XY {
        XY {
            x: 0,
            y: 0,
            weight: 0,
        }
    }
    pub fn update(&mut self, x: usize, y: usize, weight: u64) {
        if weight > self.weight {
            self.x = x;
            self.y = y;
            self.weight = weight;
        }
    }
}

struct AggregateMountain {
    diff_data: Vec<Vec<u64>>,
    width: usize,
    height: usize,
    is_last: bool,
    next: Option<Box<AggregateMountain>>,
}

impl AggregateMountain {
    fn new(diff_data: Vec<Vec<u64>>, width: usize, height: usize) -> AggregateMountain {
        AggregateMountain {
            diff_data,
            width,
            height,
            is_last: false,
            next: None,
        }
    }

    pub fn fetch_top_point(&self) -> XY {
        if self.is_last {
            let mut xy = XY::new();
            for i in 0..self.width {
                for j in 0..self.height {
                    xy.update(i, j, self.diff_data[i][j]);
                }
            }
            return xy;
        }
        let next_xy = self.next.as_ref().unwrap().fetch_top_point();
        let start_x = next_xy.x * 5;
        let end_x = min(next_xy.x * 5 + 4, self.width - 1);
        let start_y = next_xy.y * 5;
        let end_y = min(next_xy.y * 5 + 4, self.height - 1);

        let mut xy = XY::new();
        for i in start_x..=end_x {
            for j in start_y..=end_y {
                xy.update(i, j, self.diff_data[i][j]);
            }
        }
        return xy;
    }

    pub fn invalid_rectangle(&mut self, left_top_x: usize, left_top_y: usize, right_bottom_x: usize, right_bottom_y: usize) {
        if self.is_last {
            return;
        }
        let mut next_start_x = left_top_x / 5;
        let mut next_start_y = left_top_y / 5;

        let mut next_end_x = (right_bottom_x + 4) / 5;
        let mut next_end_y = (right_bottom_y + 4) / 5;


        if left_top_x % 5 != 0 {
            if next_start_x < 1 {
                next_start_x = 0;
            }
        }

        if left_top_y % 5 != 0 {
            if next_start_y < 1 {
                next_start_y = 0;
            }
        }

        if right_bottom_x % 5 != 0 {
            next_end_x = min(next_end_x + 1, self.next.as_ref().unwrap().width - 1);
        }

        if right_bottom_y % 5 != 0 {
            next_end_y = min(next_end_y + 1, self.next.as_ref().unwrap().height - 1);
        }

        // fill in next diff data
        for x in next_start_x..=next_end_x {
            for y in next_start_y..=next_end_y {
                let scan_start_x = x * 5;
                let scan_start_y = y * 5;

                let scan_end_x = min(scan_start_x + 4, self.width - 1);
                let scan_end_y = min(scan_start_y + 4, self.height - 1);
                // let center_x = (scan_start_x + scan_end_x) / 2;
                // let center_y = (scan_start_y + scan_end_y) / 2;

                let mut aggregate_diff = 0;
                for next_x in scan_start_x..=scan_end_x {
                    for next_y in scan_start_y..=scan_end_y {
                        aggregate_diff += self.diff_data[next_x][next_y];
                    }
                }
                self.next.as_mut().unwrap().diff_data[x][y] = aggregate_diff;
            }
            self.next.as_mut().unwrap().invalid_rectangle(next_start_x, next_start_y, next_end_x, next_end_y);
        }
    }

    pub fn gen_aggregate_mountain_mapping(&mut self) {
        if self.width < 5 || self.height < 5 {
            self.is_last = true;
            return;
        }
        let next_width = (self.width + 4) / 5;
        let next_height = (self.height + 4) / 5;
        let next_data = vec![vec![0; next_height]; next_width];

        self.next = Option::from(Box::new(AggregateMountain::new(next_data, next_width, next_height)));

        self.next.as_mut().unwrap().gen_aggregate_mountain_mapping();
    }
}

struct Rectangle {
    top_x: usize,
    top_y: usize,
    bottom_x: usize,
    bottom_y: usize,
}

impl Rectangle {
    pub fn rectangle_range(x: usize, y: usize, slice_size: usize, total_width: usize, total_height: usize) -> Rectangle {
        let half_slice_size = slice_size / 2;
        let top_x = if x > half_slice_size {
            x - half_slice_size
        } else {
            0
        };
        let top_y = if y > half_slice_size {
            y - half_slice_size
        } else {
            0
        };
        let mut right_bottom_x = x + half_slice_size;
        let mut right_bottom_y = y + half_slice_size;

        if right_bottom_x >= total_width {
            right_bottom_x = total_width - 1;
        }
        if right_bottom_y >= total_height {
            right_bottom_y = total_height - 1;
        }

        Rectangle {
            top_x,
            top_y,
            bottom_x: right_bottom_x,
            bottom_y: right_bottom_y,
        }
    }
}

pub fn sqrt(x: usize) -> usize {
    let mut a: usize = 1;
    while a * a <= x as usize {
        a = a + 1;
    }
    return (a - 1) as usize;
}

fn _save_image(diff_data: &Vec<Vec<u64>>, width: usize, height: usize) {
    let mut img: GrayImage = ImageBuffer::new(width as u32, height as u32);
    let mut max_diff = 0;
    for i in 0..width {
        for j in 0..height {
            if max_diff < diff_data[i][j] {
                max_diff = diff_data[i][j];
            }
        }
    }
    for i in 0..width {
        for j in 0..height {
            let rgb = diff_data[i][j] * 255 / max_diff;
            let rgb_gray = (rgb << 24 | rgb << 16 | rgb << 8 | rgb) as u8;
            img.put_pixel(i as u32, j as u32, image::Pixel::from_channels(rgb_gray, rgb_gray, rgb_gray, 100));
        }
    }
    // img.save("./src/images/gray_image.png").unwrap();
}


fn adjust_center_point(top_xy: XY, mountain: &AggregateMountain, result: &HilltopParamAndResult, result_width: usize, result_height: usize, result_diff: &Vec<Vec<i32>>) -> XY {
    let points = Rectangle::rectangle_range(top_xy.x, top_xy.y, (result.ch_size * 2) as usize, result_width, result_height);
    let thumb_times = sqrt(result.ch_size as usize) as u32;
    let mut short_curt_width = result.ch_size / thumb_times;
    short_curt_width *= 2;
    let mut short_curt = vec![vec![0; short_curt_width as usize]; short_curt_width as usize];

    for i in 0..short_curt_width {
        for j in 0..short_curt_width {
            let start_x = i * thumb_times + points.top_x as u32;
            let start_y = j * thumb_times + points.top_y as u32;

            let end_x = min(start_x + thumb_times - 1, (result_width - 1) as u32);
            let end_y = min(start_y + thumb_times - 1, (result_height - 1) as u32);

            let mut total_diff = 0u64;

            for x in start_x..=end_x {
                for y in start_y..=end_y {
                    total_diff += result_diff[x as usize][y as usize] as u64;
                }
            }
            short_curt[i as usize][j as usize] = total_diff;
        }
    }

    let short_curt_mountain_width = short_curt_width / 2;

    let mut short_curt_xy = XY::new();
    let mut short_curt_mountain = vec![vec![0; short_curt_mountain_width as usize]; short_curt_mountain_width as usize];
    for i in 0..short_curt_mountain_width {
        for j in 0..short_curt_mountain_width {
            let center_x = i + short_curt_mountain_width / 2;
            let center_y = j + short_curt_mountain_width / 2;
            let rect = Rectangle::rectangle_range(center_x as usize, center_y as usize, short_curt_mountain_width as usize, short_curt_width as usize, short_curt_width as usize);
            let mut aggregate_diff = 0.0;
            for x in rect.top_x..=rect.bottom_x {
                for y in rect.top_y..=rect.bottom_y {
                    let base = short_curt[x][y] as f64;
                    let distance = (((x as f64 - center_x as f64) * (x as f64 - center_x as f64) + (y as f64 - center_y as f64) * (y as f64 - center_y as f64)) as f64).sqrt();
                    let distance_ratio = distance / (SQRT_2 * ((short_curt_mountain_width) / 2) as f64);
                    if distance_ratio > 1.0 {
                        continue;
                    }
                    let ratio = ((PI * distance_ratio).cos() + 1.0) / 2.0;
                    aggregate_diff += base * base * base * ratio;
                }
            }
            short_curt_mountain[i as usize][j as usize] = aggregate_diff as usize;
            short_curt_xy.update(center_x as usize, center_y as usize, aggregate_diff as u64);
        }
    }

    // 在缩略图里面寻找最高点，之后再回放到原图进行
    let real_start_x = short_curt_xy.x as usize * thumb_times as usize + points.top_x;
    let real_end_x = short_curt_xy.x * thumb_times as usize + thumb_times as usize + points.top_x;
    let real_start_y = short_curt_xy.y * thumb_times as usize + points.top_y;
    let real_end_y = short_curt_xy.y * thumb_times as usize + thumb_times as usize + points.top_y;

    let mut xy = XY::new();
    for i in real_start_x..=real_end_x {
        for j in real_start_y..=real_end_y {
            let rect = Rectangle::rectangle_range(i, j, result.ch_size as usize, result_width, result_height);
            let mut aggregate_diff = 0.0;
            for x in rect.top_x..=rect.bottom_x {
                for y in rect.top_y..=rect.bottom_y {
                    let distance = (((x as i64 - i as i64).pow(2) + (y as i64 - j as i64).pow(2)) as f64).sqrt();
                    let distance_ratio = distance / (SQRT_2 * (result.ch_size / 2) as f64);
                    if distance_ratio > 1.0 {
                        continue;
                    }
                    let ratio = ((PI * distance_ratio).cos() + 1.0) / 2.0;
                    aggregate_diff += mountain.diff_data[x][y] as f64 * ratio;
                }
            }
            xy.update(i, j, aggregate_diff as u64);
        }
    }

    xy
}

fn _vec_hash(data: &Vec<Vec<u64>>, width: usize, height: usize) -> f64 {
    let mut hash = 0.0;
    for i in 0..width {
        for j in 0..height {
            hash += data[i][j] as f64;
        }
    }
    hash
}

pub fn find_top_n(mut result: HilltopParamAndResult) -> Vec<Point> {
    // 挑战图的宽和高
    let width = result.challenge_image.width() as usize;
    let height = result.challenge_image.height() as usize;

    // 缩放底图，如果宽和高不一致的话
    let bg_image = result.background_image.clone();
    let cg_image = result.challenge_image.clone();

    // 这里写法好像有点bug, 目前没解决, 就当图都一样大吧，不一样自己用open-cv处理一下, ^.^
    // if (bg_image.width() != result.width) || (bg_image.height() != result.height) {
    //     bg_image = bg_image.thumbnail(result.width, result.height);
    // }

    let mut total_diff = 0u64;
    let mut diff = vec![vec![0; height]; width];
    let mut calculate_diff = vec![vec![0u64; height]; width];

    // 计算背景图和挑战图的像素差
    for i in 0..width {
        for j in 0..height {
            let rgb_diff = rgb_diff(cg_image.get_pixel(i as u32, j as u32), bg_image.get_pixel(i as u32, j as u32));
            diff[i][j] = rgb_diff;
            calculate_diff[i][j] = rgb_diff as u64;
            total_diff += rgb_diff as u64;
        }
    }

    let avg_diff = total_diff as f64 / (width * height) as f64;

    let mut mountain = AggregateMountain::new(calculate_diff, width, height);
    mountain.gen_aggregate_mountain_mapping();
    mountain.invalid_rectangle(0, 0, width - 1, height - 1);

    let mut ret = vec![];
    result.avg_diff = avg_diff as u32;

    for i in 0..result.top_n {
        let mut top_xy = mountain.fetch_top_point();
        top_xy = adjust_center_point(top_xy, &mountain, &result, width, height, &diff);
        let point = Point::new(top_xy.x, top_xy.y, top_xy.weight as usize);
        ret.push(point);

        if i < result.top_n - 1 {
            trip_aggregate_mountain(&mut mountain, top_xy, &result, width, height);
        }
    }
    ret
}

fn trip_aggregate_mountain(mountain: &mut AggregateMountain, top_xy: XY, result: &HilltopParamAndResult, result_width: usize, result_height: usize) {
    let start_x = max(top_xy.x - result.ch_size as usize / 2, 0);
    let end_x = min(top_xy.x + result.ch_size as usize / 2, (result_width - 1) as usize);
    let start_y = max(top_xy.y - result.ch_size as usize / 2, 0);
    let end_y = min(top_xy.y + result.ch_size as usize / 2, (result_height - 1) as usize);

    let mut max_diff = 0;
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            if mountain.diff_data[x][y] > max_diff {
                max_diff = mountain.diff_data[x][y];
            }
        }
    }

    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let distance = ((x as i64 - top_xy.x as i64).pow(2) as f64 + (y as i64 - top_xy.y as i64).pow(2) as f64).sqrt();
            let distance_ratio = distance / result.ch_size as f64;
            if distance_ratio > 1.0 {
                continue;
            }
            // y = 1- x*x / 2.25 权值衰减函数，为2次函数，要求命中坐标: (0,1) (1.5,0)
            // 当距离为0的时候，衰减权重为1，当距离为1.5的时候，衰减权重为0
            // 当距离为1的时候， 衰减权重为：1- 1/2.25 = 0.55
            mountain.diff_data[x][y] = (mountain.diff_data[x][y] as f64 - (max_diff as f64 * (1.0 - distance_ratio * distance_ratio / 2.25))) as u64;

            // 这块逻辑我也没测试到走这块，有可能有特定的图可能会overflow吧，目前没测试到, 如果存usize的话, 这块是不会走的
            // if mountain.diff_data[x][y] < 0 {
            //     mountain.diff_data[x][y] = 0;
            // }
        }
    }
    mountain.invalid_rectangle(start_x, start_y, end_x, end_y);
}

#[cfg(test)]
mod tests {
    use crate::image_hill_top_v2::{HilltopParamAndResult, find_top_n};

    #[test]
    fn test() {
        let bg_image = image::open("./src/images/bg_image.png").unwrap();
        let cg_image = image::open("./src/images/cg_image.png").unwrap();
        let result = HilltopParamAndResult::new(bg_image, cg_image, 75, 2);
        let result = find_top_n(result);
        println!("{}", result.len());
        println!("{:?}", result);
    }
}