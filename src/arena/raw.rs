use std::alloc::Layout;

#[cfg(windows)]
pub(crate) const ARENA_BASE_ALIGN: usize = 65536;

#[cfg(not(windows))]
pub(crate) const ARENA_BASE_ALIGN: usize = 4096;

#[inline]
pub(crate) fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + (align - 1)) & !(align - 1)
}

#[inline]
pub(crate) fn array_layout<T>(len: usize) -> Layout {
    Layout::array::<T>(len).expect("array layout overflowed")
}

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;
    use std::mem;
    use std::ptr;

    pub(crate) const MEM_COMMIT: u32 = 0x0000_1000;
    pub(crate) const MEM_RESERVE: u32 = 0x0000_2000;
    pub(crate) const MEM_RELEASE: u32 = 0x0000_8000;
    pub(crate) const PAGE_NOACCESS: u32 = 0x01;
    pub(crate) const PAGE_READWRITE: u32 = 0x04;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SystemInfo {
        _arch: u32,
        page_size: u32,
        minimum_application_address: *mut c_void,
        maximum_application_address: *mut c_void,
        active_processor_mask: usize,
        number_of_processors: u32,
        processor_type: u32,
        allocation_granularity: u32,
        processor_level: u16,
        processor_revision: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemInfo(lp_system_info: *mut SystemInfo);
        fn VirtualAlloc(
            lp_address: *mut c_void,
            dw_size: usize,
            fl_allocation_type: u32,
            fl_protect: u32,
        ) -> *mut c_void;
        fn VirtualFree(lp_address: *mut c_void, dw_size: usize, dw_free_type: u32) -> i32;
    }

    pub(crate) fn page_size() -> usize {
        unsafe {
            let mut info = mem::zeroed::<SystemInfo>();
            GetSystemInfo(&mut info);
            info.page_size as usize
        }
    }

    pub(crate) fn reserve(size: usize) -> *mut u8 {
        unsafe { VirtualAlloc(ptr::null_mut(), size, MEM_RESERVE, PAGE_NOACCESS).cast::<u8>() }
    }

    pub(crate) fn commit(base: *mut u8, offset: usize, size: usize) -> *mut u8 {
        unsafe {
            VirtualAlloc(
                base.add(offset).cast::<c_void>(),
                size,
                MEM_COMMIT,
                PAGE_READWRITE,
            )
            .cast::<u8>()
        }
    }

    pub(crate) fn release(base: *mut u8) {
        unsafe {
            let _ = VirtualFree(base.cast::<c_void>(), 0, MEM_RELEASE);
        }
    }
}

#[cfg(windows)]
pub(crate) use win32::{commit, page_size, release, reserve};
