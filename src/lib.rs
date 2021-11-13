use pyo3::{prelude::*};

mod image_utils;
mod image_avg_merger;
mod image_hill_top_v2;

use base64::{decode};
use pyo3::types::{PyList, PyString};
use crate::image_hill_top_v2::{HilltopParamAndResult, Point};
use image_hill_top_v2::{self as x};

#[pyfunction]
pub fn demo_py_function() -> PyResult<String> {
    return PyResult::Ok(String::from("hello rust ffi!"));
}

#[pyfunction]
pub fn avg_b64(input: &PyList) -> PyResult<String> {
    let mut image_input = vec![];
    for src in input {
        let target = decode(src.to_string()).unwrap();
        let img = image::load_from_memory(&target).unwrap();
        image_input.push(img);
    }
    let result = image_avg_merger::avg(&image_input);
    let mut buf = vec![];
    result.write_to(&mut buf, image::ImageOutputFormat::Png).unwrap();
    return PyResult::Ok(base64::encode(&buf));
}

#[pyfunction]
pub fn top_n(bg_image: &PyString, cg_image: &PyString, ch_size: usize, top_n: usize) -> PyResult<Vec<Point>> {
    let target = decode(bg_image.to_string()).unwrap();
    let bg_image = image::load_from_memory(&target).unwrap();
    let target = decode(cg_image.to_string()).unwrap();
    let cg_image = image::load_from_memory(&target).unwrap();
    let result = HilltopParamAndResult::new(bg_image, cg_image, ch_size as u32, top_n);
    let result = x::find_top_n(result);
    return PyResult::Ok(result);
}

#[pymodule]
fn image_magic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(demo_py_function, m)?)?;
    m.add_function(wrap_pyfunction!(avg_b64, m)?)?;
    m.add_function(wrap_pyfunction!(top_n, m)?)?;
    m.add_class::<Point>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
