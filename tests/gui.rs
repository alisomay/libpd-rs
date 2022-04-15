#![allow(clippy::restriction)]

use libpd_rs::gui::{poll_gui, start_gui, stop_gui};
use std::{env, path::PathBuf};

#[test]
fn start_poll_stop_gui() {
    let _ = libpd_rs::init().unwrap();

    #[cfg(target_os = "macos")]
    {
        let path_to_gui = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/pd_binary/mac/Pd-0.51-4.app/Contents/Resources");

        assert!(start_gui("").is_err());
        start_gui(&path_to_gui).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(3000));

        stop_gui();
    }
}
