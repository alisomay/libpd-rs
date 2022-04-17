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
fn message_building() {
    let sample_rate = 44100;
    let output_channels = 2;

    let _ = init().unwrap();
    let _ = initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    // The implementation in libpd looks like, where maxlen is the length of the message:
    // t_atom *v = realloc(s_argv, maxlen * sizeof(t_atom));
    // if (v)
    // {
    //   s_argv = v;
    //   s_argm = maxlen;
    // }
    // else
    // {
    //   return -1;
    // }
    //
    // So it is platform dependent for example, it depends on how much memory a process can allocate I guess if this function errors or not.
    // It would be wise to handle the result.
    if start_message(i32::MAX).is_ok() {
        add_double_to_started_message(0.23);

        let result = finish_message_as_list_and_send_to("no_land");
        assert!(result.is_err());
        let result = finish_message_as_typed_message_and_send_to("no_land", "no_where");
        assert!(result.is_err());
    } else {
        start_message(1).unwrap();
        add_double_to_started_message(0.23);

        let result = finish_message_as_list_and_send_to("no_land");
        assert!(result.is_err());
        let result = finish_message_as_typed_message_and_send_to("no_land", "no_where");
        assert!(result.is_err());
    }

    close_patch(patch_handle).unwrap();
}
