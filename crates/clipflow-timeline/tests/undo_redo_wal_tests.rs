use clipflow_common::{RationalTime, TimeRange};
use clipflow_timeline::commands::{
    CompoundCommand, DeleteClipCommand, RippleDeleteCommand, SplitClipCommand, TimelineCommand,
};
use clipflow_timeline::history::TimelineHistory;
use clipflow_timeline::models::{CanvasSize, Clip, ClipPayload, Sequence, Track, TrackKind};
use clipflow_timeline::wal::TimelineWal;
use uuid::Uuid;

fn make_test_sequence() -> (Sequence, Uuid, Uuid, Uuid, Uuid) {
    let mut seq = Sequence::new("Test Seq", CanvasSize::P1080_16_9, 60, 1);
    let mut v1 = Track::new("V1", TrackKind::Video);
    let mut a2 = Track::new("A2 BGM", TrackKind::Audio);

    let clip1_id = Uuid::new_v4();
    let clip1 = Clip {
        id: clip1_id,
        name: "Clip 1".to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
        timeline_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
        speed: 1.0,
        transform: Default::default(),
        audio_props: Default::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    };

    let clip2_id = Uuid::new_v4();
    let clip2 = Clip {
        id: clip2_id,
        name: "Clip 2".to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
        timeline_range: TimeRange::new(RationalTime::new(300, 60), RationalTime::new(300, 60)),
        speed: 1.0,
        transform: Default::default(),
        audio_props: Default::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    };

    let bgm_clip_id = Uuid::new_v4();
    let bgm_clip = Clip {
        id: bgm_clip_id,
        name: "BGM Track".to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(1000, 60)),
        timeline_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(1000, 60)),
        speed: 1.0,
        transform: Default::default(),
        audio_props: Default::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    };

    let v1_id = v1.id;
    let a2_id = a2.id;
    v1.clips.push(clip1);
    v1.clips.push(clip2);
    a2.clips.push(bgm_clip);

    seq.tracks.push(v1);
    seq.tracks.push(a2);

    (seq, v1_id, a2_id, clip1_id, clip2_id)
}

#[test]
fn test_split_clip_command_execute_and_undo() {
    let (mut seq, v1_id, _, clip1_id, _) = make_test_sequence();
    let cut_point = RationalTime::new(100, 60);

    let mut split_cmd = SplitClipCommand::new(clip1_id, cut_point);
    assert!(split_cmd.execute(&mut seq).is_ok());

    let v1 = seq.find_track(v1_id).unwrap();
    assert_eq!(v1.clips.len(), 3); // 原 clip1 + split出来的后半段 + 原 clip2
    let c1 = v1.find_clip(clip1_id).unwrap();
    assert_eq!(c1.timeline_range.duration, RationalTime::new(100, 60));

    // 后半段
    let new_clip_id = split_cmd.created_clip_id().expect("New clip should be generated");
    let c_new = v1.find_clip(new_clip_id).unwrap();
    assert_eq!(c_new.timeline_range.start, RationalTime::new(100, 60));
    assert_eq!(c_new.timeline_range.duration, RationalTime::new(200, 60));

    // Undo 回退
    assert!(split_cmd.undo(&mut seq).is_ok());
    let v1_after = seq.find_track(v1_id).unwrap();
    assert_eq!(v1_after.clips.len(), 2);
    let c1_restored = v1_after.find_clip(clip1_id).unwrap();
    assert_eq!(c1_restored.timeline_range.duration, RationalTime::new(300, 60));
    assert!(v1_after.find_clip(new_clip_id).is_none());
}

#[test]
fn test_ripple_delete_command_with_multitrack_protection() {
    let (mut seq, v1_id, a2_id, clip1_id, clip2_id) = make_test_sequence();

    // 仅允许联动 v1，保护 a2 (BGM) 不被剪碎
    let mut ripple_cmd = RippleDeleteCommand::new(clip1_id, Some(vec![v1_id]));
    assert!(ripple_cmd.execute(&mut seq).is_ok());

    let v1 = seq.find_track(v1_id).unwrap();
    assert_eq!(v1.clips.len(), 1);
    let c2 = v1.find_clip(clip2_id).unwrap();
    // clip2 原先从 300 开始，向左平移 300 刻度，现从 0 开始
    assert_eq!(c2.timeline_range.start, RationalTime::new(0, 60));

    // a2 BGM 轨未受波纹删除波及
    let a2 = seq.find_track(a2_id).unwrap();
    assert_eq!(a2.clips[0].timeline_range.start, RationalTime::new(0, 60));
    assert_eq!(a2.clips[0].timeline_range.duration, RationalTime::new(1000, 60));

    // Undo 回退
    assert!(ripple_cmd.undo(&mut seq).is_ok());
    let v1_after = seq.find_track(v1_id).unwrap();
    assert_eq!(v1_after.clips.len(), 2);
    let c1 = v1_after.find_clip(clip1_id).unwrap();
    assert_eq!(c1.timeline_range.start, RationalTime::new(0, 60));
    let c2_restored = v1_after.find_clip(clip2_id).unwrap();
    assert_eq!(c2_restored.timeline_range.start, RationalTime::new(300, 60));
}

#[test]
fn test_compound_command_atomic_transaction() {
    let (mut seq, _, _, clip1_id, clip2_id) = make_test_sequence();

    let split1 = Box::new(SplitClipCommand::new(clip1_id, RationalTime::new(50, 60)));
    let split2 = Box::new(SplitClipCommand::new(clip2_id, RationalTime::new(350, 60)));
    let mut compound = CompoundCommand::new(vec![split1, split2], "批量分割口播气口");

    assert!(compound.execute(&mut seq).is_ok());
    // 应当增加了 2 个新片段，总共 4 个片段在 V1
    assert_eq!(seq.tracks[0].clips.len(), 4);

    // 单次 Undo 原子撤销全部
    assert!(compound.undo(&mut seq).is_ok());
    assert_eq!(seq.tracks[0].clips.len(), 2);
}

#[test]
fn test_timeline_history_depth_and_redo_clearing() {
    let (mut seq, v1_id, _, clip1_id, _) = make_test_sequence();
    let mut history = TimelineHistory::new(2); // 最大深度 2

    let cmd1 = Box::new(SplitClipCommand::new(clip1_id, RationalTime::new(100, 60)));
    assert!(history.execute(cmd1, &mut seq).is_ok());
    assert_eq!(history.undo_len(), 1);

    // 撤销
    assert!(history.undo(&mut seq).unwrap());
    assert_eq!(history.undo_len(), 0);
    assert_eq!(history.redo_len(), 1);

    // 重做
    assert!(history.redo(&mut seq).unwrap());
    assert_eq!(history.undo_len(), 1);
    assert_eq!(history.redo_len(), 0);

    // 执行新命令，重做栈被清空
    let v1 = seq.find_track(v1_id).unwrap();
    let clip_to_delete = v1.clips[0].id;
    let cmd2 = Box::new(DeleteClipCommand::new(clip_to_delete));
    assert!(history.execute(cmd2, &mut seq).is_ok());
    assert_eq!(history.redo_len(), 0);
}

#[test]
fn test_wal_log_append_and_recovery() {
    let mut wal = TimelineWal::new_in_memory();
    let clip_id = Uuid::new_v4();
    let cut = RationalTime::new(120, 60);
    wal.append_split(clip_id, cut).expect("WAL append failed");

    assert_eq!(wal.entry_count(), 1);
    let entries = wal.read_entries().expect("WAL read failed");
    assert_eq!(entries.len(), 1);
}
