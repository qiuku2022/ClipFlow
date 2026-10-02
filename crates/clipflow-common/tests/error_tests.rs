use clipflow_common::{ClipFlowError, Result};

#[test]
fn test_clipflow_error_formatting() {
    let err_ipc = ClipFlowError::Ipc("管道连接断开".to_string());
    assert_eq!(err_ipc.to_string(), "IPC 错误: 管道连接断开");

    let err_timeline = ClipFlowError::Timeline("片段区间冲突".to_string());
    assert_eq!(err_timeline.to_string(), "时间轴错误: 片段区间冲突");

    let err_invalid_time = ClipFlowError::InvalidTime("分母为0".to_string());
    assert_eq!(err_invalid_time.to_string(), "无效时间参数: 分母为0");

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
    let err_from_io: ClipFlowError = io_err.into();
    assert!(err_from_io.to_string().contains("I/O 错误"));
}

fn sample_failing_function() -> Result<()> {
    Err(ClipFlowError::Media("解码器初始化失败".to_string()))
}

#[test]
fn test_result_type_alias() {
    let res = sample_failing_function();
    assert!(res.is_err());
    match res {
        Err(ClipFlowError::Media(msg)) => assert_eq!(msg, "解码器初始化失败"),
        _ => panic!("预期 Media 错误"),
    }
}
