use std::io;
use std::ptr;
use std::mem;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};

pub struct JobGuard {
    handle: HANDLE,
}

impl JobGuard {
    pub fn new() -> io::Result<Self> {
        unsafe {
            // 创建匿名的 Job Object
            let handle = CreateJobObjectW(ptr::null(), ptr::null());
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            // 配置 Job Object 的限制属性
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let result = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            if result == 0 {
                let err = io::Error::last_os_error();
                CloseHandle(handle);
                return Err(err);
            }

            Ok(Self { handle })
        }
    }

    /// 将指定的进程 ID 绑定到此 Job Object。
    /// 一旦绑定，如果该 JobGuard 被释放（或者当前主进程异常退出导致 Job 句柄关闭），
    /// 操作系统将强制杀死绑定的子进程。
    pub fn assign_process(&self, pid: u32) -> io::Result<()> {
        unsafe {
            // 获取子进程句柄 (需要 PROCESS_SET_QUOTA 和 PROCESS_TERMINATE 权限)
            let process_handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
            if process_handle.is_null() || process_handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            let result = AssignProcessToJobObject(self.handle, process_handle);
            let err = if result == 0 {
                Some(io::Error::last_os_error())
            } else {
                None
            };

            CloseHandle(process_handle);

            if let Some(e) = err {
                return Err(e);
            }
            Ok(())
        }
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
        }
    }
}

