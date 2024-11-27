#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::allow_attributes_without_reason,
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
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,
    clippy::std_instead_of_alloc,
    clippy::question_mark_used,
    clippy::std_instead_of_core,
    clippy::partial_pub_fields,
    clippy::ref_patterns,
    clippy::semicolon_inside_block,
    clippy::semicolon_outside_block,
    clippy::pub_with_shorthand,
    clippy::self_named_module_files,
    clippy::integer_division_remainder_used,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::mem_forget,
    clippy::pub_use,

    // Expect is fine in relevant cases
    // clippy::expect_used,

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
//! libpd-rs = "0.3"
//! cpal = "0.15"
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
//! [evaluate](crate::Pd::eval_patch) strings as pd patches.
//!
//! This is the [method](crate::Pd::eval_patch) we'll use in the following examples.
//!
//! ### Initialize, open patch, run
//!
//! Paste the code into your `main.rs`:
//!
//! ⚠️ **Warning** ⚠️: This example will produce audio, so please keep your volume at a reasonable level for safety.
//!
//! ```no_run
//! use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//! use libpd_rs::{Pd, functions::util::calculate_ticks};
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
//!     let mut pd = Pd::init_and_configure(0, output_channels, sample_rate)?;
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
//!             libpd_rs::functions::process::process_float(ticks, &[], data);
//!
//!             // Here we could have done post processing
//!             // after pd processed our output buffer in place.
//!         },
//!         |err| eprintln!("an error occurred on stream: {}", err),
//!         None,
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
//! libpd-rs = "0.3"
//! cpal = "0.15"
//! sys-info = "0.9.1"
//! ```
//! Paste the code into your `main.rs`:
//!
//! ⚠️ **Warning** ⚠️: This example will produce audio, so please keep your volume at a reasonable level for safety.
//!
//! ```no_run
//! use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//! use libpd_rs::{
//!    Pd, functions::{util::calculate_ticks, receive::on_float, receive::receive_messages_from_pd, send::send_list_to}
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
//!     let mut pd = Pd::init_and_configure(0, output_channels, sample_rate)?;
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
//!             libpd_rs::functions::process::process_float(ticks, &[], data);
//!
//!             // Here we could have done post processing after
//!             // pd processed our output buffer in place.
//!         },
//!         |err| eprintln!("an error occurred on stream: {}", err),
//!         None,
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
//! ## Create Layers
//!
//! This crate is thought as 3 layers.
//!
//! ### Low Level
//! These are the sys bindings. You can use them directly if you'd like to through the re-export of [libpd-sys](https://github.com/alisomay/libpd-sys) from this crate.
//!
//! Since [libpd-rs](https://github.com/alisomay/libpd-rs) exhaustively wraps the sys layer, you don't need to use this layer directly.
//!
//! But if you wish, you can do so and create your own abstractions on top of it.
//!
//! ### Mid Level
//! These are the safe wrappers directly around the sys bindings and sometimes with slight extra functionality.
//!
//! You can reach this layer through the [functions](crate::functions) module.
//!
//! While a little bit more high level, here you would still need an understanding of [libpd](https://github.com/libpd) to use it effectively.
//!
//! This module is very heavily documented and tested so to learn further you can continue from here [functions](crate::functions).
//!
//! ### High Level
//! This layer consists of the [Pd](crate::Pd) struct, its methods and the types around it.
//!
//! Here we map one pure data instance to one [Pd](crate::Pd) instance.
//!
//! There are convenient instance management, drop implementations and other high level functionality here.
//!
//! This is the most advised way to use this library.
//!
//! ### Mixing Layers
//!
//! Low level and mid level layers can be used together since one is just a safe wrapper over the other.
//!
//! Mixed usage of mid and the high level layer requires significant understanding of the library and [libpd](https://github.com/libpd).
//!
//! So if you don't know what you're doing, it's advised to stick with the high level layer and not mix it with the others.
//!
//! The library tries to make this as convenient as possible.
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

/// TODO:
pub mod instance;

/// TODO:
pub mod functions;

/// Types for working with pd
///
/// Pd wraps primitive types such as a float or a string in a type called atom.
/// This enables pd to have heterogenous lists.
///
/// This module exposes the representation of that type as a Rust enum, [`Atom`](crate::types::Atom).
///
/// It also exposes some others to hold file or receiver handles returned from libpd functions.
pub mod types;

/// All errors
///
/// This module contains all the errors which can be returned by the library.
pub mod error;

pub(crate) mod helpers;

use error::PdError;
use libpd_sys::_pdinstance;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, ptr};
use tempfile::NamedTempFile;

use crate::instance::PdInstance;
use crate::{
    error::PatchLifeCycleError,
    types::{PatchFileHandle, ReceiverHandle},
};

/// Re-exports of the libpd-sys crate.
pub use libpd_sys;

/// An abstraction provided for convenience to express a pure data instance, track the state and execute some common functions.
///
/// This struct represents a single instance of pd.
///
/// After created and registered internally the instance lives in libpd's memory.
/// Dropping this struct will free the resources of the instance.
///
/// It is also important to note that the mid-level layer of this library [`functions`](crate::functions) also can modify the state of this instance.
///
/// To learn more about how instances are created and managed, please see the [`instance`](crate::instance) module level documentation.
///
/// This is why that if you're not very familiar with this library's and libpd's source code you shouldn't mix the high level layer with the mid level layer.
///
/// # Example of an unwanted mix
///
/// ```rust
/// use libpd_rs::Pd;
/// use libpd_rs::functions::util::dsp_off;
///
/// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
///
/// // We call the method of [`Pd`] to activate audio
/// // which calls [`dsp_on`] internally which then sends a message
/// // to globally initialized pd to activate dsp.
/// pd.activate_audio(true).unwrap();
///
/// // So far so good.
/// assert_eq!(pd.audio_active(), true);
///
/// // But we can send messages to globally initialized pd many ways
/// // and here is one of the ways we can do it.
/// dsp_off().unwrap();
///
/// // But now [`Pd`] is not aware of the mutated state
/// // of the instance in the background.
/// // The information it holds is outdated and not true anymore.
/// assert_eq!(pd.audio_active(), true);
/// ```
///
/// To avoid this situation if you use [`Pd`] check its methods, only use them and **not** their function counterparts.
///
/// If you really need to mix the layers, you should read the source of the relevant part before doing so.
pub struct Pd {
    inner: PdInstance,
    audio_active: bool,
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
    running_patch: Option<PatchFileHandle>,
    temporary_evaluated_patch: Option<NamedTempFile>,
    /// A store to keep track of subscriptions which are made to senders in pd through the app lifecycle.
    pub subscriptions: HashMap<String, ReceiverHandle>,
    /// A store to keep track of paths which are added to pd search paths through the app lifecycle.
    pub search_paths: Vec<PathBuf>,
}

impl Pd {
    // TODO: Document these functions maybe even add other impls exposing other functions through the instance.

    pub const fn inner(&self) -> &PdInstance {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut PdInstance {
        &mut self.inner
    }

    pub fn audio_context(&self) -> PdAudioContext {
        PdAudioContext {
            instance: self.inner.clone(),
        }
    }

    pub fn set_as_current(&mut self) {
        self.inner.set_as_current();
    }

    pub const fn instance_number(&self) -> i32 {
        self.inner.number()
    }

    pub fn is_main_instance(&self) -> bool {
        self.inner.is_main_instance()
    }

    pub fn is_current_instance(&self) -> bool {
        self.inner.is_current_instance()
    }

    pub(crate) fn set_as_active_instance(&mut self) -> ActiveInstanceGuard {
        if self.inner.is_current_instance() {
            // This would render the guard useless and as a no-op on drop which is what we want.
            return ActiveInstanceGuard::wrap(ptr::null_mut::<_pdinstance>());
        }
        let previous_instance = unsafe { libpd_sys::libpd_this_instance() };
        self.inner.set_as_current();
        ActiveInstanceGuard::wrap(previous_instance)
    }

    /// Initializes a pd instance.
    ///
    /// It calls [`PdInstance::new`](crate::instance::PdInstance::new) and [`initialize_audio`](crate::initialize_audio) with the provided arguments and returns an instance where a user can keep simple state and call some convenience methods.
    ///
    /// You may crate any number of instances of [`Pd`] but only one of them can be active at a time.
    /// Many of the methods in this struct would set it as the active instance before operating and reset it to the last set active after.
    ///
    /// It is your duty to keep this struct alive as long as you need to use it.
    /// When it goes out of scope, the instance will be dropped and the pd instance it wraps will be destroyed.
    ///
    /// # Examples
    /// ```rust
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`InitializationError`](crate::error::InitializationError)
    ///   - [`RingBufferInitializationError`](crate::error::InitializationError::RingBufferInitializationError)
    ///   - [`InitializationFailed`](crate::error::InitializationError::InitializationFailed)
    /// - [`AudioInitializationError`](crate::error::AudioInitializationError)
    ///   - [`InitializationFailed`](crate::error::AudioInitializationError::InitializationFailed)
    pub fn init_and_configure(
        input_channels: i32,
        output_channels: i32,
        sample_rate: i32,
    ) -> Result<Self, PdError> {
        let inner = PdInstance::new()?;
        functions::initialize_audio(input_channels, output_channels, sample_rate)?;
        Ok(Self {
            inner,
            audio_active: false,
            input_channels,
            output_channels,
            sample_rate,
            running_patch: None,
            temporary_evaluated_patch: None,
            subscriptions: HashMap::default(),
            search_paths: vec![],
        })
    }

    /// Adds a path to the list of paths where this instance searches in.
    ///
    /// Relative paths are relative to the current working directory.
    /// Unlike the desktop pd application, **no** search paths are set by default.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`IoError`](crate::error::IoError)
    ///   - [`PathDoesNotExist`](crate::error::IoError::PathDoesNotExist)
    pub fn add_path_to_search_paths<T: AsRef<Path>>(&mut self, path: T) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        let path = path.as_ref().to_path_buf();
        if !self.search_paths.contains(&path) {
            functions::add_to_search_paths(path.clone())?;
            self.search_paths.push(path);
        }
        Ok(())
    }

    /// Adds many paths to the list of paths where this instance searches in.
    ///
    /// Relative paths are relative to the current working directory.
    /// Unlike the desktop pd application, **no** search paths are set by default.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`IoError`](crate::error::IoError)
    ///   - [`PathDoesNotExist`](crate::error::IoError::PathDoesNotExist)
    pub fn add_paths_to_search_paths<T: AsRef<Path>>(
        &mut self,
        paths: &[T],
    ) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        for path in paths {
            if !self.search_paths.contains(&path.as_ref().to_path_buf()) {
                functions::add_to_search_paths(path)?;
                self.search_paths.push(path.as_ref().to_path_buf());
            }
        }
        Ok(())
    }

    /// Clears all the paths where this instance searches for patches and assets.
    pub fn clear_all_search_paths(&mut self) {
        let _guard = self.set_as_active_instance();
        functions::clear_search_paths();
        self.search_paths.clear();
    }

    /// Closes a pd patch for this instance.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    pub fn close_patch(&mut self) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        if let Some(handle) = self.running_patch.take() {
            functions::close_patch(handle)?;
        }
        self.temporary_evaluated_patch.take();
        Ok(())
    }

    /// Opens a pd patch for this instance.
    ///
    /// The argument should be an absolute path to the patch file.
    /// Absolute and relative paths are supported.
    /// Relative paths and single file names are tried in executable directory and manifest directory.
    ///
    /// Tha function **first** checks the executable directory and **then** the manifest directory.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    ///   - [`FailedToOpenPatch`](crate::error::PatchLifeCycleError::FailedToOpenPatch)
    ///   - [`PathDoesNotExist`](crate::error::PatchLifeCycleError::PathDoesNotExist)
    pub fn open_patch<T: AsRef<Path>>(&mut self, path: T) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        if self.running_patch.is_some() {
            self.close_patch()?;
        }
        self.running_patch = Some(functions::open_patch(path)?);
        Ok(())
    }

    /// Evaluate a string as a pd patch for this instance.
    ///
    /// This function creates a temporary file with the contents passed behind the scenes.
    /// and saves it into the [`Pd`] struct holding onto it until the patch is closed or the instantiated [`Pd`] is dropped.
    ///
    /// Note: The patch opened after this evaluation could be closed safely with [`close_patch`](Pd::close_patch).
    ///
    /// # Examples
    /// ```rust
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    ///     
    /// assert!(pd.eval_patch(
    /// r#"
    /// #N canvas 577 549 158 168 12;
    /// #X obj 23 116 dac~;
    /// #X obj 23 17 osc~ 440;
    /// #X obj 23 66 *~ 0.1;
    /// #X obj 81 67 *~ 0.1;
    /// #X connect 1 0 2 0;
    /// #X connect 1 0 3 0;
    /// #X connect 2 0 0 0;
    /// #X connect 3 0 0 1;
    /// "#
    /// ,).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToEvaluateAsPatch`](crate::error::PatchLifeCycleError::FailedToEvaluateAsPatch)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    ///   - [`FailedToOpenPatch`](crate::error::PatchLifeCycleError::FailedToOpenPatch)
    ///   - [`PathDoesNotExist`](crate::error::PatchLifeCycleError::PathDoesNotExist)
    pub fn eval_patch<T: AsRef<str>>(&mut self, contents: T) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        if self.running_patch.is_some() {
            self.close_patch()?;
        }
        let temp_file =
            NamedTempFile::new().map_err(|err| PatchLifeCycleError::FailedToEvaluateAsPatch {
                content: contents.as_ref().to_owned(),
                msg: err.to_string(),
            })?;
        fs::write(temp_file.path(), contents.as_ref()).map_err(|err| {
            PatchLifeCycleError::FailedToEvaluateAsPatch {
                content: contents.as_ref().to_owned(),
                msg: err.to_string(),
            }
        })?;
        self.running_patch = Some(functions::open_patch(temp_file.path())?);
        self.temporary_evaluated_patch = Some(temp_file);
        Ok(())
    }

    /// Starts listening messages from a source.
    ///
    /// If the source is already being listened to, this function will early return not doing anything without an error.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to("sender").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SubscriptionError`](crate::error::SubscriptionError)
    ///   - [`FailedToSubscribeToSender`](crate::error::SubscriptionError::FailedToSubscribeToSender)
    pub fn subscribe_to<T: AsRef<str>>(&mut self, source: T) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        if self.subscriptions.contains_key(source.as_ref()) {
            return Ok(());
        }
        self.subscriptions.insert(
            source.as_ref().to_owned(),
            functions::receive::start_listening_from(source.as_ref())?,
        );
        Ok(())
    }

    /// Starts listening messages from many sources.
    ///
    /// If the any source is already being listened to, this function will will ignore them.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SubscriptionError`](crate::error::SubscriptionError)
    ///   - [`FailedToSubscribeToSender`](crate::error::SubscriptionError::FailedToSubscribeToSender)
    pub fn subscribe_to_many<T: AsRef<str>>(&mut self, sources: &[T]) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        for source in sources {
            if self.subscriptions.contains_key(source.as_ref()) {
                continue;
            }
            self.subscriptions.insert(
                source.as_ref().to_owned(),
                functions::receive::start_listening_from(source.as_ref())?,
            );
        }
        Ok(())
    }

    /// Stops listening messages from a source.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to("sender").unwrap();
    /// pd.unsubscribe_from("sender");
    /// ```
    pub fn unsubscribe_from<T: AsRef<str>>(&mut self, source: T) {
        let _guard = self.set_as_active_instance();
        if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
            functions::receive::stop_listening_from(handle);
        }
    }

    /// Stops listening messages from many sources.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    ///
    /// pd.unsubscribe_from_many(&["sender", "other_sender"]);
    /// ```
    pub fn unsubscribe_from_many<T: AsRef<str>>(&mut self, sources: &[T]) {
        let _guard = self.set_as_active_instance();
        for source in sources {
            if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
                functions::receive::stop_listening_from(handle);
            }
        }
    }

    /// Stops listening from all sources.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::Pd;
    ///
    /// let mut pd = Pd::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    ///
    /// pd.unsubscribe_from_all();
    /// ```
    pub fn unsubscribe_from_all(&mut self) {
        let _guard = self.set_as_active_instance();
        let sources: Vec<String> = self.subscriptions.keys().cloned().collect();
        for source in &sources {
            if let Some(handle) = self.subscriptions.remove(source) {
                functions::receive::stop_listening_from(handle);
            }
        }
    }

    /// Gets the `$0` of the running patch.
    ///
    /// `$0` id in pd could be thought as a auto generated unique identifier for the patch.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`PatchIsNotOpen`](crate::error::PatchLifeCycleError::PatchIsNotOpen)
    pub fn dollar_zero(&mut self) -> Result<i32, PdError> {
        let _guard = self.set_as_active_instance();
        if let Some(ref patch) = self.running_patch {
            let dollar_zero = functions::get_dollar_zero(patch)?;
            return Ok(dollar_zero);
        }
        Err(PatchLifeCycleError::PatchIsNotOpen.into())
    }

    /// Checks if the audio is active.
    ///
    /// # Important
    ///
    /// The state is tracked by libpd internally for the instance and we expose other functions to modify this state.
    ///
    /// If messages sent to pd previously used another way to modify this information, this state might not reflect reality.
    pub const fn audio_active(&self) -> bool {
        self.audio_active
    }

    /// Activates or deactivates audio in pd.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SendError`](crate::error::SendError)
    ///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
    /// - [`SizeError`](crate::error::SizeError)
    ///   - [`TooLarge`](crate::error::SizeError::TooLarge)
    pub fn activate_audio(&mut self, on: bool) -> Result<(), PdError> {
        let _guard = self.set_as_active_instance();
        if on && !self.audio_active {
            functions::util::dsp_on()?;
            self.audio_active = true;
        } else if !on && self.audio_active {
            functions::util::dsp_off()?;
            self.audio_active = false;
        } else {
            return Ok(());
        }
        Ok(())
    }

    /// Gets the sample rate which pd is configured with.
    ///
    /// # Important
    ///
    /// The state is tracked by libpd internally for the instance and we expose other functions to modify this state.
    ///
    /// If messages sent to pd previously used another way to modify this information, this state might not reflect reality.
    #[must_use]
    pub const fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Gets the number of input channels which pd is configured with.
    ///
    /// # Important
    ///
    /// The state is tracked by libpd internally for the instance and we expose other functions to modify this state.
    ///
    /// If messages sent to pd previously used another way to modify this information, this state might not reflect reality.
    #[must_use]
    pub const fn input_channels(&self) -> i32 {
        self.input_channels
    }

    /// Gets the number of output channels which pd is configured with.
    ///
    /// # Important
    ///
    /// The state is tracked by libpd internally for the instance and we expose other functions to modify this state.
    ///
    /// If messages sent to pd previously used another way to modify this information, this state might not reflect reality.
    #[must_use]
    pub const fn output_channels(&self) -> i32 {
        self.output_channels
    }
}

/// TODO: Document
#[derive(Debug, Clone)]
pub struct PdAudioContext {
    instance: PdInstance,
}

impl PdAudioContext {
    pub fn receive_messages_from_pd(&mut self) {
        self.instance.set_as_current();
        functions::receive::receive_messages_from_pd();
    }

    pub fn receive_midi_messages_from_pd(&mut self) {
        self.instance.set_as_current();
        functions::receive::receive_midi_messages_from_pd();
    }

    pub fn process_float(&mut self, ticks: i32, input: &[f32], output: &mut [f32]) {
        self.instance.set_as_current();
        functions::process::process_float(ticks, input, output);
    }

    pub fn process_double(&mut self, ticks: i32, input: &[f64], output: &mut [f64]) {
        self.instance.set_as_current();
        functions::process::process_double(ticks, input, output);
    }

    pub fn process_short(&mut self, ticks: i32, input: &[i16], output: &mut [i16]) {
        self.instance.set_as_current();
        functions::process::process_short(ticks, input, output);
    }

    pub fn process_raw(&mut self, input: &[f32], output: &mut [f32]) {
        self.instance.set_as_current();
        functions::process::process_raw(input, output);
    }

    pub fn process_raw_short(&mut self, input: &[i16], output: &mut [i16]) {
        self.instance.set_as_current();
        functions::process::process_raw_short(input, output);
    }

    pub fn process_raw_double(&mut self, input: &[f64], output: &mut [f64]) {
        self.instance.set_as_current();
        functions::process::process_raw_double(input, output);
    }
}

struct ActiveInstanceGuard {
    previous_instance: *mut _pdinstance,
}

impl ActiveInstanceGuard {
    const fn wrap(previous_instance: *mut _pdinstance) -> Self {
        Self { previous_instance }
    }
}

impl Drop for ActiveInstanceGuard {
    fn drop(&mut self) {
        if self.previous_instance.is_null() {
            // TODO: Maybe inform the user about this?
            return;
        }

        unsafe {
            libpd_sys::libpd_set_instance(self.previous_instance);
        }
    }
}
