#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{InitializationError, LibpdError};
use crate::send::{
    add_float_to_started_message, finish_message_as_typed_message_and_send_to, start_message,
};
use crate::types::{PatchFileHandle, ReceiverHandle};
use crate::{init, initialize_audio};

/// Activates audio in pd.
pub fn dsp_on() -> Result<(), Box<dyn LibpdError>> {
    start_message(1)?;
    add_float_to_started_message(1.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// De-activates audio in pd.
pub fn dsp_off() -> Result<(), Box<dyn LibpdError>> {
    start_message(1)?;
    add_float_to_started_message(0.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// Find the number of pd ticks according to the case.
#[must_use]
#[allow(clippy::integer_division)]
pub fn calculate_ticks(channels: i32, buffer_size: i32) -> i32 {
    let block_size = crate::block_size();
    buffer_size / (block_size * channels)
}

pub struct PdGlobal {
    audio_active: bool,
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
    running_patch: Option<PatchFileHandle>,
    subscriptions: HashMap<String, ReceiverHandle>,
}

impl PdGlobal {
    pub fn init_and_configure(
        input_channels: i32,
        output_channels: i32,
        sample_rate: i32,
    ) -> Result<Self, Box<dyn LibpdError>> {
        match init() {
            Ok(_) => (),
            Err(err) => match err {
                // Ignore re-initialization errors.
                InitializationError::AlreadyInitialized => (),
                err => return Err(err.into()),
            },
        }
        initialize_audio(input_channels, output_channels, sample_rate)?;
        Ok(Self {
            audio_active: false,
            input_channels,
            output_channels,
            sample_rate,
            running_patch: None,
            subscriptions: HashMap::default(),
        })
    }

    /// Closes a pd patch.
    pub fn close_patch(&mut self) -> Result<(), Box<dyn LibpdError>> {
        if let Some(handle) = self.running_patch.take() {
            crate::close_patch(handle)?;
        }
        Ok(())
    }

    /// Opens a pd patch.
    pub fn open_patch<T: AsRef<Path>>(&mut self, path: T) -> Result<(), Box<dyn LibpdError>> {
        self.running_patch = Some(crate::open_patch(path)?);
        Ok(())
    }

    /// Starts listening messages from a source.
    ///
    /// If the source is already being listened to, this function will early return but not error.
    ///
    /// # Errors
    ///
    /// Errors if the subscription fails.
    ///
    /// Which may be down casted to [`SubscriptionError::FailedToSubscribeToSender`]
    pub fn subscribe_to<T: AsRef<str>>(&mut self, source: T) -> Result<(), Box<dyn LibpdError>> {
        if self.subscriptions.contains_key(source.as_ref()) {
            return Ok(());
        }
        self.subscriptions.insert(
            source.as_ref().to_owned(),
            crate::receive::start_listening_from(source.as_ref())?,
        );
        Ok(())
    }

    /// Starts listening messages from many source.
    ///
    /// If the any source is already being listened to, this function will will ignore them.
    ///
    /// # Errors
    ///
    /// Errors if any of the subscriptions fail.
    ///
    /// Which may be down casted to [`SubscriptionError::FailedToSubscribeToSender`]
    pub fn subscribe_to_many<T: AsRef<str>>(
        &mut self,
        sources: &[T],
    ) -> Result<(), Box<dyn LibpdError>> {
        for source in sources {
            if self.subscriptions.contains_key(source.as_ref()) {
                continue;
            }
            self.subscriptions.insert(
                source.as_ref().to_owned(),
                crate::receive::start_listening_from(source.as_ref())?,
            );
        }
        Ok(())
    }

    /// Stops listening messages from a source.
    pub fn unsubscribe_from<T: AsRef<str>>(&mut self, source: T) {
        if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
            crate::receive::stop_listening_from(handle);
        }
    }

    /// Stops listening messages from many sources.
    pub fn unsubscribe_from_many<T: AsRef<str>>(&mut self, sources: &[T]) {
        for source in sources {
            if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
                crate::receive::stop_listening_from(handle);
            }
        }
    }

    /// Checks if the audio is active.
    ///
    /// The state is tracked by [`PdGlobal`].
    ///
    /// If messages sent to pd previously to activate or de-activate audio not using methods provided in this struct.
    /// This state might be false.
    #[must_use]
    pub const fn audio_active(&self) -> bool {
        self.audio_active
    }

    /// Activates audio in pd.
    pub fn dsp_on(&mut self) -> Result<(), Box<dyn LibpdError>> {
        dsp_on()?;
        self.audio_active = true;
        Ok(())
    }

    /// De-activates audio in pd.
    pub fn dsp_off(&mut self) -> Result<(), Box<dyn LibpdError>> {
        dsp_off()?;
        self.audio_active = false;
        Ok(())
    }
}
