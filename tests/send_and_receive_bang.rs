#![allow(clippy::restriction)]

use std::sync::{mpsc, Arc, Mutex};

use libpd_rs::{
    block_size, close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    process::process_float,
    receive::{on_bang, receive_messages_from_pd, start_listening_from, stop_listening_from},
    send::send_bang_to,
};

#[test]
fn send_and_receive_bang() {
    let sample_rate = 44100;
    let output_channels = 2;

    let bangs: Arc<Mutex<Vec<&str>>> = Arc::new(Mutex::new(vec![]));

    init().unwrap();
    initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    let bangs_to_fill = bangs.clone();
    on_bang(move |source| {
        assert_eq!(source, "bang_from_pd");
        bangs_to_fill.lock().unwrap().push("bang");
    });
    let receiver_handle = start_listening_from("bang_from_pd").unwrap();

    let (tx, rx) = mpsc::channel::<()>();

    let handle = std::thread::spawn(move || {
        // Mimic audio callback buffers.
        let input_buffer = [0.0f32; 512];
        let mut output_buffer = [0.0f32; 1024];

        // Run pd
        loop {
            // Mimic an audio callback.
            let approximate_buffer_duration =
                (output_buffer.len() as f32 / sample_rate as f32) * 1000.0;
            std::thread::sleep(std::time::Duration::from_millis(
                approximate_buffer_duration as u64,
            ));

            receive_messages_from_pd();
            let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
            process_float(ticks, &input_buffer, &mut output_buffer);
            match rx.try_recv() {
                Ok(_) => break,
                _ => continue,
            }
        }
    });

    // Send 5 bangs.
    for _ in 0..5 {
        send_bang_to("bang_from_rust").unwrap();
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stop pd.
    tx.send(()).unwrap();
    handle.join().unwrap();

    assert_eq!(bangs.lock().unwrap().len(), 5);

    // Stop listening and close handle.
    stop_listening_from(receiver_handle);
    close_patch(patch_handle).unwrap();
}
