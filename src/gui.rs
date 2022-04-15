use crate::{error::GuiLifeCycleError, C_STRING_FAILURE};

use std::ffi::CString;
use std::path::Path;

/// Opens the current patch within a pd vanilla GUI
///
/// This function requires that there is a valid pd installation in your computer and a path to pd's main folder which contains bin/, tcl/, etc.
///
/// # Example
/// ```no_run
/// use libpd_rs::gui::start_gui;
/// use std::path::PathBuf;
///
/// // In mac os probably it would look something like this,
/// // The application name here is an example.
/// let path_to_pd = PathBuf::from("/Applications/Pd-0.51-4.app/Contents/Resources");
/// start_gui(&path_to_pd);
/// ```
pub fn start_gui<T: AsRef<Path>>(path_to_pd: T) -> Result<(), GuiLifeCycleError> {
    if path_to_pd.as_ref().exists() {
        let path_to_pd = path_to_pd.as_ref().to_string_lossy();
        let path_to_pd = CString::new(path_to_pd.as_ref()).expect(C_STRING_FAILURE);
        unsafe {
            match libpd_sys::libpd_start_gui(path_to_pd.as_ptr()) {
                0 => return Ok(()),
                _ => return Err(GuiLifeCycleError::FailedToOpenGui),
            }
        }
    }
    Err(GuiLifeCycleError::FailedToOpenGui)
}

/// Stops the current running pd vanilla GUI if it is running.
pub fn stop_gui() {
    unsafe { libpd_sys::libpd_stop_gui() };
}

/// Manually updates and handles any GUI messages
///
/// This is called automatically when running any process function in the library. e.g. `process_float`.
///
/// Note:
/// - This also facilitates network message processing, etc so it can be useful to call repeatedly when idle for more throughput.
/// - Returns a Some(()) when the polled queue is not empty. In this case it might be desirable to keep polling until it is empty or up to some reasonable limit.
///
/// # Example
/// ```no_run
/// use libpd_rs::gui::poll_gui;
///
/// libpd_rs::init();
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
