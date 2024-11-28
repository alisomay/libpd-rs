<p align="center">
  <img src="https://raw.githubusercontent.com/alisomay/libpd-rs/main/assets/logo_transparent.png"/>
</p>

# libpd-rs

[![Build and Test Status](https://github.com/alisomay/libpd-rs/workflows/CI/badge.svg)](https://github.com/alisomay/libpd-rs/actions?query=workflow%3ACI)
[![Code Coverage](https://codecov.io/gh/alisomay/libpd-rs/branch/main/graph/badge.svg?token=R25IX6EWRD)](https://codecov.io/gh/alisomay/libpd-rs)

Safe rust abstractions over [libpd-sys](https://github.com/alisomay/libpd-sys).

[Pure Data](https://puredata.info/) (Pd) is a visual programming language developed by [Miller Puckette](https://en.wikipedia.org/wiki/Miller_Puckette) in the 1990s for creating interactive computer music and multimedia works. While Puckette is the main author of the program, Pd is an [open-source project](https://github.com/pure-data/pure-data) with a [large developer base](https://github.com/pure-data/pure-data/graphs/contributors) working on new extensions. It is released under [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause).

Though pd is designed as a desktop application, [libpd](https://github.com/libpd) is an open source project which exposes it as a C library opening the possibility to embed the functionality of pd to any platform which C can compile to.

[libpd-rs](/) aims to bring [libpd](https://github.com/libpd) to the Rust [ecosystem](https://crates.io/). It aims to expose the full functionality of [libpd](https://github.com/libpd) with some extra additions such as bundling commonly used externals and addition of extra functionality for increased ease of use.

It is thoroughly [documented](https://docs.rs/libpd-rs/latest/libpd_rs/#), well [tested](/tests/) and enriched with various [examples](/examples/) to get you started right away.

Now let's make some sound! 🔔

---

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
libpd-rs = "0.1"
cpal = "0.15"
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
            libpd_rs::functions::process::process_float(ticks, &[], data);

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
```

This is just the tip of the iceberg about what you can do with [libpd](https://github.com/libpd).

The patch we had just evaluated would look like this in pd desktop application:

<p align="left">
  <img src="https://raw.githubusercontent.com/alisomay/libpd-rs/main/assets/sine_patch.png"/>
</p>

## Running the examples and tests

After cloning the repository, in the repository root run:

```sh
cargo run --example <name of the example>
```

e.g.

```sh
cargo run --example with_nannou
```

Please check the [README](/examples/) on examples for more information.

For the tests, you may run `cargo test` directly.

## Next steps

Please check the [examples](/examples/) and [tests](/tests/) directories if you learn better when reading code.

Or if you would like to dive in to [documentation](https://docs.rs/libpd-rs/latest/libpd_rs/#) please go ahead.

## Resources

- Pure Data
  - <https://puredata.info/>
  - <https://forum.pdpatchrepo.info/>
  - <http://www.pd-tutorial.com/>
  - <https://www.worldscientific.com/worldscibooks/10.1142/6277>
  - <https://mitpress.mit.edu/books/designing-sound>
  - <https://www.soundonsound.com/techniques/pure-data-introduction>
  - [collection of resources in modwiggler](https://modwiggler.com/forum/viewtopic.php?t=236092)
  - [collection of resources in reddit](https://www.reddit.com/r/puredata/comments/cpb4um/resources_for_learning_pd/)
  - [a guide to writing pd externals in C](https://github.com/pure-data/externals-howto)
- [libpd](https://github.com/libpd)
- Audio in Rust
  - <https://github.com/kfrncs/awesome-rust-audio>
  - <https://github.com/RustAudio>
  - <https://www.theaudioprogrammer.com/discord>

## Road map

- [Multi hooks support](https://github.com/libpd/libpd/pull/282/files#diff-51ce01cd8a0f2a0249dc73e318ccfb430fbe0e341edfd69a8a83ccd81f58e29aR502)
- [Multi instance support](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.h#L529)
- Support for Android and IOS
- Enrich [examples](/examples/) with nice patches and add also examples with [bevy](https://bevyengine.org/) and [nannou](https://github.com/nannou-org/nannou).

## Support

- Desktop
  - macOS:
    - `x86_64` ✅
    - `aarch64` ✅
  - linux:
    - `x86_64` ✅
    - `aarch64` ✅
  - windows:
    - msvc
      - `x86_64` ✅
      - `aarch64` (not tested but should work)
    - gnu
      - `x86_64` (not tested but should work)
      - `aarch64` (not tested but should work)
- Mobile

  - iOS (not yet but will be addressed)
  - Android (not yet but will be addressed)

- Web (not yet but will be addressed)

## List of bundled externals

The way to add externals to [libpd](https://github.com/libpd/libpd) is to compile and statically link them.

[libpd-rs](https://github.com/alisomay/libpd-rs) will be bundling some of the essential and commonly used externals in pure data.
This list will be growing as we add more externals.

If you have ideas please consider writing an answer to this [post](https://www.reddit.com/r/puredata/comments/u9h1bk/a_list_of_essential_indispensable_and_commonly/).

- `moog~`
- `freeverb~`

## Contributing

- Be friendly and productive
- Follow common practice open source contribution culture
- Rust [code of conduct](https://www.rust-lang.org/policies/code-of-conduct) applies

Thank you 🙏

## Similar projects

- <https://github.com/x37v/puredata-rust>
- <https://github.com/wavejumper/pd-external-rs>
- <https://github.com/x37v/puredata-rust/tree/HEAD/pd-sys>

## Last words

Generative or algorithmic music is a powerful tool for exploration, pumps up creativity and goes very well together with traditional music making approaches also.

Making apps which produce meaningful sound is difficult, I wish that this crate would ease your way on doing that and make complicated audio ideas in apps accessible to more people.
