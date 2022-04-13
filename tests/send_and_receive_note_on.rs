#![allow(clippy::restriction)]

use std::sync::{mpsc, Arc, Mutex};

use libpd_rs::{
    block_size, close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    process::process_float,
    receive::{on_midi_note_on, receive_messages_from_pd},
    send::send_note_on,
};

#[test]
fn send_and_receive_note_on() {
    let sample_rate = 44100;
    let output_channels = 2;

    let note_on_messages_received: Arc<Mutex<Vec<(i32, i32, i32)>>> = Arc::new(Mutex::new(vec![]));

    let _ = init().unwrap();
    let _ = initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    let messages_to_fill = note_on_messages_received.clone();
    on_midi_note_on(move |channel, pitch, velocity| {
        messages_to_fill
            .lock()
            .unwrap()
            .push((channel, pitch, velocity));
    });

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

            receive_messages_from_pd();
            let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
            process_float(ticks, &input_buffer, &mut output_buffer);
            match rx.try_recv() {
                Ok(_) => break,
                _ => continue,
            }
        }
    });

    let mut channel: i32 = 0;
    let mut pitch: i32 = 0;
    let mut velocity: i32 = 1;
    // Send 5 floats in sequence.
    for _ in 0..5 {
        send_note_on(channel, pitch, velocity).unwrap();
        channel += 1;
        pitch += 1;
        velocity += 1;
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stop pd.
    tx.send(()).unwrap();
    handle.join().unwrap();

    let notes_to_compare: Vec<(i32, i32, i32)> =
        vec![(0, 0, 1), (1, 1, 2), (2, 2, 3), (3, 3, 4), (4, 4, 5)];
    // dbg!(floats.lock().unwrap());
    // dbg!(&floats_to_compare);
    note_on_messages_received
        .lock()
        .unwrap()
        .iter()
        .zip(notes_to_compare.iter())
        .for_each(|((c1, p1, v1), (c2, p2, v2))| {
            assert_eq!(c1, c2);
            assert_eq!(p1, p2);
            assert_eq!(v1, v2);
        });

    close_patch(patch_handle).unwrap();
}
