use crate::error::{IoError, LibpdError, SendError};
use crate::mirror::{
    add_float_to_started_message, close_patch, finish_message_as_typed_message_and_send_to, init,
    open_patch, start_message,
};
use crate::types::{Atom, PatchFileHandle};

pub fn dsp_on() -> Result<(), LibpdError> {
    start_message(1)?;
    add_float_to_started_message(1.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}
pub fn dsp_off() -> Result<(), LibpdError> {
    start_message(1)?;
    add_float_to_started_message(0.0);
    finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}
