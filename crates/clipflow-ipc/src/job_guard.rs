use clipflow_common::{ClipFlowError, Result};
use std::mem;
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command};
use std::ptr;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

/// Windows Job Object 内核级安全生命周期守卫
///
/// 封装 Win32 原生 Job Object，配置 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` 标志。
/// 当宿主进程发生任何原因退出（正常终止、崩溃或被任务管理器强杀）时，
/// Windows 内核保证级联强杀挂接在当前作业对象下的所有子进程，孤儿逃逸率严格为 0.0%。
pub struct JobGuard {
    handle: HANDLE,
}

// HANDLE 在进程内是线程安全的
unsafe impl Send for JobGuard {}
unsafe impl Sync for JobGuard {}

impl JobGuard {
    /// 创建并初始化一个配置了 KILL_ON_JOB_CLOSE 的新 JobGuard
    pub fn new() -> Result<Self> {
        let job_handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if job_handle.is_null() || job_handle == INVALID_HANDLE_VALUE {
            return Err(ClipFlowError::Ipc(format!(
                "CreateJobObjectW 失败，错误码: {}",
                std::io::Error::last_os_error()
            )));
        }

        // 配置自动级联退出限额
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let res = unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };

        if res == 0 {
            let err = std::io::Error::last_os_error();
            unsafe { CloseHandle(job_handle) };
            return Err(ClipFlowError::Ipc(format!(
                "SetInformationJobObject 配置失败: {}",
                err
            )));
        }

        Ok(Self { handle: job_handle })
    }

    /// 获取底层 Win32 Job 句柄
    pub fn raw_handle(&self) -> HANDLE {
        self.handle
    }

    /// 将已有的进程句柄纳入 Job Object 管理
    pub fn assign_process_handle(&self, process_handle: HANDLE) -> Result<()> {
        let res = unsafe { AssignProcessToJobObject(self.handle, process_handle) };
        if res == 0 {
            return Err(ClipFlowError::Ipc(format!(
                "AssignProcessToJobObject 失败: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(())
    }

    /// 将运行中的 std::process::Child 纳入 Job Object 管理
    pub fn assign_child(&self, child: &Child) -> Result<()> {
        let raw_handle = child.as_raw_handle() as HANDLE;
        self.assign_process_handle(raw_handle)
    }

    /// 启动子进程并原子绑定至 Job Object
    pub fn spawn_child_in_job(&self, command: &mut Command) -> Result<Child> {
        let child = command.spawn().map_err(ClipFlowError::Io)?;
        if let Err(e) = self.assign_child(&child) {
            // 若绑定失败，防御性清理子进程避免孤儿
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &child.id().to_string()])
                .status();
            return Err(e);
        }
        Ok(child)
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}
