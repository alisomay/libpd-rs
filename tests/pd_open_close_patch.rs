use std::sync::mpsc;

use libpd_rs::functions::block_size;
use libpd_rs::Pd;

#[test]
fn open_close_patch() {
    let mut pd = Pd::init_and_configure(0, 2, 44100).unwrap();
    let ctx = pd.audio_context();

    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());

    // Re-opens.
    assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    assert!(pd.close_patch().is_ok());

    assert!(pd.open_patch("non existent").is_err());

    // Runs osc~ on 440Hz.
    assert!(pd
        .eval_patch(
            r#"
    #N canvas 577 549 158 168 12;
    #X obj 23 116 dac~;
    #X obj 23 17 osc~ 440;
    #X obj 23 66 *~ 0.1;
    #X obj 81 67 *~ 0.1;
    #X connect 1 0 2 0;
    #X connect 1 0 3 0;
    #X connect 2 0 0 0;
    #X connect 3 0 0 1;
        "#,
        )
        .is_ok());

    assert!(pd.dollar_zero().is_ok());

    let output_channels = pd.output_channels();
    let sample_rate = pd.sample_rate();

    pd.activate_audio(true).unwrap();

    let sum = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));

    let (tx, rx) = mpsc::channel::<()>();

    let sum_clone = sum.clone();

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

            let ticks = output_buffer.len() as i32 / (block_size() * output_channels);
            ctx.process_float(ticks, &input_buffer, &mut output_buffer);

            // Collect samples to check if the patch runs later.
            sum_clone.store(
                output_buffer
                    .iter()
                    .fold(0_f32, |sum, sample| sum + (sample * 100.0)) as i32,
                std::sync::atomic::Ordering::SeqCst,
            );

            match rx.try_recv() {
                Ok(_) => break,
                _ => continue,
            }
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(1200));

    // Stop pd.
    tx.send(()).unwrap();
    handle.join().unwrap();

    assert!(sum.load(std::sync::atomic::Ordering::SeqCst) != 0);
    assert!(pd.close_patch().is_ok());
}
