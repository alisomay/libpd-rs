#![allow(clippy::restriction)]

use libpd_rs::{
    add_to_search_paths, block_size, clear_search_paths, close_patch, get_dollar_zero, init,
    initialize_audio, open_patch, release_internal_queues, types::PatchFileHandle,
    verbose_print_state, verbose_print_state_active,
};

#[test]
fn all_main_functionality() {
    let x = init();
    assert!(x.is_ok());
    let x = init();
    assert!(x.is_err());

    let x = initialize_audio(0, 2, 44100);
    assert!(x.is_ok());
    let x = initialize_audio(0, 1, 44100);
    assert!(x.is_ok());

    let x = open_patch("tests/patches/no_existent.pd");
    assert!(x.is_err());
    let x = open_patch("/bad_path");
    assert!(x.is_err());
    let x = open_patch("tests/patches/simple.pd");
    assert!(x.is_ok());

    let x = close_patch(x.unwrap());
    assert!(x.is_ok());

    let handle: PatchFileHandle = (std::ptr::null_mut() as *mut std::ffi::c_void).into();
    let x = close_patch(handle);
    assert!(x.is_err());

    let x = add_to_search_paths("tests/patches");
    assert!(x.is_ok());
    let x = add_to_search_paths("/bad_path");
    assert!(x.is_err());

    clear_search_paths();
    release_internal_queues();

    let patch_handle = open_patch("tests/patches/simple.pd").unwrap();

    let x = get_dollar_zero(&patch_handle);
    assert!(x.is_ok());
    assert_ne!(x.unwrap(), 0);

    let handle: PatchFileHandle = (std::ptr::null_mut() as *mut std::ffi::c_void).into();
    let x = get_dollar_zero(&handle);
    assert!(x.is_err());

    let x = block_size();
    assert_eq!(x, 64);

    let x = verbose_print_state_active();
    assert!(!x);
    verbose_print_state(true);
    let x = verbose_print_state_active();
    assert!(x);
    verbose_print_state(false);
    let x = verbose_print_state_active();
    assert!(!x);
}
