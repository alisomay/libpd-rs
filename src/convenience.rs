#![allow(dead_code)]

use crate::error::{InitializationError, LibpdError};
use crate::send::{
    add_float_to_started_message, finish_message_as_typed_message_and_send_to, start_message,
};
use crate::{init, initialize_audio};

//TODO: Revisit these errors

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

// calc ticks
pub struct Pd {
    audio_active: bool,
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
}

impl Pd {
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
        })
    }

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
