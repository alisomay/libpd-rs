#![allow(clippy::restriction)]

use std::sync::{mpsc, Arc, Mutex};

use libpd_rs::{
    functions::{
        block_size, close_patch, open_patch, receive::on_midi_byte, send::send_midi_byte,
        util::dsp_on,
    },
    Pd,
};

#[test]
fn send_and_receive_midi_byte() {
    let sample_rate = 44100;
    let output_channels = 2;

    let midi_byte_messages_received: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));

    let pd = Pd::init_and_configure(0, output_channels, sample_rate).unwrap();
    let mut ctx = pd.audio_context();

    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    let messages_to_fill = midi_byte_messages_received.clone();
    on_midi_byte(move |port, byte| {
        messages_to_fill.lock().unwrap().push((port, byte));
    });

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

            ctx.receive_midi_messages_from_pd();
            let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
            ctx.process_float(ticks, &input_buffer, &mut output_buffer);
            match rx.try_recv() {
                Ok(_) => break,
                _ => continue,
            }
        }
    });

    let mut port: i32 = 0;
    let mut byte: i32 = 0x7F;

    #[allow(clippy::explicit_counter_loop)]
    // Send 5 note on messages in sequence.
    for _ in 0..5 {
        send_midi_byte(port, byte).unwrap();
        port += 1;
        byte += 0x10;
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stop pd.
    tx.send(()).unwrap();
    handle.join().unwrap();

    let vales_to_compare: Vec<(i32, i32)> =
        vec![(0, 0x7F), (1, 0x8F), (2, 0x9F), (3, 0xAF), (4, 0xBF)];

    assert_eq!(midi_byte_messages_received.lock().unwrap().len(), 5);

    vales_to_compare
        .iter()
        .zip(midi_byte_messages_received.lock().unwrap().iter())
        .for_each(|((c1, p1), (c2, p2))| {
            assert_eq!(c1, c2);
            assert_eq!(p1, p2);
        });

    close_patch(patch_handle).unwrap();
}
