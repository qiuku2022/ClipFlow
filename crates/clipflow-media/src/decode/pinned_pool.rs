use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PinnedSlot {
    pool: Arc<PinnedFramePoolInner>,
    slot_id: usize,
    ptr: *mut u8,
    size: usize,
}

// SAFETY: 槽位由独占持有的 PinnedSlot 封装，且槽位在物理上相互隔离
unsafe impl Send for PinnedSlot {}
unsafe impl Sync for PinnedSlot {}

impl PinnedSlot {
    pub fn slot_id(&self) -> usize {
        self.slot_id
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr 经过 VirtualAlloc 分配，size 范围内有效可读
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: ptr 独占且 VirtualAlloc 设置了 PAGE_READWRITE
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for PinnedSlot {
    fn drop(&mut self) {
        self.pool.release_slot(self.slot_id);
    }
}

struct PinnedFramePoolInner {
    base_ptr: *mut u8,
    total_size: usize,
    slot_size: usize,
    num_slots: usize,
    in_use: Vec<AtomicBool>,
}

// SAFETY: PinnedFramePoolInner 管理已分配内存并由原子布尔数组维护槽位占用
unsafe impl Send for PinnedFramePoolInner {}
unsafe impl Sync for PinnedFramePoolInner {}

impl PinnedFramePoolInner {
    fn release_slot(&self, slot_id: usize) {
        if slot_id < self.num_slots {
            self.in_use[slot_id].store(false, Ordering::Release);
        }
    }
}

impl Drop for PinnedFramePoolInner {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Memory::{VirtualFree, VirtualUnlock, MEM_RELEASE};
            // SAFETY: 释放 VirtualLock 钉住的内存并释放虚拟分配
            unsafe {
                let _ = VirtualUnlock(self.base_ptr as *mut _, self.total_size);
                VirtualFree(self.base_ptr as *mut _, 0, MEM_RELEASE);
            }
        }
        #[cfg(not(windows))]
        {
            // 非 Windows 兜底
            let layout = std::alloc::Layout::from_size_align(self.total_size, 4096).unwrap();
            unsafe {
                std::alloc::dealloc(self.base_ptr, layout);
            }
        }
    }
}

#[derive(Clone)]
pub struct PinnedFramePool {
    inner: Arc<PinnedFramePoolInner>,
}

impl PinnedFramePool {
    pub fn new(num_slots: usize, slot_size: usize) -> std::io::Result<Self> {
        assert!(num_slots > 0, "num_slots must be positive");
        assert!(slot_size > 0, "slot_size must be positive");

        // 4KB 对齐
        let aligned_slot_size = (slot_size + 4095) & !4095;
        let total_size = num_slots * aligned_slot_size;

        let base_ptr: *mut u8;

        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Memory::{
                VirtualAlloc, VirtualLock, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE,
            };
            // SAFETY: 请求系统以 PAGE_READWRITE 分配指定大小的页对齐内存
            base_ptr = unsafe {
                VirtualAlloc(
                    std::ptr::null_mut(),
                    total_size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                ) as *mut u8
            };

            if base_ptr.is_null() {
                return Err(std::io::Error::last_os_error());
            }

            // SAFETY: 尝试将分配的内存钉入物理内存 (VirtualLock)
            unsafe {
                let _ = VirtualLock(base_ptr as *mut _, total_size);
            }
        }

        #[cfg(not(windows))]
        {
            let layout = std::alloc::Layout::from_size_align(total_size, 4096).unwrap();
            base_ptr = unsafe { std::alloc::alloc(layout) };
            if base_ptr.is_null() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::OutOfMemory,
                    "Allocation failed",
                ));
            }
        }

        let in_use = (0..num_slots).map(|_| AtomicBool::new(false)).collect();

        Ok(Self {
            inner: Arc::new(PinnedFramePoolInner {
                base_ptr,
                total_size,
                slot_size: aligned_slot_size,
                num_slots,
                in_use,
            }),
        })
    }

    pub fn capacity(&self) -> usize {
        self.inner.num_slots
    }

    pub fn available_slots(&self) -> usize {
        self.inner
            .in_use
            .iter()
            .filter(|b| !b.load(Ordering::Acquire))
            .count()
    }

    pub fn acquire_slot(&self) -> Option<PinnedSlot> {
        for (i, slot_flag) in self.inner.in_use.iter().enumerate() {
            if slot_flag
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                let slot_ptr = unsafe { self.inner.base_ptr.add(i * self.inner.slot_size) };
                return Some(PinnedSlot {
                    pool: self.inner.clone(),
                    slot_id: i,
                    ptr: slot_ptr,
                    size: self.inner.slot_size,
                });
            }
        }
        None
    }
}
