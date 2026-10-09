use clipflow_common::time::SmpteTimecode;

#[test]
fn test_smpte_non_drop_frame_formatting() {
    let tc = SmpteTimecode {
        hours: 1,
        minutes: 23,
        seconds: 45,
        frames: 12,
        is_drop_frame: false,
    };
    // Non-drop frame should use colon separator
    assert_eq!(tc.to_string(), "01:23:45:12");
}

#[test]
fn test_smpte_drop_frame_formatting() {
    let tc = SmpteTimecode {
        hours: 1,
        minutes: 23,
        seconds: 45,
        frames: 12,
        is_drop_frame: true,
    };
    // Drop frame should use semicolon separator for frames
    assert_eq!(tc.to_string(), "01:23:45;12");
}
