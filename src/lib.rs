// @attention Multi instance features implementation is scheduled for later.
// @attention If there is a necessity emerges, I'll give time to implement them.

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
    clippy::integer_arithmetic,
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


    // We decided that we're ok with expect
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    // clippy::self_named_module_files,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    // clippy::wildcard_dependencies,
    // clippy::redundant_feature_names,
    // clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
)]

//! This is the crate level doc.
//!
//! Provides an abstraction over a queue.  When the abstraction is used
//! there are these advantages:
//! - Fast
//! - [`Easy`]
//!
//! [`Easy`]: http://thatwaseasy.example.com

/// Work with pd arrays.
///
/// This module provides all tools to work with pd named arrays which are exposed by libpd with some extra safety such as bounds checking.
///
/// Corresponding libpd functions in [libpd repository](https://github.com/libpd/libpd) could be explored [here](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.h#L115).
pub mod array;
/// Convenience functions which encapsulate common actions when communicating with pd.
///
/// This crate is a thing wrapper around [libpd](https://github.com/libpd/libpd).
/// This module aims to provide functions or structs to add a layer which is easier and quick to use.
/// Now small but might grow in the future.
pub mod convenience;
/// All errors related to libpd_rs.
///
/// [`LibpdError`] is the umbrella error type for all errors related to libpd.
/// All functions which might fail would return a`Result<T, LibpdError>`.
/// From there on one may use pattern matching to get more detailed.
pub mod error;
/// TODO: Module doc.
pub mod gui;
/// TODO: Module doc.
pub(crate) mod helpers;
/// TODO: Module doc.
pub mod process;
/// TODO: Module doc.
pub mod receive;
/// TODO: Module doc.
pub mod send;
/// TODO: Module doc.
pub mod types;

use crate::{
    error::{InitializationError, IoError, LibpdError},
    types::PatchFileHandle,
};

use std::ffi::CString;
use std::path::{Path, PathBuf};

// TODO: Currently panicing is enough since this is a rare case, but may be improved later with a dedicated error.
// Proper errors?
pub(crate) const C_STRING_FAILURE: &str =
    "Provided an invalid CString, check if your string contains null bytes in the middle.";
pub(crate) const C_STR_FAILURE: &str = "Converting a CStr to an &str is failed.";

// TODO: Add errors fields to documentation.

/// Initializes libpd.
///
/// Support for multi instances of pd is not implemented yet.
/// This function should be called after setting any listeners and before any other in this crate.
/// It initializes libpd **globally** and also initializes ring buffers for internal message passing.
/// Sets internal hooks. Then initializes `libpd` by calling the underlying
/// C function which is [`libpd_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/z_libpd.c#L68).
/// See [`libpd_queued_init`](https://github.com/libpd/libpd/blob/master/libpd_wrapper/util/z_queued.c#L308) to
/// explore what it is doing.
///
/// A second call to this function will return an error.
///
/// # Example
/// ```rust
/// use libpd_rs::init;
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

/// Frees the queued ring buffers.
///
/// Currently I don't see a necessity to call this function in any case.
/// Maybe there is a reason.
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
        return Err(LibpdError::IoError(IoError::PathDoesNotExist(
            calculated_patch_path.to_string_lossy().to_string(),
        )));
    }
    // All good.
    unsafe {
        let name = CString::new(file_name).expect(C_STRING_FAILURE);
        let directory = CString::new(directory).expect(C_STRING_FAILURE);
        let file_handle =
            libpd_sys::libpd_openfile(name.as_ptr(), directory.as_ptr()).cast::<std::ffi::c_void>();
        if file_handle.is_null() {
            return Err(LibpdError::IoError(IoError::FailedToOpenPatch));
        }
        Ok(file_handle.into())
    }
}

/// Closes a pd patch which has opened before.
///
/// Handle needs to point to a valid opened patch file.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use libpd_rs::{open_patch, close_patch};
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

/// Gets the `$0` of the running patch.
///
/// `$0` id in pd could be thought as a auto generated unique identifier for the patch.
pub fn get_dollar_zero(handle: &PatchFileHandle) -> Result<i32, LibpdError> {
    unsafe {
        match libpd_sys::libpd_getdollarzero(handle.as_mut_ptr()) {
            0 => Err(LibpdError::IoError(IoError::PatchIsNotOpen)),
            other => Ok(other),
        }
    }
}

/// Returns pd's fixed block size which is 64 by default.
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
