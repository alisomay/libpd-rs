use libffi::high::{ClosureMut1, FnPtr1};

//`libpdimp_free` and `libpdimp_new` are the only functions that are not exposed. We don't want to forget them for the future.
// use libpd_sys::{libpdimp_free, libpdimp_new};
use libpd_sys::{
    _pdinstance, libpd_free_instance, libpd_get_instancedata, libpd_main_instance,
    libpd_new_instance, libpd_num_instances, libpd_set_instance, libpd_set_instancedata,
    libpd_this_instance, t_libpd_freehook,
};
use std::{any::TypeId, ffi::c_void, mem};

use crate::{error::InstanceError, functions};

type FreeHookCodePtr = *const FnPtr1<'static, *mut c_void, ()>;

/// A Pure Data instance that can be used to process audio and handle Pd patches.
///
/// # Thread Safety
///
/// This type is both `Send` and `Sync` because:
/// - The underlying Pd instance maintains its own internal thread synchronization
/// - All mutations of the instance state are done through libpd's thread-safe API
/// - The internal pointer is only modified through synchronized libpd functions
/// - Instance data access is protected by type checking
///
/// While the type is thread-safe, users should note that:
/// - Only one thread should process audio at a time for a given instance
/// - The instance should be set as current before processing audio or sending messages
/// - The instance remains valid as long as this struct exists
#[derive(Debug, Clone, Eq)]
pub struct PdInstance {
    inner: *mut _pdinstance,
    number: i32,
    // Add a field to track the type
    stored_type: Option<TypeId>,
}

impl PartialEq for PdInstance {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

// Safe because libpd handles internal synchronization and we maintain
// exclusive access to the pointer through the public API
unsafe impl Send for PdInstance {}

// Safe because all methods that mutate state use internal libpd locks
// and our public API ensures thread-safe access to the instance pointer
unsafe impl Sync for PdInstance {}

impl PdInstance {
    /// Create a new instance of Pd.
    ///
    /// If this is the first instance created, it will be the main instance.
    /// The main instance is always valid.
    ///
    /// If this is not the first instance created, it will be a new instance.
    ///
    /// All instances created come initialized.
    ///
    /// Dropping an instance will free its resources properly.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`InstanceFailedToCreate`](crate::error::InstanceError::InstanceFailedToCreate)
    pub fn new() -> Result<Self, InstanceError> {
        // First instance as the main instance.
        if instance_count() == 0 {
            functions::init()
                .map_err(|err| InstanceError::InstanceFailedToCreate(err.to_string()))?;
            let main_instance_ptr = unsafe { libpd_main_instance() };
            // Set the current instance to the main instance.
            unsafe {
                libpd_set_instance(main_instance_ptr);
            }

            return Ok(Self {
                inner: main_instance_ptr,
                // Since we've just successfully created the main instance, it's safe to dereference here.
                number: unsafe { (*main_instance_ptr).pd_instanceno },
                stored_type: None,
            });
        }

        let currently_set_instance_ptr = unsafe { libpd_this_instance() };
        let new_instance_ptr = unsafe { libpd_new_instance() };

        if currently_set_instance_ptr.is_null() || new_instance_ptr.is_null() {
            return Err(InstanceError::InstanceFailedToCreate(
                "Returned instance pointer is null.".to_owned(),
            ));
        }

        // Set the current instance to the new instance to initialize it.
        unsafe {
            libpd_set_instance(new_instance_ptr);
        }

        // TODO: Learn why it is required to be called after each instance creation and returns like the global init. (low priority)
        functions::init().map_err(|err| InstanceError::InstanceFailedToCreate(err.to_string()))?;

        // Set the current instance back to the previous instance.
        unsafe {
            libpd_set_instance(currently_set_instance_ptr);
        }

        Ok(Self {
            inner: new_instance_ptr,
            // Since we've just successfully created the main instance, it's safe to dereference here.
            number: unsafe { (*new_instance_ptr).pd_instanceno },
            stored_type: None,
        })
    }

    /// Get the raw pointer to the internal pd instance.
    ///
    /// From this point on you're responsible for managing the instance's lifetime.
    ///
    /// If you are not sure about what you're doing do not use this method.
    ///
    /// # Important
    /// The caller must ensure they don't violate pd's threading and ownership rules
    /// when using this pointer.
    pub const fn as_ptr(&self) -> *mut _pdinstance {
        self.inner
    }

    /// Makes this instance the current instance.
    ///
    /// So that all subsequent calls to libpd functions will be made on this instance.
    pub fn set_as_current(&mut self) {
        unsafe { libpd_set_instance(self.inner) }
    }

    /// Gets the instance number of this instance.
    ///
    /// Returns `pd_instanceno`.
    pub const fn number(&self) -> i32 {
        self.number
    }

    /// Gets the system time of this instance.
    ///
    /// Returns `pd_systime`.
    pub fn system_time(&self) -> f64 {
        unsafe { (*self.inner).pd_systime }
    }

    /// Gets if this instance is locked.
    ///
    /// Returns `pd_islocked`.
    pub fn is_locked(&self) -> bool {
        unsafe { (*self.inner).pd_islocked != 0 }
    }

    /// Checks if this instance is the main instance.
    ///
    /// The main instance is the first instance created.
    ///
    /// The main instance is always valid.
    pub fn is_main_instance(&self) -> bool {
        let main_instance = unsafe { libpd_main_instance() };
        // # Safety
        // Main instance is always valid, it is safe to dereference here.
        let main_instance = unsafe { &mut *main_instance };
        main_instance.pd_instanceno == self.number
    }

    /// Checks if this instance is set as the current instance.
    pub fn is_current_instance(&self) -> bool {
        unsafe {
            if libpd_this_instance().is_null() {
                return false;
            }
        }

        let current_instance = unsafe { libpd_this_instance() };
        // # Safety
        // We've done a null check above, it is safe to dereference here.
        let current_instance = unsafe { &mut *current_instance };
        current_instance.pd_instanceno == self.number
    }

    /// Set custom instance data with an optional free hook
    ///
    /// We expose this since it is a library function but I'm not sure if it is useful.
    pub fn set_instance_data<T, F>(&mut self, data: T, free_hook: Option<F>)
    where
        T: 'static + Send + Sync,
        F: FnMut(&mut T) + Send + Sync + 'static,
    {
        let boxed = Box::new(data);

        // Handle the free hook if provided
        let hook_ptr = free_hook.and_then(|mut free_hook| {
            let closure: &'static mut _ = Box::leak(Box::new(move |data: *mut c_void| {
                // Convert back to the original type for the user's closure
                let data = unsafe { &mut *data.cast::<T>() };
                free_hook(data);
            }));
            let callback = ClosureMut1::new(closure);
            let code = callback.code_ptr() as FreeHookCodePtr;
            let ptr = unsafe { *code.cast::<t_libpd_freehook>() };
            mem::forget(callback);
            ptr
        });

        unsafe {
            libpd_set_instancedata(Box::into_raw(boxed).cast(), hook_ptr);
        }

        self.stored_type = Some(TypeId::of::<T>());
    }

    /// Get custom instance data
    ///
    /// Returns `None`
    /// - If the stored type does not match the requested type.
    /// - If no data is stored.
    pub fn get_instance_data<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        // Check if the requested type matches what was stored
        match self.stored_type {
            Some(stored) if stored == TypeId::of::<T>() => {
                let ptr = unsafe { libpd_get_instancedata() };
                if ptr.is_null() {
                    None
                } else {
                    // # Safety
                    // We've checked the type above, it is safe to dereference here.
                    Some(unsafe { &*(ptr as *const T) })
                }
            }
            _ => None,
        }
    }
}

impl Drop for PdInstance {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }

        // The advice below can be found in z_queued.h:

        // free the queued ringbuffers
        // with multiple instances, call before freeing each instance:
        //     libpd_set_instance(pd1);
        //     libpd_queued_release();
        //     libpd_free_instance(pd1);

        self.set_as_current();
        functions::release_internal_queues();
        unsafe { libpd_free_instance(self.inner) }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "The instance count can not be negative."
)]
/// Gets the number of instances of Pd registered.
pub fn instance_count() -> usize {
    unsafe { libpd_num_instances() as usize }
}
