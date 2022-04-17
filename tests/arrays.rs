#![allow(clippy::restriction)]

use libpd_rs::{
    array::{
        array_size, read_double_array_from, read_float_array_from, resize_array,
        write_double_array_to, write_float_array_to,
    },
    close_patch,
};

#[test]
fn apply_array_operations_in_a_row() {
    let _ = libpd_rs::init().unwrap();
    let handle = libpd_rs::open_patch("tests/patches/array_sketch_pad.pd").unwrap();

    let bad_name = "not_exists";
    let sketch_pad = "sketch_pad";

    assert!(array_size(bad_name).is_err());

    // Default
    let size = array_size(sketch_pad).unwrap();
    assert_eq!(size, 100);

    assert!(resize_array(bad_name, 1024).is_err());

    resize_array(sketch_pad, -1).unwrap();
    let size = array_size(sketch_pad).unwrap();
    assert_eq!(size, 1);

    resize_array(sketch_pad, 0).unwrap();
    let size = array_size(sketch_pad).unwrap();
    assert_eq!(size, 1);

    // Protected with size limits. Over the limit is also 1.
    resize_array(sketch_pad, i32::MAX).unwrap();
    let size = array_size(sketch_pad).unwrap();
    assert_eq!(size, 1);

    resize_array(sketch_pad, 1024).unwrap();
    let size = array_size(sketch_pad).unwrap();
    assert_eq!(size, 1024);

    resize_array(sketch_pad, 8).unwrap();

    // Reading and writing works floats.
    let will_write_f: Vec<f32> = vec![0.0, 0.0, 1.0, 1.0];
    let mut read_to: Vec<f32> = vec![0.0; 4];

    write_float_array_to(sketch_pad, 0, &will_write_f, 4).unwrap();
    read_float_array_from(sketch_pad, 4, &mut read_to, 0).unwrap();
    assert_eq!(read_to, will_write_f);

    let will_write_d: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0];
    let mut read_to: Vec<f64> = vec![0.0; 4];

    write_double_array_to(sketch_pad, 0, &will_write_d, 4).unwrap();
    read_double_array_from(sketch_pad, 4, &mut read_to, 0).unwrap();
    assert_eq!(read_to, will_write_d);
    //

    // Clear float arrays.
    let mut read_to: Vec<f32> = vec![0.0; 6];
    write_float_array_to(sketch_pad, 0, &[0.0, 0.0, 0.0, 0.0], 4).unwrap();

    // Offsets
    write_float_array_to(sketch_pad, 2, &[1.0, 1.0, 0.0, 0.0], 4).unwrap();
    read_float_array_from(sketch_pad, 6, &mut read_to, 0).unwrap();

    assert_eq!(read_to, vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0]);

    // Clear float arrays.
    let mut read_to: Vec<f64> = vec![0.0; 6];
    write_double_array_to(sketch_pad, 0, &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 6).unwrap();

    // Offsets
    write_double_array_to(sketch_pad, 2, &[1.0, 1.0, 0.0, 0.0], 4).unwrap();
    read_double_array_from(sketch_pad, 6, &mut read_to, 0).unwrap();

    assert_eq!(read_to, vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0]);

    // Bounds

    // Double
    let will_write_long_array: Vec<f64> = vec![0.0; 1024];
    let result = write_double_array_to(sketch_pad, 4, &will_write_long_array, 1024);
    assert!(result.is_err());

    let mut read_to: Vec<f64> = vec![0.0; 4];
    let result = read_double_array_from(sketch_pad, 50, &mut read_to, 0);
    assert!(result.is_err());

    let mut read_to: Vec<f64> = vec![0.0; 1024];
    let result = read_double_array_from(sketch_pad, 1024, &mut read_to, 0);
    assert!(result.is_err());

    let mut read_to: Vec<f64> = vec![0.0; 1024];
    let result = read_double_array_from(sketch_pad, -1, &mut read_to, 0);
    assert!(result.is_err());

    // Float
    let will_write_long_array: Vec<f32> = vec![0.0; 1024];
    let result = write_float_array_to(sketch_pad, 4, &will_write_long_array, 1024);
    assert!(result.is_err());

    let mut read_to: Vec<f32> = vec![0.0; 4];
    let result = read_float_array_from(sketch_pad, 50, &mut read_to, 0);
    assert!(result.is_err());

    let mut read_to: Vec<f32> = vec![0.0; 1024];
    let result = read_float_array_from(sketch_pad, 1024, &mut read_to, 0);
    assert!(result.is_err());

    let mut read_to: Vec<f32> = vec![0.0; 1024];
    let result = read_float_array_from(sketch_pad, -1, &mut read_to, 0);
    assert!(result.is_err());

    close_patch(handle).unwrap();
}
