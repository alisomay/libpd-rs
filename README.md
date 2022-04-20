<p align="center">
  <img src="https://raw.githubusercontent.com/alisomay/libpd-rs/main/assets/logo_transparent.png"/>
</p>

# libpd-rs

[![Build and Test Status](https://github.com/alisomay/libpd-rs/workflows/CI/badge.svg)](https://github.com/alisomay/libpd-rs/actions?query=workflow%3ACI)
[![Code Coverage](https://codecov.io/gh/alisomay/libpd-rs/branch/main/graph/badge.svg?token=R25IX6EWRD)](https://codecov.io/gh/alisomay/libpd-rs)

Safe rust abstractions over [libpd-sys](https://github.com/alisomay/libpd-sys).

[Pure Data](https://puredata.info/) (Pd) is a visual programming language developed by [Miller Puckette](https://en.wikipedia.org/wiki/Miller_Puckette) in the 1990s for creating interactive computer music and multimedia works. While Puckette is the main author of the program, Pd is an [open-source project](https://github.com/pure-data/pure-data) with a [large developer base](https://github.com/pure-data/pure-data/graphs/contributors) working on new extensions. It is released under [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause).

Though pd is designed as a desktop application, [libpd](https://github.com/libpd) is an open source project which exposes it as a C library opening the possibility to embed the functionality of pd to any platform which C can compile to.

[libpd-rs](https://github.com/alisomay/libpd-rs) aims to bring [libpd](https://github.com/libpd) to Rust [ecosystem](https://crates.io/). It aims to expose the full functionality of [libpd](https://github.com/libpd) even with some extra additions such as bundling commonly used externals and functionality to increase ease of use.

It is thoroughly documented, well [tested](https://github.com/alisomay/libpd-rs/tests) and enriched with different [examples](https://github.com/alisomay/libpd-rs/examples) to get you started right away.

Now let's make some sound! 🔔

---

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
libpd-rs = "0.1.0"
cpal = "0.13"
```

Paste the code into your `main.rs`:

⚠️ **Warning** ⚠️: This example will produce audio, so please keep your volume at a reasonable level for safety.

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use libpd_rs::convenience::PdGlobal;

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
            let ticks = libpd_rs::convenience::calculate_ticks(output_channels, data.len() as i32);

            // Here if we had an input buffer we could have modified it to do pre-processing.

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
```

This is just the tip of the iceberg about what you can do with [libpd](https://github.com/libpd).

## Next steps

Please check the [examples](https://github.com/alisomay/libpd-rs/examples) and [tests](https://github.com/alisomay/libpd-rs/tests) directory if you're learning better when reading code.

Or if you would like to dive in to [documentation]() please go ahead.

## Resources

- Pure Data
- libpd
- Audio in Rust

...

## Road map

## Contributing

## Last words
