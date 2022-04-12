use crate::{
    error::{
        ArrayError, CommunicationError, InitializationError, IoError, LibpdError, SizeError,
        SubscriptionError,
    },
    helpers::{make_atom_list_from_t_atom_list, make_t_atom_list_from_atom_list},
    types::{Atom, PatchFileHandle, ReceiverHandle},
};

use libffi::high::{ClosureMut1, ClosureMut2, ClosureMut3, ClosureMut4};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

// TODO: Currently panicing is enough since this is a rare case, but may be improved later with a dedicated error.
const C_STRING_FAILURE: &str =
    "Provided an invalid CString, check if your string contains null bytes in the middle.";
const C_STR_FAILURE: &str = "Converting a CStr to an &str is failed.";

// TODO: Bring use statements visible in documentation.
// TODO: Also please, change the titles of each individual function to look nice in doc.
// TODO: We're compiling for f64 floats in pd, check how float and double functions behave and change documentation where necessary.

/// Initializes libpd.
///
/// Support for multi instances of pd is not implemented yet.
/// This function should be called before any other in this crate.
/// It initializes libpd globally and also initializes ring buffers for internal message passing.
/// Sets internal hooks. Then initializes `libpd` by calling the underlying
/// C function which is [`libpd_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.c#L68).
/// See [`libpd_queued_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/util/z_queued.c#L308) to
/// explore what it is doing.
///
/// A second call to this function will return an error.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::init;
///
/// assert_eq!(init().is_ok(), true);
/// assert_eq!(init().is_err(), true);
/// ```
pub fn init() -> Result<(), LibpdError> {
    unsafe {
        match libpd_sys::libpd_queued_init() {
            0 => Ok(()),
            -1 => Err(LibpdError::InitializationError(
                InitializationError::AlreadyInitialized,
            )),
            -2 => Err(LibpdError::InitializationError(
                InitializationError::RingBufferInitializationError,
            )),
            _ => Err(LibpdError::InitializationError(
                InitializationError::InitializationFailed,
            )),
        }
    }
}

#[allow(dead_code)]
/// Frees the queued ring buffers.
///
/// Currently I don't see a necessity to call this function in any case.
/// So it is kept private.
fn release_internal_queues() {
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

/// Adds a path to the list of paths where libpd searches in.
///
/// Relative paths are relative to the current working directory.
/// Unlike the desktop pd application, **no** search paths are set by default.
pub fn add_to_search_paths<T: AsRef<Path>>(path: T) -> Result<(), LibpdError> {
    if !path.as_ref().exists() {
        return Err(LibpdError::IoError(IoError::PathDoesNotExist(
            path.as_ref().to_string_lossy().to_string(),
        )));
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
/// # Example
/// ```no_run
/// use libpd_rs::mirror::open_patch;
/// use std::path::PathBuf;
///
/// let absolute_path = PathBuf::from("/home/user/my_patch.pd");
/// let relative_path = PathBuf::from("../my_patch.pd");
/// let patch_name = PathBuf::from("my_patch.pd");
///
/// let patch_handle = open_patch(&patch_name).unwrap();
/// // or others..
/// ```
pub fn open_patch<T: AsRef<Path>>(path_to_patch: T) -> Result<PatchFileHandle, LibpdError> {
    let file_name = path_to_patch
        .as_ref()
        .file_name()
        .ok_or(LibpdError::IoError(IoError::FailedToOpenPatch))?;
    let file_name = file_name.to_string_lossy();
    let file_name = file_name.as_ref();
    let parent_path = path_to_patch
        .as_ref()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/"));
    let parent_path_string: String = parent_path.to_string_lossy().into();

    // Assume absolute path.
    let mut directory: String = parent_path_string.clone();

    // "../some.pd" --> prepend working directory
    if parent_path.is_relative() {
        let mut app_dir =
            std::env::current_exe().map_err(|_| LibpdError::IoError(IoError::FailedToOpenPatch))?;
        app_dir.pop();
        app_dir.push(parent_path);
        let parent_path_str = app_dir.to_string_lossy();

        if app_dir.exists() {
            directory = parent_path_str.into();
        } else {
            let manifest_dir = PathBuf::from(&std::env!("CARGO_MANIFEST_DIR")).join(parent_path);
            // Try manifest dir.
            let manifest_dir_str = manifest_dir.to_string_lossy();
            directory = manifest_dir_str.into();
        }
    }
    // "some.pd" --> prepend working directory
    if parent_path_string.is_empty() {
        let mut app_dir =
            std::env::current_exe().map_err(|_| LibpdError::IoError(IoError::FailedToOpenPatch))?;
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

    // Invalid path.
    let calculated_patch_path = PathBuf::from(&directory).join(file_name);
    if !calculated_patch_path.exists() {
        return Err(LibpdError::IoError(IoError::FailedToOpenPatch));
        // TODO: Update returned errors when umbrella error type is in action.
        // return Err(IoError::PathDoesNotExist(
        //     path.to_string_lossy().to_string(),
        // ));
    }
    // All good.
    unsafe {
        let name = CString::new(file_name).expect(C_STRING_FAILURE);
        let directory = CString::new(directory).expect(C_STRING_FAILURE);
        dbg!(&name, &directory);
        let file_handle =
            libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr()).cast::<std::ffi::c_void>();
        if file_handle.is_null() {
            return Err(LibpdError::IoError(IoError::FailedToOpenPatch));
        }
        Ok(file_handle.into())
    }
}

/// Closes a pd patch.
///
/// Handle needs to point to a valid opened patch file.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use libpd_rs::mirror::{open_patch, close_patch};  
///
/// let patch = PathBuf::from("my_patch.pd");
/// let patch_handle = open_patch(&patch).unwrap();
///
/// assert!(close_patch(patch_handle).is_ok());
/// ```
pub fn close_patch(handle: PatchFileHandle) -> Result<(), LibpdError> {
    unsafe {
        let ptr: *mut std::ffi::c_void = handle.into();
        if ptr.is_null() {
            Err(LibpdError::IoError(IoError::FailedToClosePatch))
        } else {
            libpd_sys::libpd_closefile(ptr);
            Ok(())
        }
    }
}

/// Get the `$0` id of the running patch.
///
/// `$0` id in pd could be thought as a auto generated unique identifier for the patch.
pub fn get_dollar_zero(handle: &PatchFileHandle) {
    unsafe {
        libpd_sys::libpd_getdollarzero(handle.as_mut_ptr());
    }
}

/// Return pd's fixed block size which is 64 by default.
///
/// The number of frames per 1 pd tick.
///
/// For every pd tick, pd will process frames by the amount of block size.
/// e.g. this would make 128 samples if we have a stereo output and the default block size.
///
/// It will first process the input buffers and then will continue with the output buffers.
/// Check the `PROCESS` macro in `libpd.c` for more information.
///
/// # Examples
///
/// ```rust
/// use libpd_rs::mirror::block_size;
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

/// Initialize audio rendering
pub fn initialize_audio(
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
) -> Result<(), LibpdError> {
    unsafe {
        match libpd_sys::libpd_init_audio(input_channels, output_channels, sample_rate) {
            0 => Ok(()),
            _ => Err(LibpdError::InitializationError(
                InitializationError::AudioInitializationFailed,
            )),
        }
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `f32` interleaved audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::{process_float, block_size};
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f32; 512];
/// let mut output_buffer = [0.0_f32; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_float(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_float(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_float(ticks: i32, input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_float(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `i16` interleaved audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Float samples are converted to short by multiplying by 32767 and casting,
/// so any values received from pd patches beyond -1 to 1 will result in garbage.
///
/// Note: for efficiency, does *not* clip input
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::{process_short, block_size};
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0_i16; 512];
/// let mut output_buffer = [0_i16; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_short(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_short(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.

pub fn process_short(ticks: i32, input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_short(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `f64` interleaved audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::{process_double, block_size};
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f64; 512];
/// let mut output_buffer = [0.0_f64; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_double(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_double(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_double(ticks: i32, input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_double(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `f32` **non-interleaved** audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
/// Copies buffer contents to/from libpd without striping.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::process_raw;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f32; 512];
/// let mut output_buffer = [0.0_f32; 1024];
///
/// process_raw(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_raw(input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_raw(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `i16` **non-interleaved** audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
/// Copies buffer contents to/from libpd without striping.
///
/// Float samples are converted to short by multiplying by 32767 and casting,
/// so any values received from pd patches beyond -1 to 1 will result in garbage.
///
/// Note: for efficiency, does *not* clip input
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::process_raw_short;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0_i16; 512];
/// let mut output_buffer = [0_i16; 1024];
///
/// process_raw_short(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw_short(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.

pub fn process_raw_short(input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_raw_short(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Process the audio buffer in place through the loaded pd patch.
///
/// Processes `f64` **non-interleaved** audio buffers.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
/// Copies buffer contents to/from libpd without striping.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::mirror::process_raw_double;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f64; 512];
/// let mut output_buffer = [0.0_f64; 1024];
///
/// process_raw_double(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw_double(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_raw_double(input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_raw_double(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/* Array access */

/// Gets the size of an array by name in the pd patch which is running.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::array_size;
///
/// let size = array_size("my_array").unwrap();
/// ```
pub fn array_size<T: AsRef<str>>(name: T) -> Result<i32, LibpdError> {
    unsafe {
        let name = CString::new(name.as_ref()).expect(C_STRING_FAILURE);
        // Returns size or negative error code if non-existent
        let result = libpd_sys::libpd_arraysize(name.as_ptr());
        if result >= 0 {
            return Ok(result);
        }
        Err(LibpdError::SizeError(SizeError::CouldNotDetermine))
    }
}

/// Resizes an array found by name in the pd patch which is running.
///
/// Sizes <= 0 are clipped to 1
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::{array_size, resize_array};
///
/// resize_array("my_array", 1024).unwrap();
/// let size = array_size("my_array").unwrap();
/// assert_eq!(size, 1024);
///
/// resize_array("my_array", 0).unwrap();
/// let size = array_size("my_array").unwrap();
/// assert_eq!(size, 1);
/// ```
pub fn resize_array<T: AsRef<str>>(name: T, size: i64) -> Result<(), LibpdError> {
    unsafe {
        let name = CString::new(name.as_ref()).expect(C_STRING_FAILURE);
        // returns 0 on success or negative error code if non-existent
        match libpd_sys::libpd_resize_array(name.as_ptr(), size) {
            0 => Ok(()),
            _ => Err(LibpdError::SizeError(SizeError::CouldNotDetermine)),
        }
    }
}

/// Reads values as much as `read_amount` from the array which is specified with the `source_name` argument
/// and writes them to `destination` starting at `destination_offset`.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::read_float_array_from;
///
/// let mut destination = [0.0_f32; 64];
/// read_float_array_from("my_array", 32, &mut destination, 32).unwrap();
/// ```
/// # Errors
/// This function performs no bounds checking on the destination.
///
/// If `destination_offset` + `read_amount` is greater than the size of the `destination` or
/// the array which we're trying to read from is not existent it will return an error.
pub fn read_float_array_from<T: AsRef<str>>(
    source_name: T,
    read_amount: i32,
    destination: &mut [f32],
    destination_offset: i32,
) -> Result<(), LibpdError> {
    unsafe {
        let name = CString::new(source_name.as_ref()).expect(C_STRING_FAILURE);
        // Returns 0 on success or a negative error code if the array is non-existent
        // or offset + n exceeds range of array
        match libpd_sys::libpd_read_array(
            destination.as_mut_ptr(),
            name.as_ptr(),
            destination_offset,
            read_amount,
        ) {
            0 => Ok(()),
            -2 => Err(LibpdError::ArrayError(ArrayError::OutOfBounds)),
            // TODO: See if this is true..
            _ => Err(LibpdError::ArrayError(ArrayError::NonExistent)),
        }
    }
}

/// Reads values as much as `read_amount` from the array which is given as the `source` argument
/// and writes them to a named array in pd which is specified with `destination_name` argument starting at `destination_offset`.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::write_float_array_to;
///
/// let mut source = [1.0_f32; 64];
/// write_float_array_to("my_array", 32, &source, 32).unwrap();
/// ```
/// # Errors
/// This function performs no bounds checking on the destination.
///
/// If `destination_offset` + `read_amount` is greater than the size of the `destination` or
/// the array which we're trying to read from is not existent it will return an error.
pub fn write_float_array_to<T: AsRef<str>>(
    destination_name: T,
    destination_offset: i32,
    source: &[f32],
    read_amount: i32,
) -> Result<(), LibpdError> {
    unsafe {
        let name = CString::new(destination_name.as_ref()).expect(C_STRING_FAILURE);
        // Returns 0 on success or a negative error code if the array is non-existent
        // or offset + n exceeds range of array
        match libpd_sys::libpd_write_array(
            name.as_ptr(),
            destination_offset,
            source.as_ptr(),
            read_amount,
        ) {
            0 => Ok(()),
            -2 => Err(LibpdError::ArrayError(ArrayError::OutOfBounds)),
            // TODO: See if this is true..
            _ => Err(LibpdError::ArrayError(ArrayError::NonExistent)),
        }
    }
}

/// Reads values as much as `read_amount` from the array which is specified with the `source_name` argument
/// and writes them to `destination` starting at `destination_offset`.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::read_double_array_from;
///
/// let mut destination = [0.0_f64; 64];
/// read_double_array_from("my_array", 32, &mut destination, 32).unwrap();
/// ```
/// # Errors
/// This function performs no bounds checking on the destination.
///
/// If `destination_offset` + `read_amount` is greater than the size of the `destination` or
/// the array which we're trying to read from is not existent it will return an error.
pub fn read_double_array_from<T: AsRef<str>>(
    source_name: T,
    read_amount: i32,
    destination: &mut [f64],
    destination_offset: i32,
) -> Result<(), LibpdError> {
    unsafe {
        let name = CString::new(source_name.as_ref()).expect(C_STRING_FAILURE);
        // Returns 0 on success or a negative error code if the array is non-existent
        // or offset + n exceeds range of array
        match libpd_sys::libpd_read_array_double(
            destination.as_mut_ptr(),
            name.as_ptr(),
            destination_offset,
            read_amount,
        ) {
            0 => Ok(()),
            -2 => Err(LibpdError::ArrayError(ArrayError::OutOfBounds)),
            // TODO: See if this is true..
            _ => Err(LibpdError::ArrayError(ArrayError::NonExistent)),
        }
    }
}

/// Reads values as much as `read_amount` from the array which is given as the `source` argument
/// and writes them to a named array in pd which is specified with `destination_name` argument starting at `destination_offset`.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::write_double_array_to;
///
/// let source = [1.0_f64; 64];
/// write_double_array_to("my_array", 32, &source, 32).unwrap();
/// ```
/// # Errors
/// This function performs no bounds checking on the destination.
///
/// If `destination_offset` + `read_amount` is greater than the size of the `destination` or
/// the array which we're trying to read from is not existent it will return an error.
pub fn write_double_array_to<T: AsRef<str>>(
    destination_name: T,
    destination_offset: i32,
    source: &[f64],
    read_amount: i32,
) -> Result<(), LibpdError> {
    unsafe {
        let name = CString::new(destination_name.as_ref()).expect(C_STRING_FAILURE);
        // Returns 0 on success or a negative error code if the array is non-existent
        // or offset + n exceeds range of array
        match libpd_sys::libpd_write_array_double(
            name.as_ptr(),
            destination_offset,
            source.as_ptr(),
            read_amount,
        ) {
            0 => Ok(()),
            -2 => Err(LibpdError::ArrayError(ArrayError::OutOfBounds)),
            // TODO: See if this is true..
            _ => Err(LibpdError::ArrayError(ArrayError::NonExistent)),
        }
    }
}

/// Sends a `bang` to the pd receiver object specified in `receiver` the argument
///
/// `send_bang_to("foo")` will send a bang to `|s foo|` on the next tick.
/// The `bang` can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::send_bang_to;
///
/// // Handle the error if the receiver object is not found
/// send_bang_to("foo").unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_bang_to("foo");
/// ```
pub fn send_bang_to<T: AsRef<str>>(receiver: T) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_bang(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Sends an `f32` value to the pd receiver object specified in the `receiver` argument
///
/// `send_float_to("foo", 1.0)` will send the `f32` value to `|s foo|` on the next tick.
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::send_float_to;
///
/// // Handle the error if the receiver object is not found
/// send_float_to("foo", 1.0).unwrap_or_else(|err| {
///   dbg!("{}", err);
/// });
/// // or don't care..
/// let _ = send_float_to("foo", 1.0);
/// ```
pub fn send_float_to<T: AsRef<str>>(receiver: T, value: f32) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_float(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Sends an `f64` value to the pd receiver object specified in the `receiver` argument
///
/// `send_double_to("foo", 1.0)` will send the `f64` value to `|s foo|` on the next tick.
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::send_double_to;
///
/// // Handle the error if the receiver object is not found
/// send_double_to("foo", 1.0).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_double_to("foo", 1.0);
/// ```
pub fn send_double_to<T: AsRef<str>>(receiver: T, value: f64) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_double(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Sends a symbol to the pd receiver object specified in the `receiver` argument
///
/// `send_symbol_to("foo", "bar")` will send the symbol value to `|s foo|` on the next tick.
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::send_symbol_to;
///
/// // Handle the error if the receiver object is not found
/// send_symbol_to("foo", "bar").unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_symbol_to("foo", "bar");
/// ```
pub fn send_symbol_to<T: AsRef<str>, S: AsRef<str>>(
    receiver: T,
    value: S,
) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    let sym = CString::new(value.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_symbol(recv.as_ptr(), sym.as_ptr()) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Start composition of a new list or typed message of up to max **element** length
///
/// Messages can be of a smaller length as max length is only an upper bound.
/// No cleanup is required for unfinished messages.
/// Returns error if the length is too large.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   // Add some values to the message..
/// }
/// ```
pub fn start_message(length: i32) -> Result<(), LibpdError> {
    unsafe {
        match libpd_sys::libpd_start_message(length) {
            0 => Ok(()),
            _ => Err(LibpdError::SizeError(SizeError::TooLarge)),
        }
    }
}

/// Add an `f32` to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message, add_float_to_started_message};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_float_to_started_message(42.0);
/// }
/// ```
///
/// # Panics
/// To be honest I'd expect this to panic if you overflow a message buffer.
///
/// Although I didn't check that, please let me know if this is the case.
pub fn add_float_to_started_message(value: f32) {
    unsafe {
        libpd_sys::libpd_add_float(value);
    }
}

/// Add an `f64` to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message, add_double_to_started_message};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_double_to_started_message(42.0);
/// }
/// ```
///
/// # Panics
/// To be honest I'd expect this to panic if you overflow a message buffer.
///
/// Although I didn't check that, please let me know if this is the case.
pub fn add_double_to_started_message(value: f64) {
    unsafe {
        libpd_sys::libpd_add_double(value);
    }
}

/// Add a symbol to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message, add_symbol_to_started_message};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_symbol_to_started_message("foo");
/// }
/// ```
///
/// # Panics
/// To be honest I'd expect this to panic if you overflow a message buffer.
///
/// Although I didn't check that, please let me know if this is the case.
pub fn add_symbol_to_started_message<T: AsRef<str>>(value: T) {
    let sym = CString::new(value.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        libpd_sys::libpd_add_symbol(sym.as_ptr());
    }
}

/// Finish the current message and send as a list to a receiver in the loaded pd patch
///
/// The following example will send a list `42.0 bar` to `|s foo|` on the next tick.
/// The list can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message, add_symbol_to_started_message, add_float_to_started_message, finish_message_as_list_and_send_to};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 2;
/// if start_message(message_length).is_ok() {
///   add_float_to_started_message(42.0);
///   add_symbol_to_started_message("bar");
///   finish_message_as_list_and_send_to("foo").unwrap_or_else(|err| {
///      println!("{}", err);
///   });
/// }
/// ```
pub fn finish_message_as_list_and_send_to<T: AsRef<str>>(receiver: T) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_finish_list(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Finish the current message and send as a typed message to a receiver in the loaded pd patch
///
/// Typed message handling currently only supports up to 4 elements
/// internally in pd, **additional elements may be ignored.**
///
/// The following example will send a message `; pd dsp 1` on the next tick.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_message, add_float_to_started_message, finish_message_as_typed_message_and_send_to};
///
/// init();
///
/// // Arbitrary length
/// let message_length = 1;
/// if start_message(message_length).is_ok() {
///   add_float_to_started_message(1.0);
///   finish_message_as_typed_message_and_send_to("pd","dsp").unwrap_or_else(|err| {
///      println!("{}", err);
///   });
/// }
/// ```
pub fn finish_message_as_typed_message_and_send_to<T: AsRef<str>, S: AsRef<str>>(
    receiver: T,
    message_header: S,
) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    let msg = CString::new(message_header.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        match libpd_sys::libpd_finish_message(recv.as_ptr(), msg.as_ptr()) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Send a list to a receiver in the loaded pd patch
///
/// The following example will send a list `42.0 bar` to `|s foo|` on the next tick.
/// The list can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_list_to};
/// use libpd_rs::types::Atom;
///
/// init();
///
/// let list = vec![Atom::from(42.0), Atom::from("bar")];
/// // Handle the error if the receiver object is not found
/// send_list_to("foo", &list).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_list_to("foo", &list);
/// ```

pub fn send_list_to<T: AsRef<str>>(receiver: T, list: &[Atom]) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);

    unsafe {
        let mut atom_list: Vec<libpd_sys::t_atom> = make_t_atom_list_from_atom_list!(list);
        let atom_list_slice = atom_list.as_mut_slice();

        #[allow(clippy::cast_possible_wrap)]
        #[allow(clippy::cast_possible_truncation)]
        match libpd_sys::libpd_list(
            recv.as_ptr(),
            // This is fine since a list will not be millions of elements long and not negative for sure.
            list.len() as i32,
            atom_list_slice.as_mut_ptr(),
        ) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Send a typed message to a receiver in the loaded pd patch
///
/// The following example will send a typed message `dsp 1` to the receiver `pd` on the next tick.
/// The equivalent of this example message would have looked like `[; pd dsp 1]` in pd gui.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_message_to};
/// use libpd_rs::types::Atom;
///
/// init();
///
/// let values = vec![Atom::from(1.0)];
/// // Handle the error if the receiver object is not found
/// send_message_to("pd", "dsp", &values).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_message_to("pd", "dsp", &values);
/// ```
pub fn send_message_to<T: AsRef<str>>(
    receiver: T,
    message: T,
    list: &[Atom],
) -> Result<(), LibpdError> {
    let recv = CString::new(receiver.as_ref()).expect(C_STRING_FAILURE);
    let msg = CString::new(message.as_ref()).expect(C_STRING_FAILURE);
    unsafe {
        let mut atom_list: Vec<libpd_sys::t_atom> = make_t_atom_list_from_atom_list!(list);
        let atom_list_slice = atom_list.as_mut_slice();

        #[allow(clippy::cast_possible_wrap)]
        #[allow(clippy::cast_possible_truncation)]
        match libpd_sys::libpd_message(
            recv.as_ptr(),
            msg.as_ptr(),
            // This is fine since a list will not be millions of elements long and not negative for sure.
            list.len() as i32,
            atom_list_slice.as_mut_ptr(),
        ) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::MissingDestination(receiver.as_ref().to_owned()),
            )),
        }
    }
}

/// Subscribes to messages sent to a receiver in the loaded pd patch
///
/// `start_listening_from("foo")` would add a **virtual** `|r foo|` which would
/// forward messages to the libpd message listeners.
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
/// use libpd_rs::mirror::{init, start_listening_from};
/// use libpd_rs::types::ReceiverHandle;
///
/// init();
///
/// let sources = vec!["foo", "bar"];
/// // Maybe you would like to use the receiver handles later so you may store them..
/// let mut handles: HashMap<String, ReceiverHandle> = HashMap::new();
/// for source in sources {
///     start_listening_from(&source).map_or_else(|err| {
///         // Handle the error if there is no source to listen from
///         dbg!(err);
///     }, |handle| {
///         // Start listening from a source and keep the handle for later
///         handles.insert(source.to_string(), handle);
///     });
/// }
/// ```
pub fn start_listening_from<T: AsRef<str>>(sender: T) -> Result<ReceiverHandle, LibpdError> {
    let send = CString::new(sender.as_ref()).expect(C_STRING_FAILURE);

    unsafe {
        let handle = libpd_sys::libpd_bind(send.as_ptr());
        if handle.is_null() {
            Err(LibpdError::SubscriptionError(
                SubscriptionError::FailedToSubscribeToSender(sender.as_ref().to_owned()),
            ))
        } else {
            Ok(ReceiverHandle::from(handle))
        }
    }
}

/// Unsubscribes from messages sent to the receiver in the loaded pd patch
///
///`stop_listening_from("foo")` would **remove** the virtual `|r foo|`.
///  
/// Then, messages can not be received from this receiver anymore.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, start_listening_from, stop_listening_from};
///
/// init();
///
/// let receiver_handle = start_listening_from("foo").unwrap();
/// stop_listening_from(receiver_handle);
/// ```
pub fn stop_listening_from(source: ReceiverHandle) {
    let handle: *mut std::ffi::c_void = source.into();
    if handle.is_null() {
        return;
    }
    unsafe {
        libpd_sys::libpd_unbind(handle);
    }
}

// @attention Stayed here..

/// Check if a source to listen from exists.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, source_to_listen_from_exists, start_listening_from};
///
/// init();
///
/// if source_to_listen_from_exists("foo") {
///   if let Ok(receiver_handle) = start_listening_from("foo") {
///     // Do something with the handle..
///   }
/// }
/// ```
pub fn source_to_listen_from_exists<T: AsRef<str>>(sender: T) -> bool {
    let send = CString::new(sender.as_ref()).expect(C_STRING_FAILURE);
    unsafe { matches!(libpd_sys::libpd_exists(send.as_ptr()), 1) }
}

// Hooks / queued hooks / print hook utils

/// Sets a closure to be called when a message is written to the pd console.
///
/// There is also no prior call to `start_listening_from` to listen from pd console.
///  Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_print};
///
/// on_print(|msg: &str| {
///  println!("pd is printing: {msg}");
/// });
///
/// init();
/// ```
pub fn on_print<F: FnMut(&str) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(move |out: *const std::os::raw::c_char| {
        let out = unsafe { CStr::from_ptr(out).to_str().expect(C_STR_FAILURE) };
        user_provided_closure(out);
    }));
    let callback = ClosureMut1::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr1<*const i8, ()>)
            .cast::<unsafe extern "C" fn(*const i8)>()
    };
    std::mem::forget(callback);
    unsafe {
        // Always concatenate
        libpd_sys::libpd_set_queued_printhook(Some(libpd_sys::libpd_print_concatenator));
        libpd_sys::libpd_set_concatenated_printhook(Some(*ptr));
    };
}

/// Sets a closure to be called when a bang is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_bang, start_listening_from};
///
/// on_bang(|source: &str| {
///   match source {
///     "foo" => println!("bang from foo"),   
///     "bar" => println!("bang from bar"),
///      _ => unreachable!(),
///   }
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// ```
pub fn on_bang<F: FnMut(&str) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ =
        Box::leak(Box::new(move |source: *const std::os::raw::c_char| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            user_provided_closure(source);
        }));
    let callback = ClosureMut1::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr1<*const i8, ()>)
            .cast::<unsafe extern "C" fn(*const i8)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_banghook(Some(*ptr)) };
}

/// Sets a closure to be called when an `f32` is received from a subscribed receiver
///
/// You may either have `on_double` registered or `on_float` registered. **Not both**.
/// If you set both, the one you have set the latest will **overwrite the previously set one**.
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_float, start_listening_from};
///
/// on_float(|source: &str, value: f32| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {value}"),  
///     "bar" =>  println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// ```
pub fn on_float<F: FnMut(&str, f32) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char, float: f32| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            user_provided_closure(source, float);
        },
    ));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<*const i8, f32, ()>)
            .cast::<unsafe extern "C" fn(*const i8, f32)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_floathook(Some(*ptr)) };
}

/// Sets a closure to be called when an `f64` is received from a subscribed receiver
///
/// You may either have `on_double` registered or `on_float` registered. **Not both**.
/// If you set both, the one you have set the latest will **overwrite the previously set one**.
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_double, start_listening_from};
///
/// on_double(|source: &str, value: f64| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {value}"),  
///     "bar" =>  println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// ```
pub fn on_double<F: FnMut(&str, f64) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char, double: f64| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            user_provided_closure(source, double);
        },
    ));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<*const i8, f64, ()>)
            .cast::<unsafe extern "C" fn(*const i8, f64)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_doublehook(Some(*ptr)) };
}

/// Sets a closure to be called when a symbol is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_symbol, start_listening_from};
///
/// on_symbol(|source: &str, symbol: &str| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {symbol}"),  
///     "bar" =>  println!("Received a float from bar, value is: {symbol}"),
///      _ => unreachable!(),
///   }
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// ```
pub fn on_symbol<F: FnMut(&str, &str) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char, symbol: *const std::os::raw::c_char| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            let symbol = unsafe { CStr::from_ptr(symbol).to_str().expect(C_STR_FAILURE) };
            user_provided_closure(source, symbol);
        },
    ));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<*const i8, *const i8, ()>)
            .cast::<unsafe extern "C" fn(*const i8, *const i8)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_symbolhook(Some(*ptr)) };
}

/// Sets a closure to be called when a list is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_list, start_listening_from};
/// use libpd_rs::types::Atom;
///
/// on_list(|source: &str, list: &[Atom]| match source {
///     "foo" => {
///         for atom in list {
///             match atom {
///                 Atom::Float(value) => {
///                     println!("Received a float from foo, value is: {value}")
///                 }
///                 Atom::Symbol(value) => {
///                     println!("Received a symbol from foo, value is: {value}")
///                 }
///                 _ => unreachable!(),
///             }
///         }
///     }
///     "bar" => {
///         for atom in list {
///             match atom {
///                 Atom::Float(value) => {
///                     println!("Received a float from bar, value is: {value}")
///                 }
///                 Atom::Symbol(value) => {
///                     println!("Received a symbol from bar, value is: {value}")
///                 }
///                 _ => unreachable!(),
///             }
///         }
///     }
///     _ => unreachable!(),
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// ```
pub fn on_list<F: FnMut(&str, &[Atom]) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char,
              list_length: i32,
              atom_list: *mut libpd_sys::t_atom| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            // It is practically impossible that this list will have a negative size or a size of millions so this is safe.
            #[allow(clippy::cast_sign_loss)]
            let atom_list = unsafe { std::slice::from_raw_parts(atom_list, list_length as usize) };
            let atoms = make_atom_list_from_t_atom_list!(atom_list);
            user_provided_closure(source, &atoms);
        },
    ));
    let callback = ClosureMut3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr3<*const i8, i32, *mut libpd_sys::_atom, ()>)
            .cast::<unsafe extern "C" fn(*const i8, i32, *mut libpd_sys::_atom)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_listhook(Some(*ptr)) };
}

// TODO: Can we receive messages here without binding??
/// Sets a closure to be called when a typed message is received from a subscribed receiver
///
/// In a message like [; foo hello 1.0 merhaba] which is sent from the patch,
///
/// The arguments of the closure would be:
///
/// ```sh
/// source: "foo"
/// message: "hello"
/// values: [Atom::from(1.0), Atom::from("merhaba")]
/// ```
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_message, start_listening_from};
/// use libpd_rs::types::Atom;
///
/// on_message(|source: &str, message: &str, values: &[Atom]| match source {
///     "foo" => {
///         println!("Received a message from foo, message is: {message}");
///         for atom in values {
///             match atom {
///                 Atom::Float(value) => {
///                     println!("In message, {message}, a float value is: {value}")
///                 }
///                 Atom::Symbol(value) => {
///                     println!("In message, {message}, a symbol value is: {value}")
///                 }
///                 _ => unreachable!(),
///             }
///         }
///     }
///     _ => unreachable!(),
/// });
///
/// init();
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// ```
pub fn on_message<F: FnMut(&str, &str, &[Atom]) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char,
              message: *const std::os::raw::c_char,
              list_length: i32,
              atom_list: *mut libpd_sys::t_atom| {
            let source = unsafe { CStr::from_ptr(source).to_str().expect(C_STR_FAILURE) };
            let message = unsafe { CStr::from_ptr(message).to_str().expect(C_STR_FAILURE) };
            // It is practically impossible that this list will have a negative size or a size of millions so this is safe.
            #[allow(clippy::cast_sign_loss)]
            let atom_list = unsafe { std::slice::from_raw_parts(atom_list, list_length as usize) };
            let atoms = make_atom_list_from_t_atom_list!(atom_list);
            user_provided_closure(source, message, &atoms);
        },
    ));
    let callback = ClosureMut4::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr4<
            *const i8,
            *const i8,
            i32,
            *mut libpd_sys::_atom,
            (),
        >)
            .cast::<unsafe extern "C" fn(*const i8, *const i8, i32, *mut libpd_sys::_atom)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_messagehook(Some(*ptr)) };
}

/// Receive messages from pd message queue.
///
/// This should be called repeatedly in the **application's main loop** to fetch messages from pd.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::{start_listening_from, on_symbol, receive_messages_from_pd};
///
/// on_symbol(|source: &str, value: &str| {
///   match source {
///     "foo" => println!("Received a float from foo, value is: {value}"),   
///     "bar" => println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
///
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
///
/// loop {
///     receive_messages_from_pd();
/// }
/// ```
pub fn receive_messages_from_pd() {
    unsafe { libpd_sys::libpd_queued_receive_pd_messages() };
}

/// Sends a MIDI note on message to `|notein|` objects in pd.
///
/// Channel is 0-indexed, pitch is 0-127 and velocity is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: There is no note off message, send a note on with velocity = 0 instead.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_note_on};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_note_on(0, 48, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_note_on(0, 48, 64);
/// ```
pub fn send_note_on(channel: i32, pitch: i32, velocity: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_noteon(channel, pitch, velocity) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a MIDI control change message to `ctlin` objects in pd.
///
/// Channel is 0-indexed, controller is 0-127 and value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_control_change};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_control_change(0, 0, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_control_change(0, 0, 64);
/// ```
pub fn send_control_change(channel: i32, controller: i32, value: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_controlchange(channel, controller, value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a MIDI program change message to `pgmin` objects in pd.
///
/// Channel is 0-indexed, value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_program_change};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_program_change(0, 42).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_program_change(0, 42);
/// ```
pub fn send_program_change(channel: i32, value: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_programchange(channel, value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a MIDI pitch bend message to `|bendin|` objects in pd.
///
/// Channel is 0-indexed, value is -8192 to 8192.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: `|bendin|` outputs 0-16383 while `|bendout|` accepts -8192-8192
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_pitch_bend};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_pitch_bend(0, 8192).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_pitch_bend(0, 8192);
/// ```
pub fn send_pitch_bend(channel: i32, value: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_pitchbend(channel, value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a MIDI after touch message to `|touchin|` objects in pd.
///
/// Channel is 0-indexed, value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_aftertouch};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_aftertouch(0, 42).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_aftertouch(0, 42);
/// ```
pub fn send_aftertouch(channel: i32, value: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_aftertouch(channel, value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a MIDI poly after touch message to `|polytouchin|` objects in pd.
///
/// Channel is 0-indexed, pitch is 0-127 and value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_poly_aftertouch};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_poly_aftertouch(0, 48, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_poly_aftertouch(0, 48, 64);
/// ```
pub fn send_poly_aftertouch(channel: i32, pitch: i32, value: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_polyaftertouch(channel, pitch, value) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a raw MIDI byte to `|midiin|` objects in pd.
///
/// Port is 0-indexed and byte is 0-255
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_midi_byte};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_midi_byte(0, 0xFF).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_midi_byte(0, 0xFF);
/// ```
pub fn send_midi_byte(port: i32, byte: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_midibyte(port, byte) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a raw MIDI byte to `|sysexin|` objects in pd.
///
/// Port is 0-indexed and byte is 0-255
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_sysex};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_sysex(0, 0x7F).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_sysex(0, 0x7F);
/// ```
pub fn send_sysex(port: i32, byte: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_sysex(port, byte) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sends a raw MIDI byte to `|realtimein|` objects in pd.
///
/// Port is 0-indexed and byte is 0-255
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, send_sys_realtime};
///
/// init();
///
/// // Handle the error if the receiver object is not found
/// send_sys_realtime(0, 0x7F).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_sys_realtime(0, 0x7F);
/// ```
pub fn send_sys_realtime(port: i32, byte: i32) -> Result<(), LibpdError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_sysrealtime(port, byte) {
            0 => Ok(()),
            _ => Err(LibpdError::CommunicationError(
                CommunicationError::OutOfRange,
            )),
        }
    }
}

/// Sets a closure to be called when a MIDI note on event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, pitch is 0-127 and velocity is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note:
///  - There is no note off message, a note on message with velocity = 0 is used instead.
///  - Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_note_on};
///
/// init();
///
/// on_midi_note_on(|channel: i32, pitch: i32, velocity: i32| {
///   println!("Note On: channel {channel}, pitch {pitch}, velocity {velocity}");
/// });
/// ```
pub fn on_midi_note_on<F: FnMut(i32, i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ =
        Box::leak(Box::new(move |channel: i32, pitch: i32, velocity: i32| {
            user_provided_closure(channel, pitch, velocity);
        }));
    let callback = ClosureMut3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr3<i32, i32, i32, ()>).cast::<unsafe extern "C" fn(
            i32,
            i32,
            i32,
        )>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_noteonhook(Some(*ptr)) };
}

/// Sets a closure to be called when a MIDI control change event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, controller is 0-127 and value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_control_change};
///
/// init();
///
/// on_midi_control_change(|channel: i32, controller: i32, value: i32| {
///   println!("Control Change: channel {channel}, controller number {controller}, value {value}");
/// });
/// ```
pub fn on_midi_control_change<F: FnMut(i32, i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ = Box::leak(Box::new(
        move |channel: i32, controller: i32, value: i32| {
            user_provided_closure(channel, controller, value);
        },
    ));
    let callback = ClosureMut3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr3<i32, i32, i32, ()>).cast::<unsafe extern "C" fn(
            i32,
            i32,
            i32,
        )>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_controlchangehook(Some(*ptr)) };
}

/// Sets a closure to be called when a MIDI program change event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_program_change};
///
/// init();
///
/// on_midi_program_change(|channel: i32, value: i32| {
///   println!("Program Change: channel {channel}, program number {value}");
/// });
/// ```
pub fn on_midi_program_change<F: FnMut(i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<i32, i32, ()>)
            .cast::<unsafe extern "C" fn(i32, i32)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_programchangehook(Some(*ptr)) };
}

/// Sets a closure to be called when a MIDI pitch bend event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, value is -8192-8192.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note:
///  - `|bendin|` object in pd outputs 0-16383 while `|bendout|` accepts -8192 to +8192.
///  - Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_pitch_bend};
///
/// init();
///
/// on_midi_pitch_bend(|channel: i32, value: i32| {
///   println!("Pitch Bend: channel {channel}, bend amount {value}");
/// });
/// ```
pub fn on_midi_pitch_bend<F: FnMut(i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<i32, i32, ()>)
            .cast::<unsafe extern "C" fn(i32, i32)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_pitchbendhook(Some(*ptr)) };
}

/// Sets a closure to be called when a MIDI after touch event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_after_touch};
///
/// init();
///
/// on_midi_after_touch(|channel: i32, value: i32| {
///   println!("After Touch: channel {channel}, after touch amount {value}");
/// });
/// ```
pub fn on_midi_after_touch<F: FnMut(i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<i32, i32, ()>)
            .cast::<unsafe extern "C" fn(i32, i32)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_aftertouchhook(Some(*ptr)) };
}

/// Sets a closure to be called when a MIDI poly after touch event is received.
///
/// You do not need to register this listener explicitly.
///
/// Channel is 0-indexed, pitch is 0-127 and value is 0-127.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_poly_after_touch};
///
/// init();
///
/// on_midi_poly_after_touch(|channel: i32, pitch: i32, value: i32| {
///   println!("Poly After Touch: channel {channel}, pitch {pitch}, after touch amount {value}");
/// });
/// ```
pub fn on_midi_poly_after_touch<F: FnMut(i32, i32, i32) + Send + Sync + 'static>(
    mut user_provided_closure: F,
) {
    let closure: &'static mut _ =
        Box::leak(Box::new(move |channel: i32, pitch: i32, value: i32| {
            user_provided_closure(channel, pitch, value);
        }));
    let callback = ClosureMut3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr3<i32, i32, i32, ()>).cast::<unsafe extern "C" fn(
            i32,
            i32,
            i32,
        )>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_polyaftertouchhook(Some(*ptr)) };
}

/// Sets a closure to be called when a single raw MIDI byte is received.
///
/// You do not need to register this listener explicitly.
///
/// Port is 0-indexed and byte is 0-255
///
/// Note: Out of range values which are sent from the patch are clamped.
///
/// # Example
/// ```rust
/// use libpd_rs::mirror::{init, on_midi_byte};
///
/// init();
///
/// on_midi_byte(|port: i32, byte: i32| {
///   println!("Raw MIDI Byte: port {port}, byte {byte}");
/// });
/// ```
pub fn on_midi_byte<F: FnMut(i32, i32) + Send + Sync + 'static>(mut user_provided_closure: F) {
    let closure: &'static mut _ = Box::leak(Box::new(move |port: i32, byte: i32| {
        user_provided_closure(port, byte);
    }));
    let callback = ClosureMut2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe {
        &*(code as *const libffi::high::FnPtr2<i32, i32, ()>)
            .cast::<unsafe extern "C" fn(i32, i32)>()
    };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_queued_midibytehook(Some(*ptr)) };
}

/// Receive messages from pd midi message queue.
///
/// This should be called repeatedly in the **application's main loop** to fetch midi messages from pd.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::{on_midi_byte, receive_midi_messages_from_pd};
///
/// on_midi_byte(|port: i32, byte: i32| {
///     println!("{port}, {byte}");
/// });
///
/// loop {
///     receive_midi_messages_from_pd();
/// }
/// ```
pub fn receive_midi_messages_from_pd() {
    unsafe { libpd_sys::libpd_queued_receive_midi_messages() };
}

/// Opens the current patch within a pd vanilla GUI
///
/// This function requires that there is a valid pd installation in your computer and a path to the pd binary.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::start_gui;
/// use std::path::PathBuf;
///
/// // In mac os probably it would look something like this,
/// // The application name here is an example.
/// let path_to_pd = PathBuf::from("/Applications/Pd-0.51-4.app/Contents/Resources/bin/pd");
/// start_gui(&path_to_pd);
/// ```
pub fn start_gui<T: AsRef<Path>>(path_to_pd: T) -> Result<(), LibpdError> {
    if path_to_pd.as_ref().exists() {
        let path_to_pd = path_to_pd.as_ref().to_string_lossy();
        let path_to_pd = CString::new(path_to_pd.as_ref()).expect(C_STRING_FAILURE);
        unsafe {
            match libpd_sys::libpd_start_gui(path_to_pd.as_ptr()) {
                0 => return Ok(()),
                // TODO: This can be a different error.
                _ => return Err(LibpdError::IoError(IoError::FailedToOpenGui)),
            }
        }
    }
    Err(LibpdError::IoError(IoError::FailedToOpenGui))
}

/// Stops the current running pd vanilla GUI if it is running.
pub fn stop_gui() {
    unsafe { libpd_sys::libpd_stop_gui() };
}

/// Manually update and handle any GUI messages
///
/// This is called automatically when running any process function in the library. e.g. `process_float`.
///
/// Note:
/// - This also facilitates network message processing, etc so it can be useful to call repeatedly when idle for more throughput.
/// - Returns a Some(()) when the polled queue is not empty. In this case it might be desirable to keep polling until it is empty or up to some reasonable limit.
///
/// # Example
/// ```no_run
/// use libpd_rs::mirror::{init, poll_gui};
///
/// init();
///
/// loop {
///     while let Some(_) = poll_gui() {
///         // Do something
///     }
/// }
/// ```
#[must_use]
pub fn poll_gui() -> Option<()> {
    unsafe {
        match libpd_sys::libpd_poll_gui() {
            1 => Some(()),
            _ => None,
        }
    }
}

// @attention Multi instance features implementation is scheduled for later.

/* multiple instance functions in z_libpd.h  could have been here.. */

// @attention If there is a necessity emerges, I'll give time to implement them.

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
