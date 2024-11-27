use std::path::PathBuf;

use libpd_rs::Pd;

#[test]
fn search_paths() {
    let mut pd = Pd::init_and_configure(0, 2, 44100).unwrap();

    assert!(pd.add_path_to_search_paths("does not exist").is_err());
    assert!(pd
        .add_path_to_search_paths(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .is_ok());
    assert!(pd
        .add_path_to_search_paths(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .is_ok());
    assert_eq!(
        pd.search_paths
            .iter()
            .filter(|p| p.to_str().unwrap() == env!("CARGO_MANIFEST_DIR"))
            .count(),
        1
    );
    assert_eq!(
        pd.search_paths[0].to_string_lossy(),
        env!("CARGO_MANIFEST_DIR").to_string()
    );

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let other = base.join("tests");
    pd.add_paths_to_search_paths(&[base.clone(), other.clone()])
        .unwrap();

    assert_eq!(pd.search_paths[0].to_string_lossy(), base.to_string_lossy());
    assert_eq!(
        pd.search_paths[1].to_string_lossy(),
        other.to_string_lossy()
    );

    assert!(pd.add_paths_to_search_paths(&["does not exist"]).is_err());

    pd.clear_all_search_paths();

    assert_eq!(pd.search_paths.len(), 0);
}
