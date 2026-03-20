use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::{align_of, needs_drop, size_of, MaybeUninit};
use std::ptr::{self, NonNull};

#[cfg(not(windows))]
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

use super::raw::{align_up, array_layout, ARENA_BASE_ALIGN};
#[cfg(windows)]
use super::raw::{commit, page_size, release, reserve};
use super::temp::TempArena;

struct ArenaInner {
    cursor: usize,
    drops: Vec<DropRecord>,
}

struct DropRecord {
    ptr: *mut u8,
    drop_fn: unsafe fn(*mut u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    cursor: usize,
    drops_len: usize,
}

pub struct Arena {
    ptr: NonNull<u8>,
    cap: usize,
    #[cfg(windows)]
    page_size: usize,
    #[cfg(windows)]
    committed: UnsafeCell<usize>,
    #[cfg(not(windows))]
    layout: Layout,
    inner: UnsafeCell<ArenaInner>,
    _not_sync: PhantomData<*mut ()>,
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
                page_size,
                committed: UnsafeCell::new(0),
                inner: UnsafeCell::new(ArenaInner {
                    cursor: 0,
                    drops: Vec::new(),
                }),
                _not_sync: PhantomData,
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
                layout,
                inner: UnsafeCell::new(ArenaInner {
                    cursor: 0,
                    drops: Vec::new(),
                }),
                _not_sync: PhantomData,
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn used(&self) -> usize {
        self.inner().cursor
    }

    pub fn remaining(&self) -> usize {
        self.cap - self.used()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        let inner = self.inner();
        Checkpoint {
            cursor: inner.cursor,
            drops_len: inner.drops.len(),
        }
    }

    pub fn rewind(&self, checkpoint: Checkpoint) {
        assert!(
            checkpoint.cursor <= self.cap,
            "checkpoint cursor is out of range"
        );

        let inner = self.inner_mut();
        assert!(
            checkpoint.drops_len <= inner.drops.len(),
            "checkpoint drop stack is out of range"
        );

        while inner.drops.len() > checkpoint.drops_len {
            let record = inner.drops.pop().expect("drop stack underflow");
            unsafe { (record.drop_fn)(record.ptr) };
        }

        inner.cursor = checkpoint.cursor;
    }

    pub fn clear(&self) {
        self.rewind(Checkpoint {
            cursor: 0,
            drops_len: 0,
        });
    }

    pub fn temp(&self) -> TempArena<'_> {
        TempArena::new(self)
    }

    pub fn alloc<T>(&self, value: T) -> &mut T {
        let ptr = self.alloc_uninit::<T>().as_ptr();
        unsafe {
            ptr.write(value);
        }
        self.register_drop::<T>(ptr);
        unsafe { &mut *ptr }
    }

    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        let dst = self.alloc_array_uninit::<T>(slice.len());
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), dst.as_mut_ptr() as *mut T, slice.len());
            std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut T, slice.len())
        }
    }

    pub fn alloc_array_uninit<T>(&self, len: usize) -> &mut [MaybeUninit<T>] {
        if len == 0 {
            return &mut [];
        }

        let layout = array_layout::<T>(len);
        let ptr = self.alloc_layout(layout);
        unsafe { std::slice::from_raw_parts_mut(ptr.cast::<MaybeUninit<T>>(), len) }
    }

    pub(crate) fn alloc_raw_array<T>(&self, len: usize) -> NonNull<T> {
        if len == 0 || size_of::<T>() == 0 {
            return NonNull::dangling();
        }

        let layout = array_layout::<T>(len);
        let ptr = self.alloc_layout(layout);
        unsafe { NonNull::new_unchecked(ptr.cast::<T>()) }
    }

    fn alloc_uninit<T>(&self) -> NonNull<T> {
        if size_of::<T>() == 0 {
            return NonNull::dangling();
        }

        let layout = std::alloc::Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        unsafe { NonNull::new_unchecked(ptr.cast::<T>()) }
    }

    fn alloc_layout(&self, layout: std::alloc::Layout) -> *mut u8 {
        assert!(
            layout.align() <= ARENA_BASE_ALIGN.max(align_of::<usize>()),
            "requested alignment {} exceeds arena base alignment {}",
            layout.align(),
            ARENA_BASE_ALIGN
        );

        if layout.size() == 0 {
            return NonNull::<u8>::dangling().as_ptr();
        }

        let inner = self.inner_mut();
        let start = align_up(inner.cursor, layout.align());
        let end = start
            .checked_add(layout.size())
            .expect("arena allocation overflowed");

        #[cfg(windows)]
        self.ensure_committed(end);

        assert!(
            end <= self.cap,
            "arena exhausted: requested {} bytes with {} remaining",
            layout.size(),
            self.cap.saturating_sub(inner.cursor)
        );

        inner.cursor = end;

        unsafe { self.ptr.as_ptr().add(start) }
    }

    #[cfg(windows)]
    fn ensure_committed(&self, end: usize) {
        assert!(
            end <= self.cap,
            "arena exhausted: requested {} bytes beyond reserved capacity {}",
            end,
            self.cap
        );

        let committed = self.committed_mut();
        if end <= *committed {
            return;
        }

        let new_committed = align_up(end, self.page_size);
        let commit_size = new_committed - *committed;
        let result = commit(self.ptr.as_ptr(), *committed, commit_size);
        assert!(
            !result.is_null(),
            "VirtualAlloc commit failed for {} bytes",
            commit_size
        );
        *committed = new_committed;
    }

    fn register_drop<T>(&self, ptr: *mut T) {
        if !needs_drop::<T>() || size_of::<T>() == 0 {
            return;
        }

        unsafe fn drop_value<T>(ptr: *mut u8) {
            ptr.cast::<T>().drop_in_place();
        }

        self.inner_mut().drops.push(DropRecord {
            ptr: ptr.cast::<u8>(),
            drop_fn: drop_value::<T>,
        });
    }

    fn inner(&self) -> &ArenaInner {
        unsafe { &*self.inner.get() }
    }

    fn inner_mut(&self) -> &mut ArenaInner {
        unsafe { &mut *self.inner.get() }
    }

    #[cfg(windows)]
    fn committed_mut(&self) -> &mut usize {
        unsafe { &mut *self.committed.get() }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.clear();

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
