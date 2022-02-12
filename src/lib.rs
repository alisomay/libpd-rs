pub mod error;
pub mod pd;
pub mod types;
use error::{SizeError, SubscriptionError};
use types::ReceiverHandle;

use crate::error::{InitializationError, IoError, SendError};
use crate::types::{Atom, PatchFileHandle};
use libffi::high::{Closure1, Closure2, Closure3, Closure4};
use std::ffi::{CStr, CString};

/// Initializes libpd.
///
/// Support for multi instances of pd is not implemented yet.
/// This function should be called before any other in this crate.
/// It initializes libpd globally and it lives for 'static lifetime.
/// A second call to this function will return an error.
///
/// # Example
/// ```rust
/// assert_eq!(libpd_rs::init().is_ok(), true);
/// assert_eq!(libpd_rs::init().is_err(), true);
/// ```
pub fn init() -> Result<(), InitializationError> {
    unsafe {
        match libpd_sys::libpd_init() {
            0 => Ok(()),
            -1 => Err(InitializationError::AlreadyInitialized),
            _ => Err(InitializationError::InitializationFailed),
        }
    }
}

// TODO: Queued functions are not implemented yet. Find out if they are necessary.
// libpd_sys::libpd_queued_init;
// libpd_sys::libpd_queued_release;

// TODO: Reference init function in a right way.
/// Clears all the paths where libpd searches for patches and assets.
///
/// This function is also called by `init`.
pub fn clear_search_paths() {
    unsafe {
        libpd_sys::libpd_clear_search_path();
    }
}

/// Adds a path to the list of paths where libpd searches for patches and assets.
///
/// Relative paths are relative to the current working directory.
/// Unlike the desktop pd application, **no** search paths are set by default.
pub fn add_to_search_paths(path: &std::path::Path) {
    unsafe {
        let c_path = CString::new(&*path.to_string_lossy()).unwrap();
        libpd_sys::libpd_add_to_search_path(c_path.as_ptr());
    }
}

/// Opens a pd patch.
///
/// It would be useful to keep the return value of this function.
/// It can be used later to close it.
///
/// # Example
/// ```rust
/// let patch_handle = libpd_rs::open_patch("test.pd").unwrap();
/// ```
pub fn open_patch(path_to_patch: &std::path::Path) -> Result<PatchFileHandle, IoError> {
    if let Some(file_name) = path_to_patch.file_name() {
        if let Some(file_name) = file_name.to_str() {
            let parent_directory = path_to_patch
                .parent()
                .unwrap_or_else(|| std::path::Path::new("/"));
            if let Some(parent_directory) = parent_directory.to_str() {
                let name = CString::new(file_name).unwrap();
                let directory = CString::new(parent_directory).unwrap();
                unsafe {
                    let file_handle = libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr())
                        as *mut std::os::raw::c_void;
                    if file_handle.is_null() {
                        Err(IoError::FailedToOpenPatch)
                    } else {
                        Ok(PatchFileHandle::from(file_handle))
                    }
                }
            } else {
                Err(IoError::FailedToOpenPatch)
            }
        } else {
            Err(IoError::FailedToOpenPatch)
        }
    } else {
        Err(IoError::FailedToOpenPatch)
    }
}

/// Closes a pd patch.
///
/// # Example
/// ```rust
/// let patch_handle = libpd_rs::open_patch("test.pd").unwrap();
/// libpd_rs::close_patch(patch_handle);
/// ```
pub fn close_patch(handle: PatchFileHandle) {
    unsafe {
        // TODO: Can this file handle ever be null or invalidated?
        // This is a free, is there any way for a double free to occur?
        libpd_sys::libpd_closefile(handle.into_inner());
    }
}

// TODO: This function shouldn't consume PatchFileHandle.
// PatchFileHandle should be a smart pointer.

/// Get the `$0` id of the running patch.
///
/// `$0` id in pd could be thought as a auto generated unique identifier for the patch.
pub fn get_dollar_zero(handle: PatchFileHandle) {
    unsafe {
        libpd_sys::libpd_getdollarzero(handle.into_inner());
    }
}

// TODO: Maybe explain what a pd tick is?

/// Return pd's fixed block size
///
/// The number of sample frames per 1 pd tick
pub fn block_size() -> i32 {
    unsafe { libpd_sys::libpd_blocksize() }
}

/// Initialize audio rendering
pub fn initialize_audio(
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
) -> Result<(), InitializationError> {
    unsafe {
        match libpd_sys::libpd_init_audio(input_channels, output_channels, sample_rate) {
            0 => Ok(()),
            _ => Err(InitializationError::AudioInitializationFailed),
        }
    }
}

// TODO: Handle return? returns 0 on success
// TODO: Re-check formula
// TODO: Write an example
// TODO: Then write doc for all process functions accordingly.

/// Process the audio buffer in place through the loaded pd patch
///
/// Processes `f32` interleaved audio buffers.
/// The processing order is like the following, input_buffer -> libpd -> output_buffer
/// You may calculate the ticks argument with the following formula:
/// `(output_buffer.len() / block_size()) / channels`
/// `size = ticks * libpd_blocksize() * (in/out)channel`
pub fn process_float(ticks: i32, input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_float(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Document

/// process interleaved short samples from inBuffer -> libpd -> outBuffer
/// buffer sizes are based on # of ticks and channels where:
///     size = ticks * libpd_blocksize() * (in/out)channels
/// float samples are converted to short by multiplying by 32767 and casting,
/// so any values received from pd patches beyond -1 to 1 will result in garbage
/// note: for efficiency, does *not* clip input
/// returns 0 on success
pub fn process_short(ticks: i32, input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_short(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Document

/// process interleaved double samples from inBuffer -> libpd -> outBuffer
/// buffer sizes are based on # of ticks and channels where:
///     size = ticks * libpd_blocksize() * (in/out)channels
/// returns 0 on success
pub fn process_double(ticks: i32, input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_double(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Document

/// process non-interleaved float samples from inBuffer -> libpd -> outBuffer
/// copies buffer contents to/from libpd without striping
/// buffer sizes are based on a single tick and # of channels where:
///     size = libpd_blocksize() * (in/out)channels
/// returns 0 on success
pub fn process_raw(input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_raw(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Document

/// process non-interleaved short samples from inBuffer -> libpd -> outBuffer
/// copies buffer contents to/from libpd without striping
/// buffer sizes are based on a single tick and # of channels where:
///     size = libpd_blocksize() * (in/out)channels
/// float samples are converted to short by multiplying by 32767 and casting,
/// so any values received from pd patches beyond -1 to 1 will result in garbage
/// note: for efficiency, does *not* clip input
/// returns 0 on success
pub fn process_raw_short(input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_raw_short(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Document

/// process non-interleaved double samples from inBuffer -> libpd -> outBuffer
/// copies buffer contents to/from libpd without striping
/// buffer sizes are based on a single tick and # of channels where:
///     size = libpd_blocksize() * (in/out)channels
/// returns 0 on success
pub fn process_raw_double(input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_raw_double(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

// TODO: Implement array utilities!
// TODO: Document

// /* array access */
// /// get the size of an array by name
// /// returns size or negative error code if non-existent
// EXTERN int libpd_arraysize(const char *name);

// /// (re)size an array by name; sizes <= 0 are clipped to 1
// /// returns 0 on success or negative error code if non-existent
// EXTERN int libpd_resize_array(const char *name, long size);

// /// read n values from named src array and write into dest starting at an offset
// /// note: performs no bounds checking on dest
// /// returns 0 on success or a negative error code if the array is non-existent
// /// or offset + n exceeds range of array
// EXTERN int libpd_read_array(float *dest, const char *name, int offset, int n);

// /// read n values from src and write into named dest array starting at an offset
// /// note: performs no bounds checking on src
// /// returns 0 on success or a negative error code if the array is non-existent
// /// or offset + n exceeds range of array
// EXTERN int libpd_write_array(const char *name, int offset,
// 	const float *src, int n);

/// Sends a `bang` to the pd receiver object specified in `receiver` the argument
///
/// `send_bang_to("foo")` will send a bang to `|s foo|` on the next tick.
/// The `bang` can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// // Handle the error if the receiver object is not found
/// send_bang_to("foo").unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_bang_to("foo");
/// ```
pub fn send_bang_to<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_bang(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

// TODO: This function takes a float instead of a double.
// TODO: Does pd also support doubles?

/// Sends a float value to the pd receiver object specified in the `receiver` argument
///
/// `send_float_to("foo", 1.0)` will send the float value to `|s foo|` on the next tick.
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// // Handle the error if the receiver object is not found
/// send_float_to("foo", 1.0).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_float_to("foo", 1.0);
/// ```
pub fn send_float_to<T: AsRef<str>>(receiver: T, value: f32) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_float(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Sends a symbol to the pd receiver object specified in the `receiver` argument
///
/// `send_symbol_to("foo", "bar")` will send the symbol value to `|s foo|` on the next tick.
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// // Handle the error if the receiver object is not found
/// send_symbol_to("foo", "bar").unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_symbol_to("foo", "bar");
/// ```
pub fn send_symbol_to<T: AsRef<str>, S: AsRef<str>>(
    receiver: T,
    value: S,
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    let sym = CString::new(value.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_symbol(recv.as_ptr(), sym.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
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
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   // Add some values to the message..
/// }
/// ```
pub fn start_message(length: i32) -> Result<(), SizeError> {
    unsafe {
        match libpd_sys::libpd_start_message(length) {
            0 => Ok(()),
            _ => Err(SizeError::TooLarge),
        }
    }
}

// TODO: Find what happens if you add a float to a maxed out message

/// Add a float to the current message in the progress of composition
///
/// # Example
/// ```rust
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_float_to_started_message(42.0);
/// }
/// ```
pub fn add_float_to_started_message(value: f32) {
    unsafe {
        libpd_sys::libpd_add_float(value);
    }
}

// TODO: Find what happens if you add a symbol to a maxed out message

/// Add a symbol to the current message in the progress of composition
///
/// # Example
/// ```rust
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_symbol_to_started_message("foo");
/// }
/// ```
pub fn add_symbol_to_started_message<T: AsRef<str>>(value: T) {
    let sym = CString::new(value.as_ref()).unwrap();
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
pub fn finish_message_as_list_and_send_to<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_finish_list(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
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
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    let msg = CString::new(message_header.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_finish_message(recv.as_ptr(), msg.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

// TODO: Implementation might be wrong, re-visit doc

/// Finish the current message and send as a list to a receiver in the loaded pd patch
///
/// The following example will send a list `42.0 bar` to `|s foo|` on the next tick.
/// The list can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// use libpd_rs::types::Atom;
/// let list = vec![Atom::from(42.0), Atom::from("bar")]
/// // Handle the error if the receiver object is not found
/// send_list_to("foo", &list).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_list_to("foo", &list);
/// ```
pub fn send_list_to<T: AsRef<str>>(receiver: T, list: &[Atom]) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        // First element should give the start of the list hopefully. :)
        // TODO: Revisit and test this implementation.

        // TODO: Probably the implementation is wrong./
        // TODO: Really re-visit
        // ex: send [list 1 2 bar( to [r foo] on the next tick with:
        //     t_atom v[3];
        //     libpd_set_float(v, 1);
        //     libpd_set_float(v + 1, 2);
        //     libpd_set_symbol(v + 2, "bar");
        //     libpd_list("foo", 3, v);
        match libpd_sys::libpd_list(recv.as_ptr(), list.len() as i32, list[0].as_mut_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

// TODO: Correct this after re-visiting list implementation

/// send a atom array of a given length as a typed message to a destination
/// receiver, returns 0 on success or -1 if receiver name is non-existent
/// ex: send [; pd dsp 1( on the next tick with:
///     t_atom v[1];
///     libpd_set_float(v, 1);
///     libpd_message("pd", "dsp", 1, v);
pub fn send_message<T: AsRef<str>>(
    receiver: T,
    message: T,
    list: &[Atom],
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    let msg = CString::new(message.as_ref()).unwrap();
    unsafe {
        // First element should give the start of the list hopefully. :)
        // TODO: Revisit and test this implementation.
        match libpd_sys::libpd_message(
            recv.as_ptr(),
            msg.as_ptr(),
            list.len() as i32,
            list[0].as_mut_ptr(),
        ) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Subscribes to messages sent to a receiver in the loaded pd patch
///
/// `start_listening_from("foo")` would add a **virtual** `|r foo|` which would
/// forward messages to the libpd message listeners
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
/// let sources = vec!["foo", "bar"];
/// // Maybe you would like to use the receiver handles later..
/// let handles: Hashmap<String, ReceiverHandle> = HashMap::new();
/// for source in sources {
///     match start_listening_from(&source) {
///         // Start listening from a source and keep the handle for later
///         Ok(handle) => Hashmap::insert(source.to_owned(), handle),
///         // Handle the error if there is no source to listen from
///         Err(err) => println!("{}", err),
///     }
/// }
/// ```
pub fn start_listening_from<T: AsRef<str>>(sender: T) -> Result<ReceiverHandle, SubscriptionError> {
    let send = CString::new(sender.as_ref()).unwrap();

    unsafe {
        let handle = libpd_sys::libpd_bind(send.as_ptr());
        if handle.is_null() {
            Err(SubscriptionError::FailedToSubscribeToSender(
                sender.as_ref().to_owned(),
            ))
        } else {
            Ok(ReceiverHandle::from(handle))
        }
    }
}

/// Unsubscribes from messages sent to the receiver in the loaded pd patch
///
/// `stop_listening_from("foo")` would remove the virtual `|r foo|`.
///  Messages can not be received from this receiver anymore.
///
/// # Example
/// ```rust
/// let receiver_handle = start_listening_from("foo").unwrap();
/// stop_listening_from(receiver_handle);
/// ```
pub fn stop_listening_from(source: ReceiverHandle) {
    let handle = source.into_inner();
    if handle.is_null() {
        // TODO: Actually handle shouldn't be null in any case!
        // TODO: Check if this assumption is correct, if so remove this check
        return;
    }
    unsafe {
        libpd_sys::libpd_unbind(handle);
    }
}

/// Check if a source to listen from exists
///
/// # Example
/// ```rust
/// if source_to_listen_from_exists("foo") {
///   if let receiver_handle = start_listening_from("foo") {
///     // Do something with the handle..
///   }
/// }
/// ```
pub fn source_to_listen_from_exists<T: AsRef<str>>(sender: T) -> bool {
    let send = CString::new(sender.as_ref()).unwrap();
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
/// on_print(|msg: &str| {
///  println!("pd is printing: {msg}");
/// });
/// ```
pub fn on_print<F: Fn(&str) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |out: *const std::os::raw::c_char| {
        let out = unsafe { CStr::from_ptr(out).to_str().unwrap() };
        user_provided_closure(out);
    }));
    let callback = Closure1::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe {
        // Always concatenate
        libpd_sys::libpd_set_printhook(Some(libpd_sys::libpd_print_concatenator));
        libpd_sys::libpd_set_concatenated_printhook(Some(*ptr));
    };
}

/// Sets a closure to be called when a bang is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// // This is an example, handle the result properly here..
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// on_bang(|source: &str| {
///   match source {
///     "foo" => println!("bang from foo"),   
///     "bar" => println!("bang from bar"),
///      _ => unreachable!(),
///   }
/// });
/// ```
pub fn on_bang<F: Fn(&str) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |source: *const std::os::raw::c_char| {
        let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
        user_provided_closure(source);
    }));
    let callback = Closure1::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_banghook(Some(*ptr)) };
}

/// Sets a closure to be called when a float is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// // This is an example, handle the result properly here..
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// on_float(|source: &str, value: f32| {
///   match source {
///     "foo" => println!("Received a float from foo, value is: {value}"),   
///     "bar" => println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
/// ```
pub fn on_float<F: Fn(&str, f32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char, float: f32| {
            let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
            user_provided_closure(source, float);
        },
    ));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_floathook(Some(*ptr)) };
}

/// Sets a closure to be called when a symbol is received from a subscribed receiver
///
/// Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// // This is an example, handle the result properly here..
/// let foo_receiver_handle = start_listening_from("foo").unwrap();
/// let bar_receiver_handle = start_listening_from("bar").unwrap();
/// on_symbol(|source: &str, value: &str| {
///   match source {
///     "foo" => println!("Received a float from foo, value is: {value}"),   
///     "bar" => println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
/// ```
pub fn on_symbol<F: Fn(&str, &str) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char, symbol: *const std::os::raw::c_char| {
            let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
            let symbol = unsafe { CStr::from_ptr(symbol).to_str().unwrap() };
            user_provided_closure(source, symbol);
        },
    ));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_symbolhook(Some(*ptr)) };
}

// TODO: Re-document and check this after re-visiting atom list implementation.
pub fn on_list<F: Fn(&str, i32, &[Atom]) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char,
              list_length: i32,
              atom_list: *mut libpd_sys::t_atom| {
            let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
            unsafe {
                let atoms: Vec<libpd_sys::t_atom> =
                    Vec::from_raw_parts(atom_list, list_length as usize, list_length as usize);
                let wrapped_atoms: Vec<Atom> =
                    atoms.iter().map(|t_atom| Atom::from(*t_atom)).collect();
                user_provided_closure(source, list_length, &wrapped_atoms);
            };
        },
    ));
    let callback = Closure3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_listhook(Some(*ptr)) };
}

// TODO: Re-document and check this after re-visiting atom list implementation.
pub fn on_message<F: Fn(&str, &str, i32, &[Atom]) + Send + Sync + 'static>(
    user_provided_closure: F,
) {
    let closure: &'static _ = Box::leak(Box::new(
        move |source: *const std::os::raw::c_char,
              message: *const std::os::raw::c_char,
              list_length: i32,
              atom_list: *mut libpd_sys::t_atom| {
            let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
            let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
            unsafe {
                let atoms: Vec<libpd_sys::t_atom> =
                    Vec::from_raw_parts(atom_list, list_length as usize, list_length as usize);
                let wrapped_atoms: Vec<Atom> =
                    atoms.iter().map(|t_atom| Atom::from(*t_atom)).collect();
                user_provided_closure(source, message, list_length, &wrapped_atoms);
            };
        },
    ));
    let callback = Closure4::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_messagehook(Some(*ptr)) };
}

// TODO: Find out if there is a necessity to implement the queued ones, currently don't see any need.

// libpd_sys::libpd_set_queued_banghook;
// libpd_sys::libpd_set_queued_listhook;
// libpd_sys::libpd_set_queued_messagehook;
// libpd_sys::libpd_set_queued_printhook;
// libpd_sys::libpd_set_queued_symbolhook;
// libpd_sys::libpd_queued_receive_pd_messages;

// @attention Stayed here..

/// send a MIDI note on message to [notein] objects
/// channel is 0-indexed, pitch is 0-127, and velocity is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: there is no note off message, send a note on with velocity = 0 instead
/// returns 0 on success or -1 if an argument is out of range
pub fn send_note_on(channel: i32, pitch: i32, velocity: i32) {
    unsafe { libpd_sys::libpd_noteon(channel, pitch, velocity) };
}

/// send a MIDI control change message to [ctlin] objects
/// channel is 0-indexed, controller is 0-127, and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// returns 0 on success or -1 if an argument is out of range
pub fn send_control_change(channel: i32, controller: i32, value: i32) {
    unsafe { libpd_sys::libpd_controlchange(channel, controller, value) };
}

/// send a MIDI program change message to [pgmin] objects
/// channel is 0-indexed and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// returns 0 on success or -1 if an argument is out of range
pub fn send_program_change(channel: i32, value: i32) {
    unsafe { libpd_sys::libpd_programchange(channel, value) };
}

/// send a MIDI pitch bend message to [bendin] objects
/// channel is 0-indexed and value is -8192-8192
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: [bendin] outputs 0-16383 while [bendout] accepts -8192-8192
/// returns 0 on success or -1 if an argument is out of range
pub fn send_pitch_bend(channel: i32, value: i32) {
    unsafe { libpd_sys::libpd_pitchbend(channel, value) };
}

/// send a MIDI after touch message to [touchin] objects
/// channel is 0-indexed and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// returns 0 on success or -1 if an argument is out of range
pub fn send_aftertouch(channel: i32, value: i32) {
    unsafe { libpd_sys::libpd_aftertouch(channel, value) };
}

/// send a MIDI poly after touch message to [polytouchin] objects
/// channel is 0-indexed, pitch is 0-127, and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// returns 0 on success or -1 if an argument is out of range
pub fn send_poly_aftertouch(channel: i32, pitch: i32, value: i32) {
    unsafe { libpd_sys::libpd_polyaftertouch(channel, pitch, value) };
}

/// send a raw MIDI byte to [midiin] objects
/// port is 0-indexed and byte is 0-256
/// returns 0 on success or -1 if an argument is out of range
pub fn send_midi_byte(port: i32, byte: i32) {
    unsafe { libpd_sys::libpd_midibyte(port, byte) };
}

/// send a raw MIDI byte to [sysexin] objects
/// port is 0-indexed and byte is 0-256
/// returns 0 on success or -1 if an argument is out of range
pub fn send_sysex(port: i32, byte: i32) {
    unsafe { libpd_sys::libpd_sysex(port, byte) };
}

/// send a raw MIDI byte to [realtimein] objects
/// port is 0-indexed and byte is 0-256
/// returns 0 on success or -1 if an argument is out of range
pub fn send_sys_realtime(port: i32, byte: i32) {
    unsafe { libpd_sys::libpd_sysrealtime(port, byte) };
}

// TODO: MIDI hooks / MIDI queued hooks

/// MIDI note on receive hook signature
/// channel is 0-indexed, pitch is 0-127, and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: there is no note off message, note on w/ velocity = 0 is used instead
/// note: out of range values from pd are clamped
pub fn on_midi_note_on<F: Fn(i32, i32, i32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ =
        Box::leak(Box::new(move |channel: i32, pitch: i32, velocity: i32| {
            user_provided_closure(channel, pitch, velocity);
        }));
    let callback = Closure3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_noteonhook(Some(*ptr)) };
}

/// MIDI control change receive hook signature
/// channel is 0-indexed, controller is 0-127, and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: out of range values from pd are clamped
pub fn on_midi_control_change<F: Fn(i32, i32, i32) + Send + Sync + 'static>(
    user_provided_closure: F,
) {
    let closure: &'static _ = Box::leak(Box::new(
        move |channel: i32, controller: i32, value: i32| {
            user_provided_closure(channel, controller, value);
        },
    ));
    let callback = Closure3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_controlchangehook(Some(*ptr)) };
}

/// MIDI program change receive hook signature
/// channel is 0-indexed and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: out of range values from pd are clamped
pub fn on_midi_program_change<F: Fn(i32, i32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_programchangehook(Some(*ptr)) };
}

/// MIDI pitch bend receive hook signature
/// channel is 0-indexed and value is -8192-8192
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: [bendin] outputs 0-16383 while [bendout] accepts -8192-8192
/// note: out of range values from pd are clamped
pub fn on_midi_pitch_bend<F: Fn(i32, i32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_pitchbendhook(Some(*ptr)) };
}

/// MIDI after touch receive hook signature
/// channel is 0-indexed and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: out of range values from pd are clamped
pub fn on_midi_after_touch<F: Fn(i32, i32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |channel: i32, value: i32| {
        user_provided_closure(channel, value);
    }));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_aftertouchhook(Some(*ptr)) };
}

/// MIDI poly after touch receive hook signature
/// channel is 0-indexed, pitch is 0-127, and value is 0-127
/// channels encode MIDI ports via: libpd_channel = pd_channel + 16 * pd_port
/// note: out of range values from pd are clamped
pub fn on_midi_poly_after_touch<F: Fn(i32, i32, i32) + Send + Sync + 'static>(
    user_provided_closure: F,
) {
    let closure: &'static _ = Box::leak(Box::new(move |channel: i32, pitch: i32, value: i32| {
        user_provided_closure(channel, pitch, value);
    }));
    let callback = Closure3::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_polyaftertouchhook(Some(*ptr)) };
}

/// raw MIDI byte receive hook signature
/// port is 0-indexed and byte is 0-256
/// note: out of range values from pd are clamped
pub fn on_midi_byte<F: Fn(i32, i32) + Send + Sync + 'static>(user_provided_closure: F) {
    let closure: &'static _ = Box::leak(Box::new(move |port: i32, byte: i32| {
        user_provided_closure(port, byte);
    }));
    let callback = Closure2::new(closure);
    let code = callback.code_ptr();
    let ptr: &_ = unsafe { std::mem::transmute(code) };
    std::mem::forget(callback);
    unsafe { libpd_sys::libpd_set_midibytehook(Some(*ptr)) };
}

// TODO: Find out if there is a necessity to implement the queued ones, currently don't see any need.

// libpd_sys::libpd_set_queued_aftertouchhook;
// libpd_sys::libpd_set_queued_controlchangehook;
// libpd_sys::libpd_set_queued_midibytehook;
// libpd_sys::libpd_set_queued_noteonhook;
// libpd_sys::libpd_set_queued_pitchbendhook;
// libpd_sys::libpd_set_queued_polyaftertouchhook;
// libpd_sys::libpd_set_queued_programchangehook;
// libpd_sys::libpd_queued_receive_midi_messages;

// TODO: Gui utils

/// open the current patches within a pd vanilla GUI
/// requires the path to pd's main folder that contains bin/, tcl/, etc
/// for a macOS .app bundle: /path/to/Pd-#.#-#.app/Contents/Resources
/// returns 0 on success
pub fn start_gui(path_to_pd: &std::path::Path) {
    // TODO: Check if path is valid
    // If it exists
    // Return result
    // Open the path
    // Return unit type on success
    // const char *path the sys function expects
    todo!()
}

/// stop the pd vanilla GUI
pub fn stop_gui() {
    unsafe { libpd_sys::libpd_stop_gui() };
}

/// manually update and handle any GUI messages
/// this is called automatically when using a libpd_process function,
/// note: this also facilitates network message processing, etc so it can be
///       useful to call repeatedly when idle for more throughput
/// returns 1 if the poll found something, in which case it might be desirable
/// to poll again, up to some reasonable limit
pub fn poll_gui() {
    // TODO: Implement Option return
    unsafe { libpd_sys::libpd_poll_gui() };
}

// @attention Multi instance features implementation is scheduled for later.
// @attention If there is a necessity emerges, I'll give time to implement.

/* multiple instance functions in z_libpd.h */

/// create a new pd instance
/// returns new instance or NULL when libpd is not compiled with PDINSTANCE
// EXTERN t_pdinstance *libpd_new_instance(void);

/// set the current pd instance
/// subsequent libpd calls will affect this instance only
/// does nothing when libpd is not compiled with PDINSTANCE
// EXTERN void libpd_set_instance(t_pdinstance *p);

/// free a pd instance
/// does nothing when libpd is not compiled with PDINSTANCE
// EXTERN void libpd_free_instance(t_pdinstance *p);

/// get the current pd instance
// EXTERN t_pdinstance *libpd_this_instance(void);

/// get a pd instance by index
/// returns NULL if index is out of bounds or "this" instance when libpd is not
/// compiled with PDINSTANCE
// EXTERN t_pdinstance *libpd_get_instance(int index);

/// get the number of pd instances
/// returns number or 1 when libpd is not compiled with PDINSTANCE
// EXTERN int libpd_num_instances(void);

/* bindings for multiple instance functions */
// extern "C" {
//     #[doc = " create a new pd instance"]
//     #[doc = " returns new instance or NULL when libpd is not compiled with PDINSTANCE"]
//     pub fn libpd_new_instance() -> *mut _pdinstance;
// }
// extern "C" {
//     #[doc = " set the current pd instance"]
//     #[doc = " subsequent libpd calls will affect this instance only"]
//     #[doc = " does nothing when libpd is not compiled with PDINSTANCE"]
//     pub fn libpd_set_instance(p: *mut _pdinstance);
// }
// extern "C" {
//     #[doc = " free a pd instance"]
//     #[doc = " does nothing when libpd is not compiled with PDINSTANCE"]
//     pub fn libpd_free_instance(p: *mut _pdinstance);
// }
// extern "C" {
//     #[doc = " get the current pd instance"]
//     pub fn libpd_this_instance() -> *mut _pdinstance;
// }
// extern "C" {
//     #[doc = " get a pd instance by index"]
//     #[doc = " returns NULL if index is out of bounds or \"this\" instance when libpd is not"]
//     #[doc = " compiled with PDINSTANCE"]
//     pub fn libpd_get_instance(index: ::std::os::raw::c_int) -> *mut _pdinstance;
// }
// extern "C" {
//     #[doc = " get the number of pd instances"]
//     #[doc = " returns number or 1 when libpd is not compiled with PDINSTANCE"]
//     pub fn libpd_num_instances() -> ::std::os::raw::c_int;
// }

/// Sets the status of verbose printing to the pd console
pub fn verbose_print_state(active: bool) {
    match active {
        true => unsafe { libpd_sys::libpd_set_verbose(1) },
        false => unsafe { libpd_sys::libpd_set_verbose(0) },
    }
}

/// Checks if verbose printing to the pd console is active
pub fn verbose_print_state_active() -> bool {
    unsafe { libpd_sys::libpd_get_verbose() == 1 }
}

#[cfg(test)]
mod tests {
    use crate::pd::Pd;

    use super::*;

    #[test]
    #[ignore]
    fn run_simple() {
        let mut pd = Pd::default();
        let project_root = std::env::current_dir().unwrap();
        assert!(pd.open(&project_root.join("simple.pd")).is_ok());
        assert!(initialize_audio(0, 2, 44100).is_ok());
        assert!(pd.dsp_on().is_ok());
        assert!(pd.is_dsp_on());
        assert!(pd.dsp_off().is_ok());
        assert!(!pd.is_dsp_on());
        pd.close();
    }

    #[test]
    #[ignore]
    fn run_safe() {
        assert!(init().is_ok());
        assert!(init().is_err());
        start_message(1);
        add_float_to_started_message(1.0);
        let result = finish_message_as_typed_message_and_send_to("pd", "dsp");
        assert!(result.is_ok());
        let project_root = std::env::current_dir().unwrap();
        let result = open_patch(&project_root.join("simple.pd"));
        assert!(result.is_ok());
        let result_2 = open_patch(&project_root.join("bad_naming.pd"));
        assert!(result_2.is_err());
        let handle = result.unwrap();

        let input_buffer = [0.0f32; 64];
        let mut output_buffer = [0.0f32; 128];

        // now run pd for ten seconds (logical time)
        for _ in 0..((10 * 44100) / 64) {
            // fill input_buffer here
            process_float(1, &input_buffer[..], &mut output_buffer[..]);
            // use output_buffer here
        }

        for sample in output_buffer.iter().take(10) {
            println!("{}", sample);
        }

        close_patch(handle);
    }

    #[test]
    fn run_unsafe() {
        unsafe {
            //    ::std::option::Option<unsafe extern "C" fn(s: *const ::std::os::raw::c_char)>;

            // // // unsafe extern "C" fn print_hook(s: *const ::std::os::raw::c_char) {}
            // unsafe extern "C" fn float_hook(s: *const ::std::os::raw::c_char, x: f32) {
            //     println!("HELLOOOO: {} {}", CStr::from_ptr(s).to_str().unwrap(), x);
            // }

            // Setting hooks are better before init!

            // libpd_sys::libpd_set_floathook(Some(float_hook));
            // libpd_sys::libpd_set_printhook(libpd_sys::libpd_print_concatenator as *const ());
            // libpd_sys::libpd_set_concatenated_printhook(print_hook as *const ());

            // Also decide for queued hooks or normal hooks.
            // If queued hooks we may use libpd_queued_init!

            on_print(|string| {
                println!("IAM DONE {}", string);
            });
            on_float(|source, value| {
                println!("HELLOOOO: {} {}", source, value);
            });
            let status = libpd_sys::libpd_init();
            assert_eq!(status, 0);
            let status = libpd_sys::libpd_init();
            assert_eq!(status, -1);
            let status = libpd_sys::libpd_init_audio(1, 2, 44100);
            assert_eq!(status, 0);

            let r = CString::new("simple_float").unwrap();
            let rp = libpd_sys::libpd_bind(r.as_ptr());

            // libpd_sys::libpd_set_queued_printhook(Some(libpd_sys::libpd_print_concatenator));
            // libpd_sys::libpd_set_concatenated_printhook(Some(abc));
            // libpd_sys::libpd_set_queued_floathook(Some(dd));
            // libpd_sys::libpd_queued_receive_pd_messages();
            // libpd_sys::libpd_queued_init();

            // libpd_sys::sys_printhook = Some(abc);

            libpd_sys::libpd_start_message(1);
            libpd_sys::libpd_add_float(1.0);
            let msg = CString::new("pd").unwrap();
            let recv = CString::new("dsp").unwrap();
            libpd_sys::libpd_finish_message(msg.as_ptr(), recv.as_ptr());

            let project_root = std::env::current_dir().unwrap();
            let name = CString::new("simple.pd").unwrap();
            let directory = CString::new(project_root.to_str().unwrap()).unwrap();
            let file_handle = libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr());

            let input_buffer = [0.0f32; 64];
            let mut output_buffer = [0.0f32; 128];

            // now run pd for ten seconds (logical time)
            for _ in 0..((10 * 44100) / 64) {
                // fill input_buffer here
                libpd_sys::libpd_process_float(
                    1,
                    input_buffer[..].as_ptr(),
                    output_buffer[..].as_mut_ptr(),
                );
                // use output_buffer here
            }

            for sample in output_buffer.iter().take(10) {
                println!("{}", sample);
            }

            // libpd_sys::libpd_closefile(file_handle);
        }
    }
}
