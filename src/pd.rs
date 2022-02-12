use crate::error::{IoError, SendError};
use crate::types::{Atom, PatchFileHandle};
use crate::{
    add_float_to_started_message, close_patch, finish_message_as_typed_message_and_send_to, init,
    open_patch, start_message,
};

pub struct Pd {
    active: bool,
    open_patch: Option<PatchFileHandle>,
}

impl Default for Pd {
    fn default() -> Self {
        // TODO: Change this unwrap to a Result
        init().unwrap();
        Self {
            active: false,
            open_patch: None,
        }
    }
}

impl Pd {
    pub fn open(&mut self, path_to_patch: &std::path::Path) -> Result<(), IoError> {
        if let Ok(handle) = open_patch(path_to_patch) {
            self.open_patch = Some(handle);
            Ok(())
        } else {
            Err(IoError::FailedToOpenPatch)
        }
    }
    pub fn close(&mut self) {
        if let Some(handle) = self.open_patch.take() {
            close_patch(handle);
        }
    }
    pub fn dsp_on(&mut self) -> Result<(), SendError> {
        start_message(1);
        add_float_to_started_message(1.0);
        if finish_message_as_typed_message_and_send_to("pd", "dsp").is_ok() {
            self.active = true;
            Ok(())
        } else {
            Err(SendError::MissingDestination("pd".to_owned()))
        }
    }
    pub fn dsp_off(&mut self) -> Result<(), SendError> {
        start_message(1);
        add_float_to_started_message(0.0);
        if finish_message_as_typed_message_and_send_to("pd", "dsp").is_ok() {
            self.active = false;
            Ok(())
        } else {
            Err(SendError::MissingDestination("pd".to_owned()))
        }
    }
    pub fn is_dsp_on(&self) -> bool {
        self.active
    }
}
