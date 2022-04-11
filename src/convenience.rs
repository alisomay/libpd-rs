use crate::error::LibpdError;
use crate::mirror::{
    add_float_to_started_message, finish_message_as_typed_message_and_send_to, start_message,
};

/// Activate audio in pd.
pub fn dsp_on() -> Result<(), LibpdError> {
    start_message(1)?;
    add_float_to_started_message(1.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// De-activate audio in pd.
pub fn dsp_off() -> Result<(), LibpdError> {
    start_message(1)?;
    add_float_to_started_message(0.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}
