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
///     functions::{
///         close_patch,
///         initialize_audio, open_patch,
///         process::process_float,
///         util::{dsp_on, calculate_ticks},
///         receive::receive_messages_from_pd,
///     },
///     instance::PdInstance,
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Automatically initializes the pd instance and sets it as the main instance for the thread.
///     let mut main_instance = PdInstance::new()?;
///     
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
///     let mut instance_in_callback = main_instance.clone();
///     
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///         
///         // Pd instances are tracked in a thread local way so we need to set it as the current instance for this thread.
///         // Handle the error properly in real life.
///         instance_in_callback.set_as_current();
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

/// Send messages to pd
///
/// Collection of functions where messages may be sent to pd
///
/// # Examples
/// ```rust
/// use libpd_rs::{
///     functions::{
///         close_patch,
///         init, initialize_audio, open_patch,
///         process::process_float,
///         util::{dsp_on, calculate_ticks},
///         receive::{receive_messages_from_pd, on_print, on_float, start_listening_from},
///         send::{send_float_to},
///     },
///     instance::PdInstance,  
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Automatically initializes the pd instance and sets it as the main instance for the thread.
///     let mut main_instance = PdInstance::new()?;
///
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
///     let mut instance_in_callback = main_instance.clone();
///
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///
///         // Pd instances are tracked in a thread local way so we need to set it as the current instance for this thread.
///         // Handle the error properly in real life.
///         instance_in_callback.set_as_current();
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

/// Receive messages from pd
///
/// Collection of endpoints where listeners (hooks) may be registered to receive messages from pd.
///
/// # Examples
/// ```rust
/// use libpd_rs::{
///     functions::{
///         close_patch,
///         initialize_audio, open_patch,
///         process::process_float,
///         util::{dsp_on, calculate_ticks},
///         receive::{receive_messages_from_pd, on_print, on_float, start_listening_from},
///     },
///     instance::PdInstance,
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Automatically initializes the pd instance and sets it as the main instance for the thread.
///     let mut main_instance = PdInstance::new()?;
///
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
///     let mut instance_in_callback = main_instance.clone();
///
///     let handle = std::thread::spawn(move || {
///         // Mimic audio callback buffers.
///         let input_buffer = [0.0f32; 512];
///         let mut output_buffer = [0.0f32; 1024];
///         
///         let output_channels = 2;
///         let sample_rate = 44100;
///
///         // Pd instances are tracked in a thread local way so we need to set it as the current instance for this thread.
///         // Handle the error properly in real life.
///         instance_in_callback.set_as_current();
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
///     functions::{
///         array::{array_size, read_float_array_from, resize_array, write_float_array_to},
///         close_patch,
///         initialize_audio, open_patch,
///     },
///     instance::PdInstance,
/// };
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Automatically initializes the pd instance and sets it as the main instance for the thread.
///     let mut main_instance = PdInstance::new()?;
///     
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

/// Start, stop, poll pd gui
///
/// This module provides functions to start, stop and poll pd gui.
///
/// If the pd desktop application is installed in your computer.
/// You may use these functions to launch or quit it.
pub mod gui;

use crate::error::{AudioInitializationError, PatchLifeCycleError, StringConversionError};

use crate::{
    error::{InitializationError, IoError},
    types::PatchFileHandle,
};

use std::ffi::{self, CString};
use std::path::{Path, PathBuf};
use std::{env, path};

/// Initializes libpd.
///
/// Initializes the currently set pd instance for the thread also initializes ring buffers for internal message passing.
/// Call this function as early as possible after either creating or setting the pd instance.
///
/// After setting internal hooks, it initializes `libpd` by calling the underlying
/// C function which is `libpd_queued_init` which could be found in `z_queued.c` in the libpd repository.
///
/// **Notes**:
///
/// - It is safe to call this function more than once.
/// - You would need to call this function once for each instance you create.
///   Using `PdInstance::new()` is recommended since it automatically initializes the pd instance.
/// - `libpd-rs` is compiled with multi instance support. So this function only effects the current set instance for the thread.
///
/// # Example
/// ```rust
/// use libpd_rs::functions::init;
///
/// assert_eq!(init().is_ok(), true);
/// assert_eq!(init().is_ok(), true);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`RingBufferInitializationError`](crate::error::InitializationError::RingBufferInitializationError)
/// - [`InitializationFailed`](crate::error::InitializationError::InitializationFailed)
pub fn init() -> Result<(), InitializationError> {
    unsafe {
        match libpd_sys::libpd_queued_init() {
            // Returns -1 if it is already initialized which we ignore.
            0 | -1 => Ok(()),
            -2 => Err(InitializationError::RingBufferInitializationError),
            _ => Err(InitializationError::InitializationFailed),
        }
    }
}

/// Frees the internal queued ring buffers.
///
/// This function needs to be called after you're done with the pd instance before freeing it.
///
/// Using [`PdInstance`](crate::instance::PdInstance) is recommended since it automatically frees the pd instance
/// and calls this function in its `Drop` implementation.
///
/// From libpd source code:
///
/// ```c
/// /// free the queued ringbuffers
/// /// with multiple instances, call before freeing each instance:
/// /// libpd_set_instance(pd1);
/// /// libpd_queued_release();
/// /// libpd_free_instance(pd1);
/// ```
pub fn release_internal_queues() {
    unsafe {
        libpd_sys::libpd_queued_release();
    };
}

/// Clears all the paths where libpd searches for patches and assets.
///
/// Initializing an instance also clears the search paths.
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
/// - [`StringConversion`](crate::error::IoError::StringConversion)
pub fn add_to_search_paths<T: AsRef<Path>>(path: T) -> Result<(), IoError> {
    if !path.as_ref().exists() {
        return Err(IoError::PathDoesNotExist(
            path.as_ref().to_string_lossy().to_string(),
        ));
    }
    unsafe {
        let c_path =
            CString::new(&*path.as_ref().to_string_lossy()).map_err(StringConversionError::from)?;
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
/// use libpd_rs::functions::open_patch;
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
/// - [`StringConversion`](crate::error::PatchLifeCycleError::StringConversion)
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
        .unwrap_or_else(|| path::Path::new("/"));
    let parent_path_string: String = parent_path.to_string_lossy().into();

    // Assume absolute path.
    let mut directory: String = parent_path_string.clone();

    if !parent_path.is_absolute() {
        // "../some.pd" --> prepend working directory
        if parent_path.is_relative() {
            let mut app_dir = env::current_exe()
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
            let mut app_dir = env::current_exe()
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
        let name = CString::new(file_name).map_err(StringConversionError::from)?;
        let directory = CString::new(directory).map_err(StringConversionError::from)?;

        let file_handle =
            libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr()).cast::<ffi::c_void>();

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
/// use libpd_rs::functions::{open_patch, close_patch};
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
        let ptr = handle.into_inner();
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
/// use libpd_rs::functions::block_size;
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
/// To start audio processing please call [`dsp_on`](crate::util::dsp_on) function after the initialization.
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

/// Useful utilities for working with pd abstracting common operations.
pub mod util {
    use crate::error::PdError;

    use super::block_size;
    use super::send;

    /// Activates audio in pd.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SendError`](crate::error::SendError)
    ///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
    /// - [`SizeError`](crate::error::SizeError)
    ///   - [`TooLarge`](crate::error::SizeError::TooLarge)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn dsp_on() -> Result<(), PdError> {
        send::start_message(1)?;
        send::add_float_to_started_message(1.0);
        send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
        Ok(())
    }

    /// De-activates audio in pd.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SendError`](crate::error::SendError)
    ///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
    /// - [`SizeError`](crate::error::SizeError)
    ///   - [`TooLarge`](crate::error::SizeError::TooLarge)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn dsp_off() -> Result<(), PdError> {
        send::start_message(1)?;
        send::add_float_to_started_message(0.0);
        send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
        Ok(())
    }

    /// Find the number of pd ticks according to the case.
    ///
    /// The calculation is `buffer_size / (block_size * channels)`
    #[expect(
        clippy::integer_division,
        reason = "This function is used to calculate the number of ticks. We don't need floating point precision here."
    )]
    pub fn calculate_ticks(channels: i32, buffer_size: i32) -> i32 {
        let block_size = block_size();
        buffer_size / (block_size * channels)
    }
}
