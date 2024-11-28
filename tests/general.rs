#![allow(clippy::restriction)]
#![allow(clippy::unnecessary_cast)]

use libpd_rs::functions::{
    add_to_search_paths, block_size, clear_search_paths, close_patch, get_dollar_zero, init,
    initialize_audio, open_patch, verbose_print_state, verbose_print_state_active,
};
use libpd_rs::types::PatchFileHandle;

#[test]
fn all_main_functionality() {
    let result = init();
    assert!(result.is_ok());

    let result = init();
    assert!(result.is_ok());

    let result = initialize_audio(0, 2, 44100);
    assert!(result.is_ok());
    let result = initialize_audio(0, 1, 44100);
    assert!(result.is_ok());

    let result = open_patch("tests/patches/no_eresultistent.pd");
    assert!(result.is_err());
    let result = open_patch("/bad_path");
    assert!(result.is_err());
    let result = open_patch("tests/patches/simple.pd");
    assert!(result.is_ok());

    let result = close_patch(result.unwrap());
    assert!(result.is_ok());

    let handle: PatchFileHandle = (std::ptr::null_mut() as *mut std::ffi::c_void).into();
    let result = close_patch(handle);
    assert!(result.is_err());

    let result = add_to_search_paths("tests/patches");
    assert!(result.is_ok());
    let result = add_to_search_paths("/bad_path");
    assert!(result.is_err());

    clear_search_paths();

    let patch_handle = open_patch("tests/patches/simple.pd").unwrap();

    let result = get_dollar_zero(&patch_handle);
    assert!(result.is_ok());
    assert_ne!(result.unwrap(), 0);

    let handle: PatchFileHandle = (std::ptr::null_mut() as *mut std::ffi::c_void).into();
    let result = get_dollar_zero(&handle);
    assert!(result.is_err());

    let result = block_size();
    assert_eq!(result, 64);

    let result = verbose_print_state_active();
    assert!(!result);
    verbose_print_state(true);
    let result = verbose_print_state_active();
    assert!(result);
    verbose_print_state(false);
    let result = verbose_print_state_active();
    assert!(!result);
}
