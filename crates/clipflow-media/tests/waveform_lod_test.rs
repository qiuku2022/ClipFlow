use clipflow_media::audio::{PeakFileHeader, PeakGenerator, PeakRecord, WaveformLodPyramid};

#[test]
fn test_peak_file_binary_header_and_records() {
    let generator = PeakGenerator::new(48000, 2, 256);

    // 构造测试正弦波样本 (48000 个采样点，即 1 秒音频)
    let mut samples = Vec::with_capacity(48000 * 2);
    for i in 0..48000 {
        let val = (i as f32 * 0.1).sin() * 20000.0;
        samples.push(val as i16);
        samples.push(val as i16);
    }

    let peak_bytes = generator.generate_from_i16_interleaved(&samples);
    assert!(peak_bytes.len() > 16);

    // 解析头部验证
    let header = PeakFileHeader::from_bytes(&peak_bytes[..16]).expect("Invalid header");
    assert_eq!(&header.magic, b"CFPK");
    assert_eq!(header.version, 1);
    assert_eq!(header.channels, 2);
    assert_eq!(header.sample_rate, 48000);
    assert_eq!(header.window_size, 256);

    // 记录数量：48000 / 256 ≈ 187 或 188
    let records = PeakRecord::read_records(&peak_bytes[16..]).expect("Read records failed");
    assert_eq!(records.len(), 48000 / 256);
    assert!(records[0].min_sample <= 0);
    assert!(records[0].max_sample >= 0);
}

#[test]
fn test_waveform_lod_pyramid_resolution_selection() {
    // 构造 1000 个 PeakRecord
    let mut records = Vec::with_capacity(1000);
    for i in 0..1000 {
        records.push(PeakRecord {
            min_sample: -((i % 100) * 200) as i16,
            max_sample: ((i % 100) * 200) as i16,
            rms_energy: 1000,
        });
    }

    let pyramid = WaveformLodPyramid::new(records, 48000, 256);

    // 1. 微观单帧级（高像素密度 200 px/s）-> LOD 0
    let points_lod0 = pyramid.query_range(0.0, 1.0, 200.0);
    assert!(!points_lod0.is_empty());

    // 2. 宏观全景级（低像素密度 2 px/s）-> 降采样 LOD
    let points_lod_macro = pyramid.query_range(0.0, 1.0, 2.0);
    assert!(points_lod_macro.len() < points_lod0.len());
}
