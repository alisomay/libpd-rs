#![allow(clippy::restriction)]
#![allow(unused)]

use libpd_rs::functions::gui::{poll_gui, start_gui, stop_gui};
use std::{env, path::PathBuf};

#[test]
fn start_poll_stop_gui() {
    libpd_rs::functions::init().unwrap();

    #[cfg(target_os = "macos")]
    {
        let path_to_gui = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pd_binary")
            .join("mac")
            .join("Pd-0.51-4.app")
            .join("Contents")
            .join("Resources");

        assert!(start_gui("").is_err());
        start_gui(&path_to_gui).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(3000));

        #[allow(clippy::redundant_pattern_matching)]
        while let Some(_) = poll_gui() {
            // Do something
        }

        std::thread::sleep(std::time::Duration::from_millis(300));

        stop_gui();
    }

    #[cfg(target_os = "windows")]
    {
        let path_to_gui = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pd_binary")
            .join("win")
            .join("Pd-0.52-2");

        assert!(start_gui("").is_err());
        start_gui(&path_to_gui).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(3000));

        #[allow(clippy::redundant_pattern_matching)]
        while let Some(_) = poll_gui() {
            // Do something
        }

        std::thread::sleep(std::time::Duration::from_millis(300));

        stop_gui();
    }

    // TODO: Extend gui tests with linux.
}
