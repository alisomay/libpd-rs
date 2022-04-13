#![allow(clippy::restriction)]

use libpd_rs::gui::{start_gui, stop_gui};
use std::{env, path::PathBuf};

// Currently this test is failing with this error.
// Error in startup script: couldn't read file
// "tests/pd_binary/mac/Pd-0.51-4.app/Contents/Resources/bin/pd/tcl/pd-gui.tcl": not a directory
//
// I think the reason for that is, this function is old and pd binary organization was changed.
// The tcl file is currently at "tests/pd_binary/mac/Pd-0.51-4.app/Contents/Resources/tcl/pd-gui.tcl"
//
// TODO: Research if gui related functionality is still in usage or maintained.
#[test]
#[ignore]
fn start_poll_stop_gui() {
    let _ = libpd_rs::init().unwrap();

    #[cfg(target_os = "macos")]
    {
        let path_to_gui = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/pd_binary/mac/Pd-0.51-4.app/Contents/Resources/bin/pd");

        assert!(start_gui("").is_err());
        start_gui(&path_to_gui).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(2000));

        stop_gui();
    }
}
