use clipflow_common::{RationalTime, TimeRange};
use clipflow_timeline::models::{CanvasSize, Clip, ClipPayload, Sequence, Track, TrackKind};
use clipflow_ui::timeline::{
    SubtitleGalleyCache, TimelineViewport, TwoDimensionalCuller,
};
use std::time::Instant;
use uuid::Uuid;

fn make_large_test_sequence(track_count: usize, clips_per_track: usize) -> Sequence {
    let mut seq = Sequence::new("Heavy Timeline", CanvasSize::P1080_16_9, 60, 1);

    for t_idx in 0..track_count {
        let kind = if t_idx % 2 == 0 {
            TrackKind::Video
        } else {
            TrackKind::Audio
        };
        let mut track = Track::new(format!("Track {}", t_idx), kind);

        for c_idx in 0..clips_per_track {
            let start_sec = (c_idx * 5) as i64;
            let clip = Clip {
                id: Uuid::new_v4(),
                name: format!("T{} C{}", t_idx, c_idx),
                asset_id: Uuid::new_v4(),
                source_range: TimeRange::new(RationalTime::new(0, 60), RationalTime::new(300, 60)),
                timeline_range: TimeRange::new(
                    RationalTime::new(start_sec * 60, 60),
                    RationalTime::new(300, 60), // 5s = 300 ticks
                ),
                speed: 1.0,
                transform: Default::default(),
                audio_props: Default::default(),
                filters: Vec::new(),
                payload: ClipPayload::Media,
                disabled: false,
            };
            track.clips.push(clip);
        }
        seq.tracks.push(track);
    }

    seq
}

#[test]
fn test_2d_orthogonal_culling_accuracy() {
    // 20 条轨道，每轨 100 个切片，总计 2000 个切片
    let seq = make_large_test_sequence(20, 100);

    // 视口设置：垂直看第 2~4 条轨道 (Y: 100.0 ~ 300.0)，水平看第 10s ~ 25s
    let viewport = TimelineViewport {
        visible_time_start: RationalTime::new(10 * 60, 60),
        visible_time_end: RationalTime::new(25 * 60, 60),
        scroll_y: 100.0,
        viewport_height: 200.0,
        pixels_per_second: 50.0,
    };

    let track_heights = vec![64.0; 20];
    let culler = TwoDimensionalCuller::new(&track_heights);

    let visible_clips = culler.cull(&seq, &viewport);
    assert!(!visible_clips.is_empty());

    // 验证所有筛选出的切片都在视口时间范围内相交
    for (t_idx, clip) in &visible_clips {
        assert!(*t_idx <= 6); // 加上 64px 垂直缓冲后的可见轨道范围
        assert!(clip.timeline_range.end_exclusive() >= viewport.visible_time_start);
        assert!(clip.timeline_range.start <= viewport.visible_time_end);
    }
}

#[test]
fn test_viewport_culling_performance_benchmark() {
    let seq = make_large_test_sequence(20, 100);
    let viewport = TimelineViewport {
        visible_time_start: RationalTime::new(50 * 60, 60),
        visible_time_end: RationalTime::new(80 * 60, 60),
        scroll_y: 150.0,
        viewport_height: 300.0,
        pixels_per_second: 50.0,
    };

    let track_heights = vec![64.0; 20];
    let culler = TwoDimensionalCuller::new(&track_heights);

    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = culler.cull(&seq, &viewport);
    }
    let elapsed = start.elapsed();
    let avg_per_cull = elapsed / iterations;

    // 断言单次裁剪筛选耗时 <= 0.05ms (50微秒)
    assert!(
        avg_per_cull.as_micros() <= 50,
        "Culling took too long: {:?} per cull",
        avg_per_cull
    );
}

#[test]
fn test_subtitle_galley_cache_hit_rate() {
    let mut cache = SubtitleGalleyCache::new(1024);
    let text = "1973年加油站排起长队";

    // 第一次未命中
    assert!(!cache.contains(text, 14.0));
    cache.insert(text, 14.0, (120.0, 24.0)); // 缓存度量尺寸

    // 后续高频命中
    assert!(cache.contains(text, 14.0));
    let hit_dims = cache.get(text, 14.0);
    assert_eq!(hit_dims, Some((120.0, 24.0)));
}
