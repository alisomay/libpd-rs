use crate::{
    error::{LibpdError, SubscriptionError},
    helpers::make_atom_list_from_t_atom_list,
    types::{Atom, ReceiverHandle},
    C_STRING_FAILURE, C_STR_FAILURE,
};

use libffi::high::{ClosureMut1, ClosureMut2, ClosureMut3, ClosureMut4};
use std::ffi::{CStr, CString};

/// Subscribes to messages sent to a receiver in the loaded pd patch
///
/// `start_listening_from("foo")` would add a **virtual** `|r foo|` which would
/// forward messages to the libpd message listeners.
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
/// use libpd_rs::receive::{start_listening_from};
/// use libpd_rs::types::ReceiverHandle;
///
/// libpd_rs::init();
///
/// let sources = vec!["foo", "bar"];
/// // Maybe you would like to use the receiver handles later so you may store them..
/// let mut handles: HashMap<String, ReceiverHandle> = HashMap::new();
/// for source in sources {
///     start_listening_from(&source).map_or_else(|err| {
///         // Handle the error creating a receiving endpoint failed.
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
/// use libpd_rs::receive::{start_listening_from, stop_listening_from};
///
/// libpd_rs::init();
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

/// Checks if a source to listen from exists.
///
/// # Example
/// ```rust
/// use libpd_rs::receive::{source_to_listen_from_exists, start_listening_from};
///
/// libpd_rs::init();
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

/// Sets a closure to be called when a message is written to the pd console.
///
/// There is also no prior call to `start_listening_from` to listen from pd console.
///  Do not register this listener while pd DSP is running.
///
/// # Example
/// ```rust
/// use libpd_rs::receive::{on_print};
///
/// on_print(|msg: &str| {
///  println!("pd is printing: {msg}");
/// });
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_bang, start_listening_from};
///
/// on_bang(|source: &str| {
///   match source {
///     "foo" => println!("bang from foo"),   
///     "bar" => println!("bang from bar"),
///      _ => unreachable!(),
///   }
/// });
///
/// libpd_rs::init();
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
    unsafe { libpd_sys::libpd_set_queued_banghook(Some(*ptr)) };
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
/// use libpd_rs::receive::{on_float, start_listening_from};
///
/// on_float(|source: &str, value: f32| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {value}"),  
///     "bar" =>  println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_double, start_listening_from};
///
/// on_double(|source: &str, value: f64| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {value}"),  
///     "bar" =>  println!("Received a float from bar, value is: {value}"),
///      _ => unreachable!(),
///   }
/// });
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_symbol, start_listening_from};
///
/// on_symbol(|source: &str, symbol: &str| {
///   match source {
///     "foo" =>  println!("Received a float from foo, value is: {symbol}"),  
///     "bar" =>  println!("Received a float from bar, value is: {symbol}"),
///      _ => unreachable!(),
///   }
/// });
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_list, start_listening_from};
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
/// libpd_rs::init();
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

/// Sets a closure to be called when a typed message is received from a subscribed receiver
///
/// In a message like [; foo hello 1.0 merhaba] which is sent from the patch,
///
/// To receive the message, you need to subscribe to the receiver with the name "foo".
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
/// use libpd_rs::receive::{on_message, start_listening_from};
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
/// libpd_rs::init();
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

/// Receives messages from pd message queue.
///
/// This should be called repeatedly in the **application's main loop** to fetch messages from pd.
///
/// # Example
/// ```no_run
/// use libpd_rs::receive::{start_listening_from, on_symbol, receive_messages_from_pd};
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
/// use libpd_rs::receive::{on_midi_note_on};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_control_change};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_program_change};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_pitch_bend};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_after_touch};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_poly_after_touch};
///
/// libpd_rs::init();
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
/// use libpd_rs::receive::{on_midi_byte};
///
/// libpd_rs::init();
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

/// Receives messages from pd midi message queue.
///
/// This should be called repeatedly in the **application's main loop** to fetch midi messages from pd.
///
/// # Example
/// ```no_run
/// use libpd_rs::receive::{on_midi_byte, receive_midi_messages_from_pd};
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
