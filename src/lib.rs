pub mod error;
pub mod pd;
pub mod types;
use error::SubscriptionError;
use types::ReceiverHandle;

use crate::error::{InitializationError, IoError, SendError};
use crate::types::{Atom, PatchFileHandle};
use libffi::high::{Closure1, Closure2, Closure3, Closure4};
use std::ffi::{CStr, CString};

/// initialize libpd; it is safe to call this more than once
/// returns 0 on success or -1 if libpd was already initialized
/// note: sets SIGFPE handler to keep bad pd patches from crashing due to divide
///       by 0, set any custom handling after calling this function
pub fn init() -> Result<(), InitializationError> {
    unsafe {
        // What if this function throws a runtime error?
        // TODO: Find out idiomatic Rust way to handle this.
        match libpd_sys::libpd_init() {
            0 => Ok(()),
            -1 => Err(InitializationError::AlreadyInitialized),
            _ => Err(InitializationError::InitializationFailed),
        }
    }
}

// TODO: Queued init and release should i implement?
// pub use libpd_sys::libpd_queued_init;
// pub use libpd_sys::libpd_queued_release;

/// clear the libpd search path for abstractions and externals
/// note: this is called by libpd_init()
pub fn clear_search_paths() {
    unsafe {
        libpd_sys::libpd_clear_search_path();
    }
}

/// add a path to the libpd search paths
/// relative paths are relative to the current working directory
/// unlike desktop pd, *no* search paths are set by default (ie. extra)
pub fn add_to_search_paths(path: &std::path::Path) {
    unsafe {
        let c_path = CString::new(&*path.to_string_lossy()).unwrap();
        libpd_sys::libpd_add_to_search_path(c_path.as_ptr());
    }
}

/// open a patch by filename and parent dir path
/// returns an opaque patch handle pointer or NULL on failure
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

/// close a patch by patch handle pointer
pub fn close_patch(handle: PatchFileHandle) {
    unsafe {
        // TODO: Can this file handle ever be null or invalidated?
        // This is a free, is there any way for a double free to occur?
        libpd_sys::libpd_closefile(handle.into_inner());
    }
}

/// get the $0 id of the patch handle pointer
/// returns $0 value or 0 if the patch is non-existent
pub fn get_dollar_zero(handle: PatchFileHandle) {
    unsafe {
        libpd_sys::libpd_getdollarzero(handle.into_inner());
    }
}

/// return pd's fixed block size: the number of sample frames per 1 pd tick
pub fn block_size() -> i32 {
    unsafe { libpd_sys::libpd_blocksize() }
}

/// initialize audio rendering
/// returns 0 on success
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

/// process interleaved float samples from inBuffer -> libpd -> outBuffer
/// buffer sizes are based on # of ticks and channels where:
///     size = ticks * libpd_blocksize() * (in/out)channels
/// returns 0 on success
pub fn process_float(ticks: i32, input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_float(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

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

/// process interleaved double samples from inBuffer -> libpd -> outBuffer
/// buffer sizes are based on # of ticks and channels where:
///     size = ticks * libpd_blocksize() * (in/out)channels
/// returns 0 on success
pub fn process_double(ticks: i32, input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_double(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

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

/// send a bang to a destination receiver
/// ex: libpd_bang("foo") will send a bang to [s foo] on the next tick
/// returns 0 on success or -1 if receiver name is non-existent
pub fn send_bang<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_bang(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// send a float to a destination receiver
/// ex: libpd_float("foo", 1) will send a 1.0 to [s foo] on the next tick
/// returns 0 on success or -1 if receiver name is non-existent
pub fn send_float<T: AsRef<str>>(receiver: T, value: f32) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_float(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// send a symbol to a destination receiver
/// ex: libpd_symbol("foo", "bar") will send "bar" to [s foo] on the next tick
/// returns 0 on success or -1 if receiver name is non-existent
pub fn send_symbol<T: AsRef<str>>(receiver: T, symbol: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    let sym = CString::new(symbol.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_symbol(recv.as_ptr(), sym.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// start composition of a new list or typed message of up to max element length
/// messages can be of a smaller length as max length is only an upper bound
/// note: no cleanup is required for unfinished messages
/// returns 0 on success or nonzero if the length is too large
pub fn start_message(length: i32) {
    unsafe {
        libpd_sys::libpd_start_message(length);
    }
}

/// add a float to the current message in progress
pub fn add_float_to_started_message(value: f32) {
    unsafe {
        libpd_sys::libpd_add_float(value);
    }
}

/// add a symbol to the current message in progress
pub fn add_symbol_to_started_message<T: AsRef<str>>(value: T) {
    let sym = CString::new(value.as_ref()).unwrap();
    unsafe {
        libpd_sys::libpd_add_symbol(sym.as_ptr());
    }
}

/// finish current message and send as a list to a destination receiver
/// returns 0 on success or -1 if receiver name is non-existent
/// ex: send [list 1 2 bar( to [s foo] on the next tick with:
///     libpd_start_message(3);
///     libpd_add_float(1);
///     libpd_add_float(2);
///     libpd_add_symbol("bar");
///     libpd_finish_list("foo");
pub fn finish_message_as_list_and_send<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_finish_list(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// finish current message and send as a typed message to a destination receiver
/// note: typed message handling currently only supports up to 4 elements
///       internally, additional elements may be ignored
/// returns 0 on success or -1 if receiver name is non-existent
/// ex: send [; pd dsp 1( on the next tick with:
///     libpd_start_message(1);
///     libpd_add_float(1);
///     libpd_finish_message("pd", "dsp");
pub fn finish_message_as_message_and_send<T: AsRef<str>>(
    receiver: T,
    message: T,
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    let msg = CString::new(message.as_ref()).unwrap();
    unsafe {
        match libpd_sys::libpd_finish_message(recv.as_ptr(), msg.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// send an atom array of a given length as a list to a destination receiver
/// returns 0 on success or -1 if receiver name is non-existent
/// ex: send [list 1 2 bar( to [r foo] on the next tick with:
///     t_atom v[3];
///     libpd_set_float(v, 1);
///     libpd_set_float(v + 1, 2);
///     libpd_set_symbol(v + 2, "bar");
///     libpd_list("foo", 3, v);
pub fn send_list<T: AsRef<str>>(receiver: T, list: &[Atom]) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).unwrap();
    unsafe {
        // First element should give the start of the list hopefully. :)
        // TODO: Revisit and test this implementation.
        match libpd_sys::libpd_list(recv.as_ptr(), list.len() as i32, list[0].as_mut_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

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

/// subscribe to messages sent to a source receiver
/// ex: libpd_bind("foo") adds a "virtual" [r foo] which forwards messages to
///     the libpd message hooks
/// returns an opaque receiver pointer or NULL on failure
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

/// unsubscribe and free a source receiver object created by libpd_bind()
pub fn stop_listening_from(source: ReceiverHandle) {
    let handle = source.into_inner();
    if handle.is_null() {
        return;
    }
    unsafe {
        libpd_sys::libpd_unbind(handle);
    }
}

/// check if a source receiver object exists with a given name
/// returns 1 if the receiver exists, otherwise 0
pub fn source_to_listen_from_exists<T: AsRef<str>>(sender: T) -> bool {
    let send = CString::new(sender.as_ref()).unwrap();
    unsafe { matches!(libpd_sys::libpd_exists(send.as_ptr()), 1) }
}

// Hooks / queued hooks / print hook utils

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
        // Concatenate by default
        libpd_sys::libpd_set_printhook(Some(libpd_sys::libpd_print_concatenator));
        libpd_sys::libpd_set_concatenated_printhook(Some(*ptr));
    };
}

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

// TODO: Find out if there is a necessity to implement the queued ones?

// pub use libpd_sys::libpd_set_queued_banghook;
// pub use libpd_sys::libpd_set_queued_listhook;
// pub use libpd_sys::libpd_set_queued_messagehook;
// pub use libpd_sys::libpd_set_queued_printhook;
// pub use libpd_sys::libpd_set_queued_symbolhook;

// pub use libpd_sys::libpd_queued_receive_pd_messages;

// TODO: MIDI senders

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

// TODO: Find out if there is a necessity to implement the queued ones?

// pub use libpd_sys::libpd_set_queued_aftertouchhook;
// pub use libpd_sys::libpd_set_queued_controlchangehook;
// pub use libpd_sys::libpd_set_queued_midibytehook;
// pub use libpd_sys::libpd_set_queued_noteonhook;
// pub use libpd_sys::libpd_set_queued_pitchbendhook;
// pub use libpd_sys::libpd_set_queued_polyaftertouchhook;
// pub use libpd_sys::libpd_set_queued_programchangehook;

// pub use libpd_sys::libpd_queued_receive_midi_messages;

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

// TODO: Multi instance functions

// Will implement later..

/// set verbose print state: 0 or 1
pub fn verbose_print_state(active: bool) {
    match active {
        true => unsafe { libpd_sys::libpd_set_verbose(1) },
        false => unsafe { libpd_sys::libpd_set_verbose(0) },
    }
}

/// get the verbose print state: 0 or 1
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
        let result = finish_message_as_message_and_send("pd", "dsp");
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
