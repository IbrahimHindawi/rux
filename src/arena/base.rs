use std::mem::{align_of, needs_drop, size_of, MaybeUninit};
use std::ptr::{self, NonNull};

#[cfg(not(windows))]
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

use super::raw::{align_up, array_layout, ARENA_BASE_ALIGN};
#[cfg(windows)]
use super::raw::{commit, page_size, release, reserve};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    cursor: usize,
}

pub struct Arena {
    ptr: NonNull<u8>,
    cap: usize,
    cursor: usize,
    #[cfg(windows)]
    page_size: usize,
    #[cfg(windows)]
    committed: usize,
    #[cfg(not(windows))]
    layout: Layout,
}

impl Arena {
    pub fn new(reserve_bytes: usize) -> Self {
        let cap = reserve_bytes.max(1);

        #[cfg(windows)]
        {
            let page_size = page_size();
            let reserve_size = align_up(cap, ARENA_BASE_ALIGN);
            let raw = reserve(reserve_size);
            let ptr = NonNull::new(raw).expect("VirtualAlloc reserve failed");

            return Self {
                ptr,
                cap: reserve_size,
                cursor: 0,
                page_size,
                committed: 0,
            };
        }

        #[cfg(not(windows))]
        {
            let layout =
                Layout::from_size_align(cap, ARENA_BASE_ALIGN).expect("invalid arena layout");
            let raw = unsafe { alloc(layout) };
            let ptr = NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout));

            Self {
                ptr,
                cap,
                cursor: 0,
                layout,
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn used(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.cap - self.cursor
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            cursor: self.cursor,
        }
    }

    pub fn rewind(&mut self, checkpoint: Checkpoint) {
        assert!(
            checkpoint.cursor <= self.cap,
            "checkpoint cursor is out of range"
        );
        self.cursor = checkpoint.cursor;
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn scope(&mut self) -> super::scope::ArenaScope<'_> {
        super::scope::ArenaScope::new(self)
    }

    pub fn temp(&mut self) -> super::temp::TempArena<'_> {
        super::temp::TempArena::new(self)
    }

    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        assert_arena_supported::<T>();
        let ptr = self.alloc_uninit::<T>().as_ptr();
        unsafe {
            ptr.write(value);
            &mut *ptr
        }
    }

    pub fn alloc_slice_copy<T: Copy>(&mut self, slice: &[T]) -> &mut [T] {
        assert_arena_supported::<T>();
        let dst = self.alloc_array_uninit::<T>(slice.len());
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), dst.as_mut_ptr() as *mut T, slice.len());
            std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut T, slice.len())
        }
    }

    pub fn alloc_array_uninit<T>(&mut self, len: usize) -> &mut [MaybeUninit<T>] {
        assert_arena_supported::<T>();
        if len == 0 {
            return &mut [];
        }

        let layout = array_layout::<T>(len);
        let ptr = self.alloc_layout(layout);
        unsafe { std::slice::from_raw_parts_mut(ptr.cast::<MaybeUninit<T>>(), len) }
    }

    pub(crate) fn alloc_raw_array<T>(&mut self, len: usize) -> NonNull<T> {
        assert_arena_supported::<T>();
        if len == 0 || size_of::<T>() == 0 {
            return NonNull::dangling();
        }

        let layout = array_layout::<T>(len);
        let ptr = self.alloc_layout(layout);
        unsafe { NonNull::new_unchecked(ptr.cast::<T>()) }
    }

    fn alloc_uninit<T>(&mut self) -> NonNull<T> {
        assert_arena_supported::<T>();
        if size_of::<T>() == 0 {
            return NonNull::dangling();
        }

        let layout = std::alloc::Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        unsafe { NonNull::new_unchecked(ptr.cast::<T>()) }
    }

    fn alloc_layout(&mut self, layout: std::alloc::Layout) -> *mut u8 {
        assert!(
            layout.align() <= ARENA_BASE_ALIGN.max(align_of::<usize>()),
            "requested alignment {} exceeds arena base alignment {}",
            layout.align(),
            ARENA_BASE_ALIGN
        );

        if layout.size() == 0 {
            return NonNull::<u8>::dangling().as_ptr();
        }

        let start = align_up(self.cursor, layout.align());
        let end = start
            .checked_add(layout.size())
            .expect("arena allocation overflowed");

        #[cfg(windows)]
        self.ensure_committed(end);

        assert!(
            end <= self.cap,
            "arena exhausted: requested {} bytes with {} remaining",
            layout.size(),
            self.cap.saturating_sub(self.cursor)
        );

        self.cursor = end;

        unsafe { self.ptr.as_ptr().add(start) }
    }

    #[cfg(windows)]
    fn ensure_committed(&mut self, end: usize) {
        assert!(
            end <= self.cap,
            "arena exhausted: requested {} bytes beyond reserved capacity {}",
            end,
            self.cap
        );

        if end <= self.committed {
            return;
        }

        let new_committed = align_up(end, self.page_size);
        let commit_size = new_committed - self.committed;
        let result = commit(self.ptr.as_ptr(), self.committed, commit_size);
        assert!(
            !result.is_null(),
            "VirtualAlloc commit failed for {} bytes",
            commit_size
        );
        self.committed = new_committed;
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            release(self.ptr.as_ptr());
        }

        #[cfg(not(windows))]
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[inline]
fn assert_arena_supported<T>() {
    assert!(
        !needs_drop::<T>(),
        "rux::Arena does not support droppable types"
    );
}
