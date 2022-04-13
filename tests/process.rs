// TODO: Tests for process functions

#![allow(clippy::restriction)]

use std::any::type_name;

use libpd_rs::{
    block_size, close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    process::{
        process_double, process_float, process_raw, process_raw_double, process_raw_short,
        process_short,
    },
    receive::{on_bang, receive_messages_from_pd, start_listening_from, stop_listening_from},
    send::send_bang_to,
};

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[test]
fn all_process_functions() {
    let sample_rate = 44100;
    let output_channels = 2;

    let _ = init().unwrap();
    let _ = initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/sine.pd").unwrap();

    // Float
    let input_buffer = [0.0f32; 512];
    let mut output_buffer = [0.0f32; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
        process_float(ticks, &input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_f32, |mut acc, element| {
        acc += element;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "f32");
    assert_ne!(sum, 0.0);

    // Double
    let input_buffer = [0.0f64; 512];
    let mut output_buffer = [0.0f64; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
        process_double(ticks, &input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_f64, |mut acc, element| {
        acc += element;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "f64");
    assert_ne!(sum, 0.0);

    // Short
    let input_buffer = [0i16; 512];
    let mut output_buffer = [0i16; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
        process_short(ticks, &input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_i32, |mut acc, element| {
        acc += *element as i32;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "i16");
    assert_ne!(sum, 0);

    // Float Raw
    let input_buffer = [0.0f32; 512];
    let mut output_buffer = [0.0f32; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        process_raw(&input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_f32, |mut acc, element| {
        acc += element;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "f32");
    assert_ne!(sum, 0.0);

    // Double Raw
    let input_buffer = [0.0f64; 512];
    let mut output_buffer = [0.0f64; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        process_raw_double(&input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_f64, |mut acc, element| {
        acc += element;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "f64");
    assert_ne!(sum, 0.0);

    // Short Raw
    let input_buffer = [0_i16; 512];
    let mut output_buffer = [0_i16; 1024];

    for _ in 0..(44100 / block_size() * output_channels) {
        process_raw_short(&input_buffer, &mut output_buffer);
    }

    let sum = output_buffer.iter().fold(0_i32, |mut acc, element| {
        acc += *element as i32;
        acc
    });

    assert_eq!(type_of(output_buffer[0]), "i16");
    assert_ne!(sum, 0);

    close_patch(patch_handle).unwrap();
}
