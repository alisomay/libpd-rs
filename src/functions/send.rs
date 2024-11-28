use crate::{
    atom::{make_t_atom_list_from_atom_list, Atom},
    error::{PdError, SendError, SizeError, StringConversionError},
};

use std::ffi::CString;

/// Sends a `bang` to the pd receiver object specified in `receiver` the argument
///
/// `send_bang_to("foo")` will send a bang to `|s foo|` on the next tick.
///
/// The `bang` can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::functions::send::send_bang_to;
///
/// // Handle the error if the receiver object is not found
/// send_bang_to("foo").unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_bang_to("foo");
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn send_bang_to<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        match libpd_sys::libpd_bang(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Sends an `f32` value to the pd receiver object specified in the `receiver` argument
///
/// `send_float_to("foo", 1.0)` will send the `f32` value to `|s foo|` on the next tick.
///
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::functions::send::send_float_to;
///
/// // Handle the error if the receiver object is not found
/// send_float_to("foo", 1.0).unwrap_or_else(|err| {
///   dbg!("{}", err);
/// });
/// // or don't care..
/// let _ = send_float_to("foo", 1.0);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn send_float_to<T: AsRef<str>>(receiver: T, value: f32) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        match libpd_sys::libpd_float(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Sends an `f64` value to the pd receiver object specified in the `receiver` argument
///
/// `send_double_to("foo", 1.0)` will send the `f64` value to `|s foo|` on the next tick.
///
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::functions::send::send_double_to;
///
/// // Handle the error if the receiver object is not found
/// send_double_to("foo", 1.0).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_double_to("foo", 1.0);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn send_double_to<T: AsRef<str>>(receiver: T, value: f64) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        match libpd_sys::libpd_double(recv.as_ptr(), value) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Sends a symbol to the pd receiver object specified in the `receiver` argument
///
/// `send_symbol_to("foo", "bar")` will send the symbol value to `|s foo|` on the next tick.
///
/// The value can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```no_run
/// use libpd_rs::functions::send::send_symbol_to;
///
/// // Handle the error if the receiver object is not found
/// send_symbol_to("foo", "bar").unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_symbol_to("foo", "bar");
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn send_symbol_to<T: AsRef<str>, S: AsRef<str>>(
    receiver: T,
    value: S,
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    let sym = CString::new(value.as_ref()).map_err(StringConversionError::from)?;
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
/// use libpd_rs::functions::send::{start_message};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   // Add some values to the message..
/// }
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`TooLarge`](crate::error::SizeError::TooLarge)
pub fn start_message(length: i32) -> Result<(), SizeError> {
    unsafe {
        match libpd_sys::libpd_start_message(length) {
            0 => Ok(()),
            _ => Err(SizeError::TooLarge),
        }
    }
}

/// Adds an `f32` to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{start_message, add_float_to_started_message};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
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
/// Although I didn't check that, please create an [issue](https://github.com/alisomay/libpd-rs/issues).
pub fn add_float_to_started_message(value: f32) {
    unsafe {
        libpd_sys::libpd_add_float(value);
    }
}

/// Adds an `f64` to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{start_message, add_double_to_started_message};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
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
/// Although I didn't check that, please create an [issue](https://github.com/alisomay/libpd-rs/issues).
pub fn add_double_to_started_message(value: f64) {
    unsafe {
        libpd_sys::libpd_add_double(value);
    }
}

/// Adds a symbol to the current message in the progress of composition
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{start_message, add_symbol_to_started_message};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Arbitrary length
/// let message_length = 4;
/// if start_message(message_length).is_ok() {
///   add_symbol_to_started_message("foo");
/// }
/// ```
/// # Errors
///
/// A list of errors that can occur:
/// - [`StringConversion`](crate::error::SendError::StringConversion)
///
/// # Panics
/// To be honest I'd expect this to panic if you overflow a message buffer.
///
/// Although I didn't check that, please create an [issue](https://github.com/alisomay/libpd-rs/issues).
pub fn add_symbol_to_started_message<T: AsRef<str>>(value: T) -> Result<(), SendError> {
    let sym = CString::new(value.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        libpd_sys::libpd_add_symbol(sym.as_ptr());
    }
    Ok(())
}

/// Finishes the current message and send as a list to a receiver in the loaded pd patch
///
/// The following example will send a list `42.0 bar` to `|s foo|` on the next tick.
///
/// The list can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{start_message, add_symbol_to_started_message, add_float_to_started_message, finish_message_as_list_and_send_to};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
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
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn finish_message_as_list_and_send_to<T: AsRef<str>>(receiver: T) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        match libpd_sys::libpd_finish_list(recv.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Finishes the current message and send as a typed message to a receiver in the loaded pd patch
///
/// Typed message handling currently only supports up to `4` elements
/// internally in pd, **additional elements may be ignored.**
///
/// The following example will send a message `; pd dsp 1` on the next tick.
///
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{start_message, add_float_to_started_message, finish_message_as_typed_message_and_send_to};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
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
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`StringConversion`](crate::error::SendError::StringConversion)
pub fn finish_message_as_typed_message_and_send_to<T: AsRef<str>, S: AsRef<str>>(
    receiver: T,
    message_header: S,
) -> Result<(), SendError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    let msg = CString::new(message_header.as_ref()).map_err(StringConversionError::from)?;
    unsafe {
        match libpd_sys::libpd_finish_message(recv.as_ptr(), msg.as_ptr()) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned())),
        }
    }
}

/// Sends a list to a receiver in the loaded pd patch
///
/// The following example will send a list `42.0 bar` to `|s foo|` on the next tick.
///
/// The list can be received from a `|r foo|` object in the loaded pd patch.
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_list_to};
/// use libpd_rs::Atom;
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// let list = vec![Atom::from(42.0), Atom::from("bar")];
/// // Handle the error if the receiver object is not found
/// send_list_to("foo", &list).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_list_to("foo", &list);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`SendError`](crate::error::SendError)
///    - [`MissingDestination`](crate::error::SendError::MissingDestination)
///    - [`StringConversion`](crate::error::SendError::StringConversion)
/// - [`InstanceError`](crate::error::InstanceError)
///    - [`NoCurrentInstanceSet`](crate::error::InstanceError::NoCurrentInstanceSet)
/// - [`PdError`](crate::error::PdError)
///    - [`StringConversion`](crate::error::PdError::StringConversion)
pub fn send_list_to<T: AsRef<str>>(receiver: T, list: &[Atom]) -> Result<(), PdError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;

    let mut atom_list: Vec<libpd_sys::t_atom> = make_t_atom_list_from_atom_list(list)?;
    let atom_list_slice = atom_list.as_mut_slice();

    unsafe {
        #[expect(
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            reason = "This is what the function wants (i32). The value is never going to be negative or huge."
        )]
        match libpd_sys::libpd_list(
            recv.as_ptr(),
            list.len() as i32,
            atom_list_slice.as_mut_ptr(),
        ) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned()).into()),
        }
    }
}

/// Sends a typed message to a receiver in the loaded pd patch
///
/// The following example will send a typed message `dsp 1` to the receiver `pd` on the next tick.
///
/// The equivalent of this example message would have looked like `[; pd dsp 1]` in pd gui.
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_message_to};
/// use libpd_rs::Atom;
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// let values = vec![Atom::from(1.0)];
/// // Handle the error if the receiver object is not found
/// send_message_to("pd", "dsp", &values).unwrap_or_else(|err| {
///   println!("{}", err);
/// });
/// // or don't care..
/// let _ = send_message_to("pd", "dsp", &values);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`SendError`](crate::error::SendError)
///    - [`MissingDestination`](crate::error::SendError::MissingDestination)
///    - [`StringConversion`](crate::error::SendError::StringConversion)
/// - [`InstanceError`](crate::error::InstanceError)
///    - [`NoCurrentInstanceSet`](crate::error::InstanceError::NoCurrentInstanceSet)
/// - [`PdError`](crate::error::PdError)
///    - [`StringConversion`](crate::error::PdError::StringConversion)
pub fn send_message_to<T: AsRef<str>>(
    receiver: T,
    message: T,
    list: &[Atom],
) -> Result<(), PdError> {
    let recv = CString::new(receiver.as_ref()).map_err(StringConversionError::from)?;
    let msg = CString::new(message.as_ref()).map_err(StringConversionError::from)?;

    let mut atom_list: Vec<libpd_sys::t_atom> = make_t_atom_list_from_atom_list(list)?;
    let atom_list_slice = atom_list.as_mut_slice();

    unsafe {
        #[expect(
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            reason = "This is what the function wants (i32). The value is never going to be negative or huge."
        )]
        match libpd_sys::libpd_message(
            recv.as_ptr(),
            msg.as_ptr(),
            // This is fine since a list will not be millions of elements long and not negative for sure.
            list.len() as i32,
            atom_list_slice.as_mut_ptr(),
        ) {
            0 => Ok(()),
            _ => Err(SendError::MissingDestination(receiver.as_ref().to_owned()).into()),
        }
    }
}

/// Sends a MIDI note on message to `|notein|` objects in pd.
///
/// Channel is zero-indexed, pitch is `0-127` and velocity is `0-127`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: There is no note off message, send a note on with velocity = 0 instead.
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_note_on};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_note_on(0, 48, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_note_on(0, 48, 64);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_note_on(channel: i32, pitch: i32, velocity: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_noteon(channel, pitch, velocity) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a MIDI control change message to `ctlin` objects in pd.
///
/// Channel is zero-indexed, controller is `0-127` and value is `0-127`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_control_change};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_control_change(0, 0, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_control_change(0, 0, 64);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_control_change(channel: i32, controller: i32, value: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_controlchange(channel, controller, value) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a MIDI program change message to `pgmin` objects in pd.
///
/// Channel is zero-indexed, value is `0-127`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_program_change};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_program_change(0, 42).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_program_change(0, 42);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_program_change(channel: i32, value: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_programchange(channel, value) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a MIDI pitch bend message to `|bendin|` objects in pd.
///
/// Channel is zero-indexed, value is `-8192 to 8192`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// Note: `|bendin|` outputs 0-16383 while `|bendout|` accepts `-8192 to 8192`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_pitch_bend};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_pitch_bend(0, 8192).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_pitch_bend(0, 8192);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_pitch_bend(channel: i32, value: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_pitchbend(channel, value) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a MIDI after touch message to `|touchin|` objects in pd.
///
/// Channel is zero-indexed, value is `0-127`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_after_touch};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_after_touch(0, 42).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_after_touch(0, 42);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_after_touch(channel: i32, value: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_aftertouch(channel, value) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a MIDI poly after touch message to `|polytouchin|` objects in pd.
///
/// Channel is zero-indexed, pitch is `0-127` and value is `0-127`.
///
/// Channels encode MIDI ports via: `libpd_channel = pd_channel + 16 * pd_port`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_poly_after_touch};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_poly_after_touch(0, 48, 64).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_poly_after_touch(0, 48, 64);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_poly_after_touch(channel: i32, pitch: i32, value: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_polyaftertouch(channel, pitch, value) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a raw MIDI byte to `|midiin|` objects in pd.
///
/// Port is zero-indexed and byte is `0-255`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_midi_byte};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_midi_byte(0, 0xFF).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_midi_byte(0, 0xFF);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_midi_byte(port: i32, byte: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_midibyte(port, byte) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a raw MIDI byte to `|sysexin|` objects in pd.
///
/// Port is zero-indexed and byte is `0-255`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_sysex};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_sysex(0, 0x7F).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_sysex(0, 0x7F);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_sysex(port: i32, byte: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_sysex(port, byte) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}

/// Sends a raw MIDI byte to `|midirealtimein|` objects in pd.
///
/// Port is zero-indexed and byte is `0-255`
///
/// # Example
/// ```rust
/// use libpd_rs::functions::send::{send_sys_realtime};
/// use libpd_rs::instance::PdInstance;
///
/// let _main_instance = PdInstance::new().unwrap();
///
/// // Handle the error if the receiver object is not found
/// send_sys_realtime(0, 0x7F).unwrap_or_else(|err| {
///   dbg!("{err}");
/// });
/// // or don't care..
/// let _ = send_sys_realtime(0, 0x7F);
/// ```
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`OutOfRange`](crate::error::SendError::OutOfRange)
pub fn send_sys_realtime(port: i32, byte: i32) -> Result<(), SendError> {
    unsafe {
        // Returns 0 on success or -1 if an argument is out of range
        match libpd_sys::libpd_sysrealtime(port, byte) {
            0 => Ok(()),
            _ => Err(SendError::OutOfRange),
        }
    }
}
