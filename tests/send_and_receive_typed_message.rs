#![allow(clippy::restriction)]

use std::sync::{mpsc, Arc, Mutex};

use libpd_rs::{
    block_size, close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    process::process_float,
    receive::{
        on_message, on_print, receive_messages_from_pd, start_listening_from, stop_listening_from,
    },
    send::send_message_to,
    verbose_print_state,
};

// TODO: This test may improve.
// TODO: Learn more about these kinds of messages.
#[test]
fn send_and_receive_typed_message() {
    let sample_rate = 44100;
    let output_channels = 2;

    // let console = Arc::new(Mutex::new(String::new()));

    let _ = init().unwrap();
    let _ = initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();
    verbose_print_state(true);

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    // let console_to_fill = console.clone();

    // on_print(move |value| {
    //     console_to_fill
    //         .lock()
    //         .unwrap()
    //         .push_str(&format!("{value}\n"));
    // });

    on_message(move |source, selector, _list| {
        assert_eq!(source, "pd");
        assert_eq!(selector, "audiostatus");
    });
    let receiver_handle = start_listening_from("pd").unwrap();

    // Mimic audio callback buffers.
    let input_buffer = [0.0f32; 512];
    let mut output_buffer = [0.0f32; 1024];

    let (tx, rx) = mpsc::channel::<()>();

    let handle = std::thread::spawn(move || {
        // Run pd
        loop {
            // Mimic an audio callback.
            let approximate_buffer_duration =
                (output_buffer.len() as f32 / sample_rate as f32) * 1000.0;
            std::thread::sleep(std::time::Duration::from_millis(
                approximate_buffer_duration as u64,
            ));

            let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
            process_float(ticks, &input_buffer, &mut output_buffer);
            receive_messages_from_pd();
            match rx.try_recv() {
                Ok(_) => break,
                _ => continue,
            }
        }
    });

    send_message_to("pd", "audiostatus", &[]).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Stop pd.
    tx.send(()).unwrap();
    handle.join().unwrap();

    // Stop listening and close handle.
    stop_listening_from(receiver_handle);
    close_patch(patch_handle).unwrap();
}
