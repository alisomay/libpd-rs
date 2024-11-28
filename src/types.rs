use core::ffi;

/// The handle which is returned from opening a patch.
///
/// This is a [`c_void`](std::ffi::c_void) in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle should be kept alive for the lifetime of the patch.
#[derive(Debug)]
pub struct PatchFileHandle(*mut ffi::c_void);
impl PatchFileHandle {
    pub(crate) const fn as_mut_ptr(&self) -> *mut ffi::c_void {
        self.0
    }

    pub const fn into_inner(self) -> *mut ffi::c_void {
        self.0
    }
}

impl From<*mut ffi::c_void> for PatchFileHandle {
    fn from(ptr: *mut ffi::c_void) -> Self {
        Self(ptr)
    }
}

/// The handle which is returned from subscribing to a sender.
///
/// This is a [`c_void`](std::ffi::c_void) in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle could be used later to unsubscribe from the sender.
#[derive(Debug)]
pub struct ReceiverHandle(*mut ffi::c_void);

impl ReceiverHandle {
    pub const fn into_inner(self) -> *mut ffi::c_void {
        self.0
    }
}

impl From<*mut ffi::c_void> for ReceiverHandle {
    fn from(ptr: *mut ffi::c_void) -> Self {
        Self(ptr)
    }
}
