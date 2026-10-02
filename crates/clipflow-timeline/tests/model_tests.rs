use clipflow_common::{RationalTime, TimeRange};
use clipflow_timeline::models::{
    Animatable, Asset, AssetKind, AssetPool, CanvasSize, Clip, ClipPayload, Interpolation,
    Keyframe, Project, Sequence, Track, TrackAudioProperties, TrackKind,
    Transform2D, AudioProperties,
};
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_project_hierarchy_and_flat_indexing() {
    let mut asset_pool = AssetPool::new();
    let asset_id = Uuid::new_v4();
    let asset = Asset {
        id: asset_id,
        name: "interview_a_roll.mp4".to_string(),
        absolute_path: PathBuf::from(r"C:\Media\interview_a_roll.mp4"),
        relative_path: Some(PathBuf::from("interview_a_roll.mp4")),
        file_size: 1024 * 1024 * 500,
        sha256_hash: "abcd1234ef567890".to_string(),
        kind: AssetKind::Video {
            width: 1920,
            height: 1080,
            has_audio: true,
        },
        native_duration: RationalTime::new(3600, 1),
        native_timebase: 60,
        proxy_path: None,
    };
    asset_pool.insert(asset);

    let clip_id = Uuid::new_v4();
    let clip = Clip {
        id: clip_id,
        name: "Clip 01".to_string(),
        asset_id,
        source_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
        timeline_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
        speed: 1.0,
        transform: Transform2D::default(),
        audio_props: AudioProperties::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    };

    let track_id = Uuid::new_v4();
    let track = Track {
        id: track_id,
        name: "V1".to_string(),
        kind: TrackKind::Video,
        mute: false,
        solo: false,
        locked: false,
        visible: true,
        audio_props: None,
        clips: vec![clip],
    };

    let seq_id = Uuid::new_v4();
    let sequence = Sequence {
        id: seq_id,
        name: "Main Timeline".to_string(),
        resolution: CanvasSize::P1080_16_9,
        timebase: 60,
        fps_denominator: 1,
        playhead: RationalTime::new(0, 60),
        work_area: None,
        tracks: vec![track],
        master_bus: Default::default(),
    };

    let project = Project {
        id: Uuid::new_v4(),
        name: "My ClipFlow Project".to_string(),
        asset_pool,
        sequences: vec![sequence],
        active_sequence_id: seq_id,
        agent_session: None,
    };

    assert_eq!(project.sequences.len(), 1);
    let active_seq = project.active_sequence().expect("Active sequence should exist");
    assert_eq!(active_seq.id, seq_id);
    let found_clip = active_seq.find_clip(clip_id);
    assert!(found_clip.is_some());
    assert_eq!(found_clip.unwrap().id, clip_id);
}

#[test]
fn test_clip_source_to_timeline_rational_mapping() {
    let clip = Clip {
        id: Uuid::new_v4(),
        name: "Speed Ramp Clip".to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(RationalTime::new(100, 60), RationalTime::new(300, 60)),
        timeline_range: TimeRange::new(RationalTime::new(200, 60), RationalTime::new(200, 60)), // 300 / 1.5 = 200
        speed: 1.5,
        transform: Transform2D::default(),
        audio_props: AudioProperties::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    };

    // timeline_range.start (200) -> source_range.start (100)
    let src_at_start = clip.timeline_to_source_time(RationalTime::new(200, 60));
    assert_eq!(src_at_start, RationalTime::new(100, 60));

    // timeline_range.start + 100 (300) -> source_range.start + 100 * 1.5 = 100 + 150 = 250
    let src_at_mid = clip.timeline_to_source_time(RationalTime::new(300, 60));
    assert_eq!(src_at_mid, RationalTime::new(250, 60));

    // 倒放测试
    let mut reverse_clip = clip.clone();
    reverse_clip.speed = -1.0;
    reverse_clip.timeline_range = TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60));
    // t_source = source_range.end - (t_timeline - start) * |speed|
    // end = 100 + 300 = 400. At timeline 0 -> 400.
    let rev_start = reverse_clip.timeline_to_source_time(RationalTime::new(0, 60));
    assert_eq!(rev_start, RationalTime::new(400, 60));
}

#[test]
fn test_keyframe_bezier_and_linear_evaluation() {
    let static_anim: Animatable<f32> = Animatable::Static(42.0);
    assert_eq!(static_anim.evaluate_at(RationalTime::new(10, 60)), 42.0);

    let kf1 = Keyframe {
        time: RationalTime::new(0, 60),
        value: 0.0f32,
        interpolation: Interpolation::Linear,
    };
    let kf2 = Keyframe {
        time: RationalTime::new(60, 60),
        value: 100.0f32,
        interpolation: Interpolation::Linear,
    };
    let animated: Animatable<f32> = Animatable::Animated(vec![kf1, kf2]);

    assert_eq!(animated.evaluate_at(RationalTime::new(0, 60)), 0.0);
    assert_eq!(animated.evaluate_at(RationalTime::new(30, 60)), 50.0);
    assert_eq!(animated.evaluate_at(RationalTime::new(60, 60)), 100.0);
    // 超出边界箝位
    assert_eq!(animated.evaluate_at(RationalTime::new(120, 60)), 100.0);
    assert_eq!(animated.evaluate_at(RationalTime::new(-30, 60)), 0.0);
}

#[test]
fn test_track_audio_properties_defaults() {
    let props = TrackAudioProperties::default();
    assert_eq!(props.fader_volume_db, 0.0);
    assert_eq!(props.pan, 0.0);
    assert_eq!(props.ai_denoise_amount, 0.0);
    assert_eq!(props.eq_bands.len(), 4);
    assert_eq!(props.eq_bands[0].freq_hz, 80.0);
    assert!(props.eq_bands[0].enabled);
    assert!(!props.eq_bands[1].enabled);
}

#[test]
fn test_project_json_roundtrip_serialization() {
    let mut asset_pool = AssetPool::new();
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "test.mp4".to_string(),
        absolute_path: PathBuf::from(r"C:\Media\test.mp4"),
        relative_path: None,
        file_size: 1024,
        sha256_hash: "hash123".to_string(),
        kind: AssetKind::Video {
            width: 1920,
            height: 1080,
            has_audio: false,
        },
        native_duration: RationalTime::new(100, 30),
        native_timebase: 30,
        proxy_path: None,
    };
    asset_pool.insert(asset);

    let seq_id = Uuid::new_v4();
    let sequence = Sequence {
        id: seq_id,
        name: "Sequence 1".to_string(),
        resolution: CanvasSize::P1080_16_9,
        timebase: 30,
        fps_denominator: 1,
        playhead: RationalTime::new(15, 30),
        work_area: None,
        tracks: Vec::new(),
        master_bus: Default::default(),
    };

    let project = Project {
        id: Uuid::new_v4(),
        name: "Serialized Project".to_string(),
        asset_pool,
        sequences: vec![sequence],
        active_sequence_id: seq_id,
        agent_session: None,
    };

    let json_str = serde_json::to_string_pretty(&project).expect("Serialization failed");
    let deserialized: Project = serde_json::from_str(&json_str).expect("Deserialization failed");

    assert_eq!(deserialized.id, project.id);
    assert_eq!(deserialized.name, project.name);
    assert_eq!(deserialized.sequences.len(), 1);
    assert_eq!(deserialized.sequences[0].playhead, RationalTime::new(15, 30));
}
