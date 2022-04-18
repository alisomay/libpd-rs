use libpd_rs::convenience::PdGlobal;

#[test]
fn state() {
    let mut pd = PdGlobal::init_and_configure(0, 2, 44100).unwrap();
    pd.open_patch("tests/patches/sine.pd").unwrap();
    assert!(!pd.audio_active());
    pd.activate_audio(true).unwrap();
    assert!(pd.audio_active());
    pd.activate_audio(false).unwrap();
    assert!(!pd.audio_active());

    assert_eq!(pd.sample_rate(), 44100);
    assert_eq!(pd.input_channels(), 0);
    assert_eq!(pd.output_channels(), 2);
}
