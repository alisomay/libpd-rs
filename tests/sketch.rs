#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]

// Currently this file is only a sketch.
// It will turn to a set of tests soon.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[test]
fn main() {
    unsafe {
        let mut v = Arc::new(Mutex::new(vec![]));
        // INIT ORDER
        // First set hooks
        libpd_rs::mirror::on_print(|val| {
            dbg!(val);
        });
        let a = v.clone();
        libpd_rs::mirror::on_float(move |src, val| {
            a.lock().unwrap().push(val);
            dbg!("This", src, val);
        });

        libpd_rs::mirror::on_float(move |src, val| {
            dbg!("Other", src, val);
        });

        libpd_rs::mirror::on_list(move |src, val| {
            dbg!("Atoms!", src, val);
        });

        libpd_rs::mirror::on_symbol(move |src, val| {
            dbg!("SYM!", src, val);
        });

        // Then init queued
        libpd_rs::mirror::init();
        // Then init audio
        libpd_rs::mirror::initialize_audio(0, 2, 44100);

        // Turn dsp on
        libpd_sys::libpd_start_message(1);
        libpd_sys::libpd_add_float(1.0);
        let msg = std::ffi::CString::new("pd").unwrap();
        let recv = std::ffi::CString::new("dsp").unwrap();
        let r = libpd_sys::libpd_finish_message(msg.as_ptr(), recv.as_ptr());
        dbg!(r);

        let mut path = PathBuf::new();
        path.push("/Users/vallahiboyle/Desktop/software/libpd-rs/simple.pd");
        libpd_rs::mirror::add_to_search_paths(&path);
        // Open patch
        let mut path1 = PathBuf::new();
        path1.push("simple.pd");

        let h = libpd_rs::mirror::open_patch(&path1);
        if let Ok(a) = h {
            dbg!("OK");
        } else {
            dbg!("Failed to open patch");
        }
        // Bind
        let r = libpd_rs::mirror::start_listening_from("simple_float");
        let r = libpd_rs::mirror::start_listening_from("listy");
        let r = libpd_rs::mirror::start_listening_from("list_loopback_float");
        let r = libpd_rs::mirror::start_listening_from("list_loopback_symbol");
        let r = libpd_rs::mirror::start_listening_from("AMK");
        dbg!(r);
        // Run
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();

        // Default devices.
        let output_device = host
            .default_output_device()
            .expect("failed to get default output device");
        println!(
            "Using default output device: \"{}\"",
            output_device.name().unwrap()
        );

        dbg!(libpd_rs::mirror::block_size());
        let config: cpal::StreamConfig = output_device.default_output_config().unwrap().into();
        let channels = 2;
        dbg!(&config);

        // let input_buffer = [0.0f32; 1024];
        // let mut output_buffer = [0.0f32; 1024];

        // // now run pd for ten seconds (logical time)
        // for _ in 0..((10 * 44100) / 64) {
        //     // fill input_buffer here
        //     libpd_sys::libpd_process_float(1024 / 128, [].as_ptr(), output_buffer[..].as_mut_ptr());
        //     // use output_buffer here
        // }

        // for sample in output_buffer.iter().take(10) {
        //     println!("{}", sample);
        // }

        let stream = output_device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    let ticks = data.len() / ((libpd_rs::mirror::block_size() as usize) * channels);
                    // process interleaved float samples from inBuffer -> libpd -> outBuffer
                    // buffer sizes are based on # of ticks and channels where:
                    //     size = ticks * libpd_blocksize() * (in/out)channels
                    // s/t  =  b * c
                    //
                    // returns 0 on success

                    // dbg!(ticks, data.len());
                    // for sample in data.iter_mut() {
                    //     *sample = 0.0;
                    // }

                    libpd_rs::mirror::process_float(ticks as i32, &[], data);
                    // libpd_sys::libpd_process_dummy(8, [].as_ptr(), data.as_mut_ptr());
                },
                |err| eprintln!("an error occurred on stream: {}", err),
            )
            .unwrap();
        stream.play().unwrap();

        let message_length = 4;
        if libpd_rs::mirror::start_message(message_length).is_ok() {
            libpd_rs::mirror::add_symbol_to_started_message("foo");
            libpd_rs::mirror::add_symbol_to_started_message("foo");
        }

        // loop {}
        // 44100
        let mut cnt = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
            // std::thread::sleep(std::time::Duration::from_millis(1000));
            // println!("Try receive");
            // dbg!(libpd_sys::libpd_exists(send.as_ptr()));
            libpd_rs::mirror::receive_messages_from_pd();
            // dbg!(v.lock().unwrap());
            if cnt % 2000 == 0 {
                let list = vec![
                    libpd_rs::types::Atom::Float(33.5),
                    libpd_rs::types::Atom::Symbol("hello".to_owned()),
                ];
                libpd_rs::mirror::send_list_to("from_rust", &list);
            }

            // println!("Available: {}", r);

            // Process
            // libpd_sys::libpd_process_float(
            //     1,
            //     input_buffer[..].as_ptr(),
            //     output_buffer[..].as_mut_ptr(),
            // );
            cnt += 1;
        }
    }
}
