use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use libpd_rs::{
    convenience::PdGlobal, receive::on_float, receive::receive_messages_from_pd, send::send_list_to,
};
use sys_info::loadavg;

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
    let mut pd = PdGlobal::init_and_configure(0, output_channels, sample_rate)?;

    // Let's evaluate another pd patch.
    // We could have opened a `.pd` file also.
    pd.eval_patch(
        r#"
    #N canvas 832 310 625 448 12;
    #X obj 18 27 r cpu_load;
    #X obj 55 394 s response;
    #X obj 13 261 *~;
    #X obj 112 240 vline~;
    #X obj 118 62 bng 15 250 50 0 empty empty empty 17 7 0 10 -262144 -1
    -1;
    #X obj 14 395 dac~;
    #X obj 50 299 sig~;
    #X floatatom 50 268 5 0 0 0 - - -;
    #X obj 13 228 phasor~ 120;
    #X obj 139 61 metro 2000;
    #X obj 139 38 tgl 15 0 empty empty empty 17 7 0 10 -262144 -1 -1 1
    1;
    #X obj 18 52 unpack f f;
    #X obj 14 362 *~ 2;
    #X obj 14 336 vcf~ 12;
    #X obj 139 12 loadbang;
    #X msg 118 86 1 8 \, 0 0 10;
    #X obj 149 197 expr (480 + 80) * ($f1 - 8) / (4 - 16) + 480;
    #X obj 29 128 * 20;
    #X obj 167 273 expr (520 + 120) * ($f1 - 5) / (12 - 5) + 120;
    #X connect 0 0 11 0;
    #X connect 2 0 13 0;
    #X connect 3 0 2 1;
    #X connect 4 0 15 0;
    #X connect 6 0 13 1;
    #X connect 7 0 6 0;
    #X connect 8 0 2 0;
    #X connect 9 0 15 0;
    #X connect 10 0 9 0;
    #X connect 11 0 16 0;
    #X connect 11 0 18 0;
    #X connect 11 1 17 0;
    #X connect 12 0 5 0;
    #X connect 12 0 5 1;
    #X connect 13 0 12 0;
    #X connect 14 0 10 0;
    #X connect 15 0 3 0;
    #X connect 16 0 9 1;
    #X connect 17 0 1 0;
    #X connect 17 0 13 2;
    #X connect 18 0 7 0;
        "#,
    )?;

    // Here we are registering a listener (hook in libpd lingo) for
    // float values which are received from the pd patch.
    on_float(|source, value| {
        if source == "response" {
            print!("\r");
            print!("Pd says that the q value of the vcf~ is: {value}");
        }
    });

    // Pd can send data to many different endpoints at a time.
    // This is why we need to declare our subscription to one or more first.
    // In this case we're subscribing to one, but it could have been many,
    pd.subscribe_to("response")?;

    // Build the audio stream.
    let output_stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Provide the ticks to advance per iteration for the internal scheduler.
            let ticks = libpd_rs::convenience::calculate_ticks(output_channels, data.len() as i32);

            // Here if we had an input buffer we could have modified it to do pre-processing.

            // To receive messages from the pd patch we need to read the ring buffers filled by the pd patch
            // repeatedly to check if there are messages there.
            // Audio callback is a nice place to do that.
            receive_messages_from_pd();

            // Process audio, advance internal scheduler.
            libpd_rs::process::process_float(ticks, &[], data);

            // Here we could have done post processing after pd processed our output buffer in place.
        },
        |err| eprintln!("an error occurred on stream: {}", err),
    )?;

    // Turn audio processing on
    pd.activate_audio(true)?;

    // Run the stream
    output_stream.play()?;

    // This program does not terminate.
    // You would need to explicitly quit it.
    loop {
        // We sample in 2 hz.
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read the avarage load of the cpu.
        let load = loadavg()?;

        let one_minute_cpu_load_average = load.one;
        let five_minutes_cpu_load_average = load.five;

        // Lists are one of the types we can send to pd.
        // Although pd allows for heterogeneous lists,
        // even if we're not using them heterogeneously in this example,
        // we still need to send it as a list of Atoms.

        // Atom is an encapsulating type in pd to unify
        // various types of data together under the same umbrella.
        // Check out `libpd_rs::types` module for more details.

        // Atoms have From trait implemented for them for
        // floats and strings.
        send_list_to(
            "cpu_load",
            &[
                one_minute_cpu_load_average.into(),
                five_minutes_cpu_load_average.into(),
            ],
        )?;
    }
}
