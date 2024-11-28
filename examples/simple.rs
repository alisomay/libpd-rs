use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use libpd_rs::Pd;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize cpal
    // This could have been another cross platform audio library
    // basically anything which gets you the audio callback of the os.
    let host = cpal::default_host();

    // Currently we're only going to output to the default device
    let device = host.default_output_device().unwrap();

    // Using the default config
    let config = device.default_output_config()?;

    // Let's get the default configuration from the audio driver.
    let sample_rate = config.sample_rate().0 as i32;
    let output_channels = config.channels() as i32;

    // Initialize libpd with that configuration,
    // with no input channels since we're not going to use them.
    let mut pd = Pd::init_and_configure(0, output_channels, sample_rate)?;
    let ctx = pd.audio_context();

    // Let's evaluate a pd patch.
    // We could have opened a `.pd` file also.
    // This patch would play a sine wave at 440hz.
    pd.eval_patch(
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
    )?;

    // Build the audio stream.
    let output_stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Provide the ticks to advance per iteration for the internal scheduler.
            let ticks =
                libpd_rs::functions::util::calculate_ticks(output_channels, data.len() as i32);

            // Here if we had an input buffer we could have modified it to do pre-processing.

            // Process audio, advance internal scheduler.
            ctx.process_float(ticks, &[], data);

            // Here we could have done post processing after pd processed our output buffer in place.
        },
        |err| eprintln!("an error occurred on stream: {}", err),
        None,
    )?;

    // Turn audio processing on
    pd.activate_audio(true)?;

    // Run the stream
    output_stream.play()?;

    // Wait a bit for listening..
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Turn audio processing off
    pd.activate_audio(false)?;

    // Pause the stream
    output_stream.pause()?;

    // Close the patch
    pd.close_patch()?;

    // Leave
    Ok(())
}
