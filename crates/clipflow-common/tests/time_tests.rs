use clipflow_common::{FrameRate, RationalTime, SmpteTimecode, TimeRange};

#[test]
fn test_rational_time_basic_math() {
    let t1 = RationalTime::new(1000, 1000); // 1.0s
    let t2 = RationalTime::new(500, 1000);  // 0.5s

    let sum = t1 + t2;
    assert_eq!(sum.value, 1500);
    assert_eq!(sum.timescale, 1000);
    assert_eq!(sum.to_seconds(), 1.5);

    let diff = t1 - t2;
    assert_eq!(diff.value, 500);
    assert_eq!(diff.timescale, 1000);
    assert_eq!(diff.to_seconds(), 0.5);

    assert!(t1 > t2);
    assert!(t2 < t1);
    assert_eq!(RationalTime::new(2000, 2000), t1);
}

#[test]
fn test_rational_time_rescale_precision() {
    // 24fps 下的 48 帧 (2.0s) 换算为 60000 timescale
    let t = RationalTime::new(48, 24);
    let rescaled = t.rescaled_to(60000);
    assert_eq!(rescaled.value, 120000);
    assert_eq!(rescaled.timescale, 60000);

    // 大数值乘法防止 i64 溢出验证 (i128 宽整型计算)
    let large_t = RationalTime::new(10_000_000_000, 1000); // 1000 万秒
    let rescaled_large = large_t.rescaled_to(60_000);
    assert_eq!(rescaled_large.value, 600_000_000_000);
    assert_eq!(rescaled_large.timescale, 60_000);
}

#[test]
#[should_panic(expected = "Timescale must be positive")]
fn test_rational_time_div_by_zero_defense() {
    let _ = RationalTime::new(100, 0);
}

#[test]
fn test_time_range_half_open_interval() {
    let start = RationalTime::new(1000, 1000);    // 1.0s
    let duration = RationalTime::new(2000, 1000); // 2.0s
    let range = TimeRange::new(start, duration);

    let end = range.end_exclusive();
    assert_eq!(end.value, 3000); // 3.0s

    // 左闭: 包含起点
    assert!(range.contains(RationalTime::new(1000, 1000)));
    // 中间包含
    assert!(range.contains(RationalTime::new(2000, 1000)));
    assert!(range.contains(RationalTime::new(2999, 1000)));
    // 右开: 不包含终点
    assert!(!range.contains(RationalTime::new(3000, 1000)));
    // 小于起点不包含
    assert!(!range.contains(RationalTime::new(999, 1000)));
}

#[test]
fn test_smpte_timecode_format_non_drop_frame() {
    let tc = SmpteTimecode {
        hours: 1,
        minutes: 23,
        seconds: 45,
        frames: 12,
        is_drop_frame: false,
    };
    assert_eq!(tc.to_string(), "01:23:45:12");
}

#[test]
fn test_smpte_timecode_format_drop_frame() {
    let tc = SmpteTimecode {
        hours: 1,
        minutes: 23,
        seconds: 45,
        frames: 12,
        is_drop_frame: true,
    };
    assert_eq!(tc.to_string(), "01:23:45;12");
}

#[test]
fn test_smpte_timecode_from_rational_time() {
    // 25 FPS PAL 制式: 1小时 0分 0秒 0帧 = 3600 秒 * 25 = 90000 帧
    let t = RationalTime::new(90000, 25);
    let tc = SmpteTimecode::from_rational_time(t, FrameRate::Fps25);
    assert_eq!(tc.hours, 1);
    assert_eq!(tc.minutes, 0);
    assert_eq!(tc.seconds, 0);
    assert_eq!(tc.frames, 0);
    assert_eq!(tc.to_string(), "01:00:00:00");

    // 25 FPS 下的 1秒 12帧 = 37 帧
    let t2 = RationalTime::new(37, 25);
    let tc2 = SmpteTimecode::from_rational_time(t2, FrameRate::Fps25);
    assert_eq!(tc2.hours, 0);
    assert_eq!(tc2.minutes, 0);
    assert_eq!(tc2.seconds, 1);
    assert_eq!(tc2.frames, 12);
    assert_eq!(tc2.to_string(), "00:00:01:12");
}
