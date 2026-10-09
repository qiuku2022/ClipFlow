use clipflow_common::time::{AgentTimelineAcl, AgentCutRequest, RationalTime};

#[test]
fn test_seconds_to_snapped_time() {
    let timebase = 60000;
    let fps_denominator = 1001; // For 59.94 fps

    // 1 frame duration is ~ 0.016683 seconds
    // Let's test a time that lands near the first frame boundary
    let seconds = 0.016; 
    let snapped = AgentTimelineAcl::seconds_to_snapped_time(seconds, timebase, fps_denominator);
    
    // It should snap to the first frame (index 1) => value = 1 * 1001 = 1001
    assert_eq!(snapped.value, 1001);
    assert_eq!(snapped.timescale, timebase);

    // Negative times should snap to 0
    let negative = AgentTimelineAcl::seconds_to_snapped_time(-1.0, timebase, fps_denominator);
    assert_eq!(negative.value, 0);
}

#[test]
fn test_sanitize_and_stitch_cuts_gap_elimination() {
    let raw_cuts = vec![
        AgentCutRequest {
            start_time_seconds: 0.0,
            end_time_seconds: 1.0, // 60 frames roughly
            reason: "First cut".to_string(),
        },
        AgentCutRequest {
            start_time_seconds: 1.01, // Very close to the end of the previous cut
            end_time_seconds: 2.0,
            reason: "Second cut".to_string(),
        }
    ];

    let timebase = 60000;
    let fps_denominator = 1000; // Exact 60 fps for simpler math (frame dur = 1000/60000 = 1/60)
    let source_duration = RationalTime::new(120000, 60000); // 2.0s

    let stitched = AgentTimelineAcl::sanitize_and_stitch_cuts(&raw_cuts, source_duration, timebase, fps_denominator);
    
    // There should be 1 stitched cut because the <= 1 frame gap was eliminated by merging
    assert_eq!(stitched.len(), 1);
    
    // First cut is exactly [0, 120 frames) = [0, 120000 ticks)
    assert_eq!(stitched[0].start.value, 0);
    assert_eq!(stitched[0].duration.value, 120000);
}
