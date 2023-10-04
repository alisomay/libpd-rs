#![allow(clippy::restriction)]
#![allow(clippy::unnecessary_cast)]

use libpd_rs::{
    close_patch,
    convenience::dsp_on,
    init, initialize_audio, open_patch,
    receive::{source_to_listen_from_exists, start_listening_from, stop_listening_from},
    types::ReceiverHandle,
};

#[test]
fn listening() {
    let sample_rate = 44100;
    let output_channels = 2;

    init().unwrap();
    initialize_audio(0, output_channels, sample_rate).unwrap();
    dsp_on().unwrap();

    let patch_handle = open_patch("tests/patches/echo.pd").unwrap();

    let result = start_listening_from("list_from_pd");
    assert!(result.is_ok());
    // Just crates the endpoint.
    let result = start_listening_from("non_existent");
    assert!(result.is_ok());

    let handle_1 = start_listening_from("list_from_pd").unwrap();
    let handle_2 = start_listening_from("list_from_pd").unwrap();

    stop_listening_from(handle_1);
    stop_listening_from(handle_2);
    // It will just ignore the null pointer.
    let handle: ReceiverHandle = (std::ptr::null_mut() as *mut std::ffi::c_void).into();
    stop_listening_from(handle);

    assert!(source_to_listen_from_exists("list_from_pd"));
    assert!(!source_to_listen_from_exists("endpoint_not_created"));

    close_patch(patch_handle).unwrap();
}
