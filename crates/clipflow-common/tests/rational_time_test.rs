use clipflow_common::time::{RationalTime, TimeRange};

#[test]
fn test_rational_time_rescale() {
    let t1 = RationalTime::new(1500, 1000); // 1.5 seconds
    let t2 = t1.rescaled_to(60000);

    assert_eq!(t2.value, 90000);
    assert_eq!(t2.timescale, 60000);
    
    // Convert back should maintain precision
    let t3 = t2.rescaled_to(1000);
    assert_eq!(t3.value, 1500);
}

#[test]
fn test_rational_time_overflow_safety() {
    // A value that when multiplied by a large timescale would overflow i64, 
    // but the final rescaled value still fits in i64.
    let val: i64 = 4000000000000000000; // ~4*10^18 (i64 MAX is ~9*10^18)
    let _t1 = RationalTime::new(val, 1000); // Very large time in milliseconds
    
    // Scale it to 60000. 
    // Internally it will do 4000000000000000000 * 60000 which is ~2.4*10^23 
    // This > i64::MAX, so i128 is needed for intermediate result.
    // The final result is / 1000 = ~2.4*10^20, which is STILL > i64::MAX.
    
    // Let's use a value where final fits in i64.
    // i64::MAX / 60 = ~1.5*10^17.
    // Let's pick val = 1.5*10^17. 
    // When multiplying by 60000, it's 9*10^21 > i64::MAX.
    // Then dividing by 1000 gives 9*10^18 < i64::MAX.
    let val: i64 = 150_000_000_000_000_000;
    let t1 = RationalTime::new(val, 1000);
    let t2 = t1.rescaled_to(60000);
    
    // Expected result
    let expected = (val as i128 * 60000) / 1000;
    assert_eq!(t2.value as i128, expected);
    assert_eq!(t2.timescale, 60000);
}

#[test]
fn test_time_range_contains_and_end_exclusive() {
    let start = RationalTime::new(1000, 1000); // 1.0s
    let duration = RationalTime::new(2500, 1000); // 2.5s
    let range = TimeRange::new(start, duration);

    // End should be exactly 3.5s
    let end = range.end_exclusive();
    assert_eq!(end.value, 3500);
    assert_eq!(end.timescale, 1000);

    // Inside range
    assert!(range.contains(RationalTime::new(2000, 1000)));
    assert!(range.contains(RationalTime::new(1000, 1000))); // Left inclusive

    // Outside range
    assert!(!range.contains(RationalTime::new(3500, 1000))); // Right exclusive
    assert!(!range.contains(RationalTime::new(999, 1000)));
}
