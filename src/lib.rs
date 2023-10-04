#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::undocumented_unsafe_blocks,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    // clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,
    clippy::std_instead_of_alloc,
    clippy::question_mark_used,
    clippy::std_instead_of_core,
    clippy::partial_pub_fields,
    clippy::ref_patterns,

    // Expect is fine in relevant cases
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,

    // Docs
    clippy::missing_docs_in_private_items,

    // Comment these out before submitting a PR
    // clippy::todo,
    // clippy::panic_in_result_fn,
    // clippy::panic,
    // clippy::unimplemented,
    // clippy::unreachable,
)]
#![cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("sine_patch", "assets/sine_patch.png"),
doc = ::embed_doc_image::embed_image!("phasor_patch", "assets/phasor_patch.png"),
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alisomay/libpd-rs/main/assets/logo_transparent.png",
    html_favicon_url = "https://raw.githubusercontent.com/alisomay/libpd-rs/main/assets/favicon/favicon.ico"
)]

//! # A safe wrapper around libpd
//!
//! [Pure Data](https://puredata.info/) (Pd) is a visual programming language developed by
//! [Miller Puckette](https://en.wikipedia.org/wiki/Miller_Puckette) in the 1990s
//! for creating interactive computer music and multimedia works.
//! While Puckette is the main author of the program,
//! Pd is an [open-source project](https://github.com/pure-data/pure-data) with a
//! [large developer base](https://github.com/pure-data/pure-data/graphs/contributors) working on new extensions.
//! It is released under [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause).
//!
//! Though pd is designed as a desktop application,
//! [libpd](https://github.com/libpd) is an open source project
//! which exposes it as a C library opening the possibility to
//! embed the functionality of pd to any platform which C can compile to.
//!
//! [libpd-rs](https://github.com/alisomay/libpd-rs) aims to bring [libpd](https://github.com/libpd)
//! to the Rust [ecosystem](https://crates.io/).
//! It aims to expose the full functionality of [libpd](https://github.com/libpd)
//! with some extra additions such as bundling commonly used externals
//! and addition of extra functionality for increased ease of use.
//!
//! It is thoroughly [documented](https://docs.rs/libpd-rs/0.1.9/libpd_rs/#),
//! well [tested](https://github.com/alisomay/libpd-rs/tests) and enriched with
//! various [examples](https://github.com/alisomay/libpd-rs/examples) to get you started right away.
//!
//! Now let's make some sound! 🔔
//!
//! ## Getting Started
//!
//! Add the latest version of [libpd-rs](https://github.com/alisomay/libpd-rs) to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! libpd-rs = "0.1.9"
//! cpal = "0.13"
//! ```
//! We also add [cpal](https://github.com/RustAudio/cpal) to our dependencies
//! to get access to the high priority audio callback from the OS.
//!
//! [cpal](https://github.com/RustAudio/cpal) is not a must.
//! You may have used any method to get audio callback from the OS.
//!
//! ## Examples and Usage
//!
//! To start making sound with [libpd-rs](https://github.com/alisomay/libpd-rs), we need to have a pd patch at hand.
//! Pd patches are `.pd` files which could be read by pd desktop [application](https://puredata.info/downloads).
//!
//! Pd patches are not binary files, they are simple files full of pd commands as text.
//! [libpd-rs](https://github.com/alisomay/libpd-rs) provides an additional way to
//! [evaluate](crate::convenience::PdGlobal::eval_patch) strings as pd patches.
//!
//! This is the [method](crate::convenience::PdGlobal::eval_patch) we'll use in the following examples.
//!
//! ### Initialize, open patch, run
//!
//! Paste the code into your `main.rs`:
//!
//! ⚠️ **Warning** ⚠️: This example will produce audio, so please keep your volume at a reasonable level for safety.
//!
//! ```no_run
//! use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//! use libpd_rs::convenience::{PdGlobal, calculate_ticks};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//!     // Initialize cpal
//!     // This could have been another cross platform audio library
//!     // basically anything which gets you the audio callback of the os.
//!     let host = cpal::default_host();
//!
//!     // Currently we're only going to output to the default device
//!     let device = host.default_output_device().unwrap();
//!
//!     // Using the default config
//!     let config = device.default_output_config()?;
//!
//!     // Let's get the default configuration from the audio driver.
//!     let sample_rate = config.sample_rate().0 as i32;
//!     let output_channels = config.channels() as i32;
//!
//!     // Initialize libpd with that configuration,
//!     // with no input channels since we're not going to use them.
//!     let mut pd = PdGlobal::init_and_configure(0, output_channels, sample_rate)?;
//!
//!     // Let's evaluate a pd patch.
//!     // We could have opened a `.pd` file also.
//!     // This patch would play a sine wave at 440hz.
//!     pd.eval_patch(
//!         r#"
//!     #N canvas 577 549 158 168 12;
//!     #X obj 23 116 dac~;
//!     #X obj 23 17 osc~ 440;
//!     #X obj 23 66 *~ 0.1;
//!     #X obj 81 67 *~ 0.1;
//!     #X connect 1 0 2 0;
//!     #X connect 1 0 3 0;
//!     #X connect 2 0 0 0;
//!     #X connect 3 0 0 1;
//!         "#,
//!     )?;
//!
//!     // Build the audio stream.
//!     let output_stream = device.build_output_stream(
//!         &config.into(),
//!         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//!             // Provide the ticks to advance per iteration for the internal scheduler.
//!             let ticks = calculate_ticks(output_channels, data.len() as i32);
//!
//!             // Here if we had an input buffer
//!             // we could have modified it to do pre-processing.
//!
//!             // Process audio, advance internal scheduler.
//!             libpd_rs::process::process_float(ticks, &[], data);
//!
//!             // Here we could have done post processing
//!             // after pd processed our output buffer in place.
//!         },
//!         |err| eprintln!("an error occurred on stream: {}", err),
//!     )?;
//!
//!     // Turn audio processing on
//!     pd.activate_audio(true)?;
//!
//!     // Run the stream
//!     output_stream.play()?;
//!
//!     // Wait a bit for listening..
//!     std::thread::sleep(std::time::Duration::from_secs(5));
//!
//!     // Turn audio processing off
//!     pd.activate_audio(false)?;
//!
//!     // Pause the stream
//!     output_stream.pause()?;
//!
//!     // Close the patch
//!     pd.close_patch()?;
//!
//!     // Leave
//!     Ok(())
//! }
//! ```
//!
//! The patch you have just evaluated and listened looks exactly like this in pd desktop [application](https://puredata.info/downloads).
//!
//! ![Sine wave generating pd patch][sine_patch]
//!
//! ### Communicate with the patch
//!
//! Again with a simplistic patch, this time we'll send and receive messages from pd.
//! We'll be monitoring our cpu load average over a minute and 5 minutes, send this data to pd
//! as a list and let it change some parameters in our simplistic patch.
//!
//! As a last thing we'll send the data we've applied to an object in the patch  back to Rust to read it.
//! This is a very simple example of how to send and receive data.
//!
//! Add the following dependencies to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! libpd-rs = "0.1.9"
//! cpal = "0.13"
//! sys-info = "0.9.1"
//! ```
//! Paste the code into your `main.rs`:
//!
//! ⚠️ **Warning** ⚠️: This example will produce audio, so please keep your volume at a reasonable level for safety.
//!
//! ```no_run
//! use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//! use libpd_rs::{
//!     convenience::{PdGlobal, calculate_ticks}, receive::on_float, receive::receive_messages_from_pd, send::send_list_to,
//! };
//! use sys_info::loadavg;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize cpal
//!     // This could have been another cross platform audio library
//!     // basically anything which gets you the audio callback of the os.
//!     let host = cpal::default_host();
//!
//!     // Currently we're only going to output to the default device
//!     let device = host.default_output_device().unwrap();
//!
//!     // Using the default config
//!     let config = device.default_output_config()?;
//!
//!     // Let's get the default configuration from the audio driver.
//!     let sample_rate = config.sample_rate().0 as i32;
//!     let output_channels = config.channels() as i32;
//!
//!     // Initialize libpd with that configuration,
//!     // with no input channels since we're not going to use them.
//!     let mut pd = PdGlobal::init_and_configure(0, output_channels, sample_rate)?;
//!
//!     // Let's evaluate another pd patch.
//!     // We could have opened a `.pd` file also.
//!     pd.eval_patch(
//!         r#"
//!     #N canvas 832 310 625 448 12;
//!     #X obj 18 27 r cpu_load;
//!     #X obj 55 394 s response;
//!     #X obj 13 261 *~;
//!     #X obj 112 240 vline~;
//!     #X obj 118 62 bng 15 250 50 0 empty empty empty 17 7 0 10 -262144 -1
//!     -1;
//!     #X obj 14 395 dac~;
//!     #X obj 50 299 sig~;
//!     #X floatatom 50 268 5 0 0 0 - - -;
//!     #X obj 13 228 phasor~ 120;
//!     #X obj 139 61 metro 2000;
//!     #X obj 139 38 tgl 15 0 empty empty empty 17 7 0 10 -262144 -1 -1 1
//!     1;
//!     #X obj 18 52 unpack f f;
//!     #X obj 14 362 *~ 2;
//!     #X obj 14 336 vcf~ 12;
//!     #X obj 139 12 loadbang;
//!     #X msg 118 86 1 8 \, 0 0 10;
//!     #X obj 149 197 expr (480 + 80) * ($f1 - 8) / (4 - 16) + 480;
//!     #X obj 29 128 * 20;
//!     #X obj 167 273 expr (520 + 120) * ($f1 - 5) / (12 - 5) + 120;
//!     #X connect 0 0 11 0;
//!     #X connect 2 0 13 0;
//!     #X connect 3 0 2 1;
//!     #X connect 4 0 15 0;
//!     #X connect 6 0 13 1;
//!     #X connect 7 0 6 0;
//!     #X connect 8 0 2 0;
//!     #X connect 9 0 15 0;
//!     #X connect 10 0 9 0;
//!     #X connect 11 0 16 0;
//!     #X connect 11 0 18 0;
//!     #X connect 11 1 17 0;
//!     #X connect 12 0 5 0;
//!     #X connect 12 0 5 1;
//!     #X connect 13 0 12 0;
//!     #X connect 14 0 10 0;
//!     #X connect 15 0 3 0;
//!     #X connect 16 0 9 1;
//!     #X connect 17 0 1 0;
//!     #X connect 17 0 13 2;
//!     #X connect 18 0 7 0;
//!         "#,
//!     )?;
//!
//!     // Here we are registering a listener (hook in libpd lingo) for
//!     // float values which are received from the pd patch.
//!     on_float(|source, value| {
//!         if source == "response" {
//!             print!("\r");
//!             print!("Pd says that the q value of the vcf~ is: {value}");
//!         }
//!     });
//!
//!     // Pd can send data to many different endpoints at a time.
//!     // This is why we need to declare our subscription to one or more first.
//!     // In this case we're subscribing to one, but it could have been many,
//!     pd.subscribe_to("response")?;
//!
//!     // Build the audio stream.
//!     let output_stream = device.build_output_stream(
//!         &config.into(),
//!         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//!             // Provide the ticks to advance per iteration for the internal scheduler.
//!             let ticks = calculate_ticks(output_channels, data.len() as i32);
//!
//!             // Here if we had an input buffer
//!             // we could have modified it to do pre-processing.
//!
//!             // To receive messages from the pd patch we need to read the ring buffers
//!             // filled by the pd patch repeatedly to check if there are messages there.
//!             // Audio callback is a nice place to do that.
//!             receive_messages_from_pd();
//!
//!             // Process audio, advance internal scheduler.
//!             libpd_rs::process::process_float(ticks, &[], data);
//!
//!             // Here we could have done post processing after
//!             // pd processed our output buffer in place.
//!         },
//!         |err| eprintln!("an error occurred on stream: {}", err),
//!     )?;
//!
//!     // Turn audio processing on
//!     pd.activate_audio(true)?;
//!
//!     // Run the stream
//!     output_stream.play()?;
//!
//!     // This program does not terminate.
//!     // You would need to explicitly quit it.
//!     loop {
//!         // We sample in 2 hz.
//!         std::thread::sleep(std::time::Duration::from_millis(500));
//!
//!         // Read the average load of the cpu.
//!         let load = loadavg()?;
//!
//!         let one_minute_cpu_load_average = load.one;
//!         let five_minutes_cpu_load_average = load.five;
//!
//!         // Lists are one of the types we can send to pd.
//!         // Although pd allows for heterogeneous lists,
//!         // even if we're not using them heterogeneously in this example,
//!         // we still need to send it as a list of Atoms.
//!
//!         // Atom is an encapsulating type in pd to unify
//!         // various types of data together under the same umbrella.
//!         // Check out `libpd_rs::types` module for more details.
//!
//!         // Atoms have From trait implemented for them for
//!         // floats and strings.
//!         send_list_to(
//!             "cpu_load",
//!             &[
//!                 one_minute_cpu_load_average.into(),
//!                 five_minutes_cpu_load_average.into(),
//!             ],
//!         )?;
//!     }
//! }
//!```
//!
//! The one minute average load is controlling the center frequency of the `vcf~` and the speed of the pulses.
//! The five minute average load is controlling the q factor of the `vcf~`.
//!
//! The result is a pulse which goes higher in pitch and speed when the load is higher and vice versa.
//!
//! Though not very interesting musically, hope it triggers your curiosity and imagination.
//!
//! The patch you have just evaluated and listened looks like this in pd desktop [application](https://puredata.info/downloads).
//!
//! ![Pd patch with phasor a/d and vcf][phasor_patch]
//!
//! ### Note about examples
//!
//! After these basic initial examples which were aimed to get you started, you may dive into the
//! individual [modules](https://docs.rs/libpd-rs/0.1.9/libpd_rs/#modules)
//! and [items](https://docs.rs/libpd-rs/0.1.9/libpd_rs/all.html) in the documentation.
//! They all have their own examples.
//!
//! You may discover [integration tests](https://github.com/alisomay/libpd-rs/tests)
//! and explore [examples](https://github.com/alisomay/libpd-rs/examples)
//! in the [repository](https://github.com/alisomay/libpd-rs).
//!
//! The [examples](https://github.com/alisomay/libpd-rs/examples)
//! directory in the [repository](https://github.com/alisomay/libpd-rs) is not filled with
//! all the examples imagined yet.
//!
//! On the other hand it'll be updated with variety of
//! [new examples](https://github.com/alisomay/libpd-rs/issues/8) very soon.
//!
//! Enjoy!
//!
//! ## Things to note
//!
//! [libpd](https://github.com/libpd/libpd) is a C library and the implementation allocates libpd **globally**.
//! This means that you can only have one instance of libpd running at a time.
//! There is support for [multi instances](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.h#L529)
//! in [libpd](https://github.com/libpd/libpd) but those
//! will be implemented in [libpd-rs](https://github.com/alisomay/libpd-rs) in the future.
//!
//! This is not very Rust like and on top of that we do not manage the memory of that instantiation.
//! Using it from different threads is fine because [libpd](https://github.com/libpd/libpd) has
//! locking implemented inside it.
//!
//! On the other hand, the programming style and state tracking might be a little different that how we do it in Rust.
//!
//! There are many functions and less data structures in this crate.
//! Because all, of those functions act as methods for a globally
//! initialized singleton libpd instance.
//!
//! There are limited ways of retrieving state from the running libpd instance but libpd-rs provides
//! [`PdGlobal`](crate::convenience::PdGlobal) to track some of the
//! state manually on the other hand this operation needs to be handled with care.
//! See [`PdGlobal`](crate::convenience::PdGlobal) for more details.
//!
//! ## Plans and Support
//!
//! Please see [libpd-rs](https://github.com/alisomay/libpd-rs) [support](https://github.com/alisomay/libpd-rs#support) and
//! [road map](https://github.com/alisomay/libpd-rs#road-map) section for more details on these topics.
//!
//! Also don't forget to check [issues](https://github.com/alisomay/libpd-rs/issues), to track the ideas and plans.
//!
//! ## Last words
//!
//! Generative or algorithmic music is a powerful tool for exploration,
//! pumps up creativity and goes very well together with traditional music making approaches also.
//!
//! Making apps which produce meaningful sound is difficult,
//! I wish that this crate would ease your way on doing that and make complicated audio ideas
//! in apps accessible to more people.
//!
//! Don't forget to check the [resources](https://github.com/alisomay/libpd-rs#resources)
//! section to expand your knowledge about pure data.
//!
//! Many thanks to [Başak Ünal](https://basakunal.design) for the logo.
//!
//! Happy patching!
//!
//! ## License
//!
//! [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause).
//! See [LICENSE](https://raw.githubusercontent.com/alisomay/libpd-rs/main/LICENCE) file.

/// Work with pd arrays
///
/// This module provides all tools to work with pd named arrays which are exposed by libpd with some extra safety such as bounds checking.
///
/// Corresponding libpd functions in [libpd repository](https://github.com/libpd/libpd) could be explored [here](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.h#L115).
///
/// # Examples
///
/// ```rust
/// use libpd_rs::{
///     array::{array_size, read_float_array_from, resize_array, write_float_array_to},
///     close_patch,
///     init, initialize_audio, open_patch,
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     initialize_audio(1, 2, 44100)?;
///     
///     // Place your own patch here.
///     let handle = open_patch("tests/patches/array_sketch_pad.pd")?;
///
///     let size = array_size("sketch_pad")?;
///     
///     // Arrays are sized to 100 by default.
///     assert_eq!(size, 100);
///
///     // We can resize this array to 8.
///     resize_array("sketch_pad", 8)?;
///
///     let size = array_size("sketch_pad")?;
///     assert_eq!(size, 8);
///
///     // Let's write some stuff to our array.
///     write_float_array_to("sketch_pad", 0, &[1.0, 2.0, 3.0, 4.0], 4)?;
///
///     // Let's overwrite the second part of the array with the first part of our slice.
///     write_float_array_to("sketch_pad", 2, &[500.0, 600.0, 3.0, 4.0], 2)?;
///
///     // Now we can read the array to the second half of our slice.
///     let mut read_to: Vec<f32> = vec![0.0; 4];
///
///     // Read it all back.
///     read_float_array_from("sketch_pad", 0, 4, &mut read_to)?;
///     assert_eq!(read_to, [1.0, 2.0, 500.0, 600.0]);
///
///     // Now we can read the second half of our array to our first half of our slice.
///     read_float_array_from("sketch_pad", 2, 2, &mut read_to)?;
///     assert_eq!(read_to, [500.0, 600.0, 500.0, 600.0]);
///
///     close_patch(handle)?;
///
///     Ok(())
/// }
/// ```
pub mod array;
/// Convenience functions and types which encapsulate common actions when communicating with pd
///
/// `libpd-rs` is a safe wrapper around [`libpd`](https://github.com/libpd/libpd) which provides a convenient interface to pd for Rust ecosystem.
///
/// [`libpd`](https://github.com/libpd/libpd) internally initializes pd and provides a way to communicate with it but it does it in global scope.
/// On the Rust side we do not have access to that piece of memory and we need to use what we have from exposed APIs.
///
/// [`libpd`](https://github.com/libpd/libpd) does not provide a very detailed way to track the state of pd yet. We need our own state tracking to track which
/// senders we have subscribed, which paths we have registered to the list of pd search paths or if the dsp is running or not.
///
/// This module provides some types for this and some wrapper functions for easier communication with pd.
///
/// There is one thing to note though. The state tracked in the Rust side is **decoupled** from libpd's internal state.
///
/// User needs to be careful to design the application in a way that the state on the Rust side **always** reflects the state on the C side.
/// Check the struct [`PdGlobal`](crate::convenience::PdGlobal) for more details on this matter.
///
/// # Examples
/// ```rust
/// use libpd_rs::convenience::PdGlobal;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut pd = PdGlobal::init_and_configure(1, 2, 44100)?;
///      
///     // pd will keep track of the open patch.
///     pd.open_patch("tests/patches/sine.pd")?;
///      
///     // We may close the open patch safely any time.
///     pd.close_patch()?;
///
///     pd.open_patch("tests/patches/sine.pd")?;
///
///     // We may subscribe to senders
///     pd.subscribe_to_many(&["some_sender", "some_other_sender"])?;
///
///     // Unsubscribe from one or many
///     pd.unsubscribe_from("some_sender");
///
///     // Or all
///     pd.unsubscribe_from_all();
///
///     // Activate or deactivate audio
///     pd.activate_audio(true)?;
///
///     // Check if it is active
///     assert!(pd.audio_active());
///
///     // And more..
///     // Check `PdGlobal` type for all the things you may do with it,
///     // which includes examples and documentation.
///
///     Ok(())
/// }
/// ```
pub mod convenience;
/// All errors
///
/// This module contains all the errors which can be returned by the library.
pub mod error;
/// Start, stop, poll pd gui
///
/// This module provides functions to start, stop and poll pd gui.
///
/// If the pd desktop application is installed in your computer.
/// You may use these functions to launch or quit it.
pub mod gui;

/// Audio processing
///
/// Process functions which you call in your audio callback are collected here.
///
/// These functions also run the scheduler of pd. The chosen function needs to be called in a loop to keep pd "running".
///
/// # Examples
///
/// ```rust
/// use libpd_rs::{
///     close_patch,
///     init, initialize_audio, open_patch,
///     process::process_float,
///     convenience::{dsp_on, calculate_ticks},
///     receive::receive_messages_from_pd
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     initialize_audio(1, 2, 44100)?;
///     
///     // Place your own patch here.
///     let patch_handle = open_patch("tests/patches/sine.pd")?;
///
///     // Turn the dsp on
///     dsp_on()?;
///
///     // Process the audio (imagine that this is your audio callback)
///     // We're going to treat this separate thread as the
///     // high priority audio callback from the OS for this example.
///
///     // Open some channels to communicate with it.
///     let (tx, rx) = std::sync::mpsc::channel::<()>();
///
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///         
///         // Run pd
///         loop {
///             // Mimic the call frequency of the audio callback.
///             let approximate_buffer_duration =
///                 (output_buffer.len() as f32 / sample_rate as f32) * 1000.0;
///             std::thread::sleep(std::time::Duration::from_millis(
///                 approximate_buffer_duration as u64,
///             ));
///             
///             // We may call this to also receive from internal ring buffers in this loop.
///             // So our registered listeners receive messages.
///             receive_messages_from_pd();
///             
///             // Calculate ticks for the internal scheduler
///             let ticks = calculate_ticks(output_buffer.len() as i32, output_channels);
///             
///             // Process the audio and advance the internal scheduler by the number of ticks.
///             process_float(ticks, &input_buffer, &mut output_buffer);
///             
///             // This is just meaningful for this example,
///             // so we can break from this loop.
///             match rx.try_recv() {
///                 Ok(_) => break,
///                 _ => continue,
///             }
///         }
///      });
///
///     // When processing starts pd becomes alive because the scheduler is running.
///     
///     // After some time
///     std::thread::sleep(std::time::Duration::from_millis(1000));
///     
///     // We may join the thread.
///     tx.send(()).unwrap();
///     handle.join().unwrap();
///
///     // And close the patch
///     close_patch(patch_handle)?;
///
///     Ok(())
/// }
/// ```
pub mod process;
/// Receive messages from pd
///
/// Collection of endpoints where listeners (hooks) may be registered to receive messages from pd.
///
/// # Examples
/// ```rust
/// use libpd_rs::{
///     close_patch,
///     init, initialize_audio, open_patch,
///     process::process_float,
///     convenience::{dsp_on, calculate_ticks},
///     receive::{receive_messages_from_pd, on_print, on_float, start_listening_from}
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     initialize_audio(1, 2, 44100)?;
///     
///     // Place your own patch here.
///     let patch_handle = open_patch("tests/patches/echo.pd")?;
///
///     // Turn the dsp on
///     dsp_on()?;
///
///     // Register some listeners
///     // Print is a special one which is always listened from.
///     on_print(|value| {
///       println!("{value}");
///     });
///     
///     on_float(|source, value| {
///       println!("{value} received from {source}");
///     });
///
///     // For others we need to register them.
///     start_listening_from("float_from_pd")?;
///     
///     // We're going to treat this separate thread as the
///     // high priority audio callback from the OS for this example.
///
///     // Open some channels to communicate with it.
///     let (tx, rx) = std::sync::mpsc::channel::<()>();
///
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///         
///         // Run pd
///         loop {
///             // Mimic the call frequency of the audio callback.
///             let approximate_buffer_duration =
///                 (output_buffer.len() as f32 / sample_rate as f32) * 1000.0;
///             std::thread::sleep(std::time::Duration::from_millis(
///                 approximate_buffer_duration as u64,
///             ));
///             
///             // Receive messages from pd.
///             receive_messages_from_pd();
///             
///             // Calculate ticks for the internal scheduler
///             let ticks = calculate_ticks(output_buffer.len() as i32, output_channels);
///             
///             // Process the audio and advance the internal scheduler by the number of ticks.
///             process_float(ticks, &input_buffer, &mut output_buffer);
///             
///             // This is just meaningful for this example,
///             // so we can break from this loop.
///             match rx.try_recv() {
///                 Ok(_) => break,
///                 _ => continue,
///             }
///         }
///      });
///
///     // When processing starts pd becomes alive because the scheduler is running.
///     
///     // After some time
///     std::thread::sleep(std::time::Duration::from_millis(1000));
///     
///     // We may join the thread.
///     tx.send(()).unwrap();
///     handle.join().unwrap();
///
///     // And close the patch
///     close_patch(patch_handle)?;
///
///     Ok(())
/// }
/// ```
pub mod receive;
/// Send messages to pd
///
/// Collection of functions where messages may be sent to pd
///
/// # Examples
/// ```rust
/// use libpd_rs::{
///     close_patch,
///     init, initialize_audio, open_patch,
///     process::process_float,
///     convenience::{dsp_on, calculate_ticks},
///     receive::{receive_messages_from_pd, on_print, on_float, start_listening_from},
///     send::{send_float_to}
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     initialize_audio(1, 2, 44100)?;
///     
///     // Place your own patch here.
///     let patch_handle = open_patch("tests/patches/echo.pd")?;
///
///     // Turn the dsp on
///     dsp_on()?;
///
///     // Register some listeners
///     // Print is a special one which is always listened from.
///     on_print(|value| {
///       println!("{value}");
///     });
///     
///     on_float(|source, value| {
///       assert_eq!(source, "float_from_pd");
///       assert_eq!(value, 42.0);
///       println!("{value} received from {source}");
///     });
///
///     // For others we need to register them.
///     start_listening_from("float_from_pd")?;
///     
///     // We're going to treat this separate thread as the
///     // high priority audio callback from the OS for this example.
///
///     // Open some channels to communicate with it.
///     let (tx, rx) = std::sync::mpsc::channel::<()>();
///
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///         
///         // Run pd
///         loop {
///             // Mimic the call frequency of the audio callback.
///             let approximate_buffer_duration =
///                 (output_buffer.len() as f32 / sample_rate as f32) * 1000.0;
///             std::thread::sleep(std::time::Duration::from_millis(
///                 approximate_buffer_duration as u64,
///             ));
///             
///             // Receive messages from pd.
///             receive_messages_from_pd();
///             
///             // Calculate ticks for the internal scheduler
///             let ticks = calculate_ticks(output_buffer.len() as i32, output_channels);
///             
///             // Process the audio and advance the internal scheduler by the number of ticks.
///             process_float(ticks, &input_buffer, &mut output_buffer);
///             
///             // This is just meaningful for this example,
///             // so we can break from this loop.
///             match rx.try_recv() {
///                 Ok(_) => break,
///                 _ => continue,
///             }
///         }
///      });
///
///     // When processing starts pd becomes alive because the scheduler is running.
///
///     // Let's send a float to pd, it will be caught by the float listener,
///     // because our patch which we've loaded, echoes it back.
///     send_float_to("float_from_rust", 42.0)?;
///     
///     // After some time
///     std::thread::sleep(std::time::Duration::from_millis(1000));
///     
///     // We may join the thread.
///     tx.send(()).unwrap();
///     handle.join().unwrap();
///
///     // And close the patch
///     close_patch(patch_handle)?;
///
///     Ok(())
/// }
/// ```
pub mod send;
/// Types for working with pd
///
/// Pd wraps primitive types such as a float or a string in a type called atom.
/// This enables pd to have heterogenous lists.
///
/// This module exposes the representation of that type as a Rust enum, [`Atom`](crate::types::Atom).
///
/// It also exposes some others to hold file or receiver handles returned from libpd functions.
pub mod types;

pub(crate) mod helpers;
use error::{AudioInitializationError, PatchLifeCycleError};

use crate::{
    error::{InitializationError, IoError},
    types::PatchFileHandle,
};

use std::ffi::CString;
use std::path::{Path, PathBuf};

// TODO: Currently panicing is enough since this is a rare case, but may be improved later with a dedicated error.
pub(crate) const C_STRING_FAILURE: &str =
    "Provided an invalid CString, check if your string contains null bytes in the middle.";
pub(crate) const C_STR_FAILURE: &str = "Converting a CStr to an &str is failed.";

/// Initializes libpd.
///
/// This function should be called **before** any other in this crate.
/// It initializes libpd **globally** and also initializes ring buffers for internal message passing.
///
/// After setting internal hooks, it initializes `libpd` by calling the underlying
/// C function which is [`libpd_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.c#L68).
/// See [`libpd_queued_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/util/z_queued.c#L308) to
/// explore what it is doing.
///
/// **Note**: *Support for multi instances of pd is not implemented yet.*
///
/// # Example
/// ```rust
/// use libpd_rs::init;
///
/// assert_eq!(init().is_ok(), true);
/// assert_eq!(init().is_err(), true);
/// ```
///
/// # Errors
///
/// A second call to this function will return an error.
///
/// A list of errors that can occur:
/// - [`AlreadyInitialized`](crate::error::InitializationError::AlreadyInitialized)
/// - [`RingBufferInitializationError`](crate::error::InitializationError::RingBufferInitializationError)
/// - [`InitializationFailed`](crate::error::InitializationError::InitializationFailed)
pub fn init() -> Result<(), InitializationError> {
    unsafe {
        match libpd_sys::libpd_queued_init() {
            0 => Ok(()),
            -1 => Err(InitializationError::AlreadyInitialized),
            -2 => Err(InitializationError::RingBufferInitializationError),
            _ => Err(InitializationError::InitializationFailed),
        }
    }
}

/// Frees the internal queued ring buffers.
///
/// Currently I don't see a necessity to call this function in any case.
/// If you find a valid use case, please open an [issue](https://github.com/alisomay/libpd-rs/issues).
pub fn release_internal_queues() {
    unsafe {
        libpd_sys::libpd_queued_release();
    };
}

/// Clears all the paths where libpd searches for patches and assets.
///
/// This function is also called by [`init`].
pub fn clear_search_paths() {
    unsafe {
        libpd_sys::libpd_clear_search_path();
    }
}

/// Adds a path to the list of paths where pd searches in.
///
/// Relative paths are relative to the current working directory.
///
/// Unlike the desktop pd application, **no** search paths are set by default.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`PathDoesNotExist`](crate::error::IoError::PathDoesNotExist)
pub fn add_to_search_paths<T: AsRef<Path>>(path: T) -> Result<(), IoError> {
    if !path.as_ref().exists() {
        return Err(IoError::PathDoesNotExist(
            path.as_ref().to_string_lossy().to_string(),
        ));
    }
    unsafe {
        let c_path = CString::new(&*path.as_ref().to_string_lossy()).expect(C_STRING_FAILURE);
        libpd_sys::libpd_add_to_search_path(c_path.as_ptr());
        Ok(())
    }
}

/// Opens a pd patch.
///
/// The argument should be an absolute path to the patch file.
/// It would be useful to keep the return value of this function.
/// It can be used later to close it.
/// Absolute and relative paths are supported.
/// Relative paths and single file names are tried in executable directory and manifest directory.
///
/// Tha function **first** checks the executable directory and **then** the manifest directory.
///
/// # Examples
/// ```no_run
/// use libpd_rs::open_patch;
/// use std::path::PathBuf;
///
/// let absolute_path = PathBuf::from("/home/user/my_patch.pd");
/// let relative_path = PathBuf::from("../my_patch.pd");
/// let patch_name = PathBuf::from("my_patch.pd");
///
/// let patch_handle = open_patch(&patch_name).unwrap();
/// // or others..
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`FailedToOpenPatch`](crate::error::PatchLifeCycleError::FailedToOpenPatch)
/// - [`PathDoesNotExist`](crate::error::PatchLifeCycleError::PathDoesNotExist)
pub fn open_patch<T: AsRef<Path>>(
    path_to_patch: T,
) -> Result<PatchFileHandle, PatchLifeCycleError> {
    let file_name = path_to_patch
        .as_ref()
        .file_name()
        .ok_or(PatchLifeCycleError::FailedToOpenPatch)?;
    let file_name = file_name.to_string_lossy();
    let file_name = file_name.as_ref();
    let parent_path = path_to_patch
        .as_ref()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/"));
    let parent_path_string: String = parent_path.to_string_lossy().into();

    // Assume absolute path.
    let mut directory: String = parent_path_string.clone();

    if !parent_path.is_absolute() {
        // "../some.pd" --> prepend working directory
        if parent_path.is_relative() {
            let mut app_dir = std::env::current_exe()
                .map_err(|_| -> PatchLifeCycleError { PatchLifeCycleError::FailedToOpenPatch })?;
            app_dir.pop();
            app_dir.push(parent_path);
            let parent_path_str = app_dir.to_string_lossy();

            if app_dir.exists() {
                directory = parent_path_str.into();
            } else {
                let manifest_dir =
                    PathBuf::from(&std::env!("CARGO_MANIFEST_DIR")).join(parent_path);
                // Try manifest dir.
                let manifest_dir_str = manifest_dir.to_string_lossy();
                directory = manifest_dir_str.into();
            }
        }
        // "some.pd" --> prepend working directory
        if parent_path_string.is_empty() {
            let mut app_dir = std::env::current_exe()
                .map_err(|_| -> PatchLifeCycleError { PatchLifeCycleError::FailedToOpenPatch })?;
            app_dir.pop();
            app_dir.push(file_name);
            let parent_path_str = app_dir.to_string_lossy();

            if app_dir.exists() {
                directory = parent_path_str.into();
            } else {
                // Try manifest dir.
                directory = std::env!("CARGO_MANIFEST_DIR").into();
            }
        }
    }

    // Invalid path.
    let calculated_patch_path = PathBuf::from(&directory).join(file_name);
    if !calculated_patch_path.exists() {
        return Err(PatchLifeCycleError::PathDoesNotExist(
            calculated_patch_path.to_string_lossy().to_string(),
        ));
    }
    // All good.
    unsafe {
        let name = CString::new(file_name).expect(C_STRING_FAILURE);
        let directory = CString::new(directory).expect(C_STRING_FAILURE);
        let file_handle =
            libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr()).cast::<std::ffi::c_void>();
        if file_handle.is_null() {
            return Err(PatchLifeCycleError::FailedToOpenPatch);
        }
        Ok(file_handle.into())
    }
}

/// Closes a pd patch which has opened before.
///
/// Handle needs to point to a valid opened patch file.
///
/// # Examples
/// ```no_run
/// use std::path::PathBuf;
/// use libpd_rs::{open_patch, close_patch};
///
/// let patch = PathBuf::from("my_patch.pd");
/// let patch_handle = open_patch(&patch).unwrap();
///
/// assert!(close_patch(patch_handle).is_ok());
/// ```
/// # Errors
///
/// A list of errors that can occur:
/// - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
pub fn close_patch(handle: PatchFileHandle) -> Result<(), PatchLifeCycleError> {
    unsafe {
        let ptr: *mut std::ffi::c_void = handle.into();
        if ptr.is_null() {
            Err(PatchLifeCycleError::FailedToClosePatch)
        } else {
            libpd_sys::libpd_closefile(ptr);
            Ok(())
        }
    }
}

/// Gets the `$0` of the running patch.
///
/// `$0` id in pd could be thought as a auto generated unique identifier number for the patch.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`PatchIsNotOpen`](crate::error::PatchLifeCycleError::PatchIsNotOpen)
pub fn get_dollar_zero(handle: &PatchFileHandle) -> Result<i32, PatchLifeCycleError> {
    unsafe {
        match libpd_sys::libpd_getdollarzero(handle.as_mut_ptr()) {
            0 => Err(PatchLifeCycleError::PatchIsNotOpen),
            other => Ok(other),
        }
    }
}

/// Returns pd's fixed block size which is `64` by default.
///
/// The number of frames per 1 pd tick.
///
/// For every pd tick, pd will process frames by the amount of block size.
/// e.g. this would make 128 samples if we have a stereo output and the default block size.
///
/// It will first process the input buffers and then will continue with the output buffers.
/// Check the [`PROCESS`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.c#L177) macro in `libpd` [source](https://github.com/libpd/libpd/blob/master/libpd_wrapper) for more information.
///
/// # Examples
///
/// ```rust
/// use libpd_rs::block_size;
///
/// let block_size = block_size();
/// let output_channels = 2;
/// let buffer_size = 1024;
///  
/// // Calculate pd ticks according to the upper information
/// let pd_ticks = buffer_size / (block_size * output_channels);
///
/// // If we know about pd_ticks, then we can also calculate the buffer size,
/// assert_eq!(buffer_size, pd_ticks * (block_size * output_channels));
/// ```
#[must_use]
pub fn block_size() -> i32 {
    unsafe { libpd_sys::libpd_blocksize() }
}

/// Initializes audio rendering
///
/// This doesn't mean that the audio is actually playing.
/// To start audio processing please call [`dsp_on`](crate::convenience::dsp_on) function after the initialization.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`InitializationFailed`](crate::error::AudioInitializationError::InitializationFailed)
pub fn initialize_audio(
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
) -> Result<(), AudioInitializationError> {
    unsafe {
        match libpd_sys::libpd_init_audio(input_channels, output_channels, sample_rate) {
            0 => Ok(()),
            _ => Err(AudioInitializationError::InitializationFailed),
        }
    }
}

/// Sets the flag for the functionality of verbose printing to the pd console
pub fn verbose_print_state(active: bool) {
    if active {
        unsafe { libpd_sys::libpd_set_verbose(1) }
    } else {
        unsafe { libpd_sys::libpd_set_verbose(0) }
    }
}

/// Checks if verbose printing functionality to the pd console is active
#[must_use]
pub fn verbose_print_state_active() -> bool {
    unsafe { libpd_sys::libpd_get_verbose() == 1 }
}
