#![allow(clippy::restriction)]

use libpd_rs::{
    close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    send::{
        add_double_to_started_message, finish_message_as_list_and_send_to,
        finish_message_as_typed_message_and_send_to, start_message,
    },
};

#[test]
fn send_and_receive_bang() {
    let sample_rate = 44100;
    let output_channels = 2;

    let _ = init().unwrap();
    let _ = initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    // This is strange! I don't think that a message can have this much of length.
    // TODO: Research this.
    let result = start_message(i32::MAX);
    assert!(result.is_ok());
    //

    add_double_to_started_message(0.23);

    let result = finish_message_as_list_and_send_to("no_land");
    assert!(result.is_err());
    let result = finish_message_as_typed_message_and_send_to("no_land", "no_where");
    assert!(result.is_err());

    close_patch(patch_handle).unwrap();
}
