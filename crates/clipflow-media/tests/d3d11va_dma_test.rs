use clipflow_common::RationalTime;
use clipflow_media::decode::{
    BackpressureController, MockVideoDecoder, PinnedFramePool, VideoTextureProvider,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_pinned_frame_pool_allocation_and_lock() {
    // 预分配 8 个槽位，每个槽位 1MB
    let pool = PinnedFramePool::new(8, 1024 * 1024).expect("PinnedFramePool creation failed");
    assert_eq!(pool.capacity(), 8);
    assert_eq!(pool.available_slots(), 8);

    let mut slot0 = pool.acquire_slot().expect("Acquiring slot 0 failed");
    assert_eq!(slot0.slot_id(), 0);
    assert_eq!(pool.available_slots(), 7);

    // 写入数据验证物理可写
    let slice = slot0.as_mut_slice();
    slice[0] = 0xAB;
    slice[1024] = 0xCD;
    assert_eq!(slice[0], 0xAB);
    assert_eq!(slice[1024], 0xCD);

    drop(slot0);
    assert_eq!(pool.available_slots(), 8);
}

#[test]
fn test_backpressure_flow_control() {
    let controller = Arc::new(BackpressureController::new(3));
    let producer_blocked = Arc::new(AtomicBool::new(false));

    // 生产 3 帧填充配额
    assert!(controller.try_push());
    assert!(controller.try_push());
    assert!(controller.try_push());
    assert_eq!(controller.current_depth(), 3);

    // 第 4 帧不可直接推送
    assert!(!controller.try_push());

    let c_clone = controller.clone();
    let pb_clone = producer_blocked.clone();
    let handle = thread::spawn(move || {
        pb_clone.store(true, Ordering::SeqCst);
        c_clone.wait_and_push(); // 阻塞等待空闲槽位
        pb_clone.store(false, Ordering::SeqCst);
    });

    thread::sleep(Duration::from_millis(50));
    assert!(producer_blocked.load(Ordering::SeqCst));

    // 消费者消费 1 帧，唤醒生产者
    controller.pop();
    handle.join().unwrap();
    assert!(!producer_blocked.load(Ordering::SeqCst));
    assert_eq!(controller.current_depth(), 3);
}

#[test]
fn test_video_texture_provider_contract() {
    let mut decoder = MockVideoDecoder::new(1920, 1080, 60);
    assert_eq!(decoder.width(), 1920);
    assert_eq!(decoder.height(), 1080);

    // 提取第 1 帧
    let frame1 = decoder.poll_next_frame().expect("Frame 1 expected");
    assert_eq!(frame1.pts, RationalTime::new(0, 60));
    assert_eq!(frame1.y_plane.len(), 1920 * 1080);
    assert_eq!(frame1.uv_plane.len(), 1920 * 1080 / 2);

    // 提取第 2 帧
    let frame2 = decoder.poll_next_frame().expect("Frame 2 expected");
    assert_eq!(frame2.pts, RationalTime::new(1, 60));

    // Seek 测试
    decoder.seek(RationalTime::new(30, 60)).expect("Seek failed");
    let frame_seek = decoder.poll_next_frame().expect("Frame after seek expected");
    assert_eq!(frame_seek.pts, RationalTime::new(30, 60));
}
