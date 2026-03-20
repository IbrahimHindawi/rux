use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::ptr::{self, NonNull};
use std::slice;

use crate::arena::Arena;

pub struct ArenaVec<'a, T> {
    arena: &'a Arena,
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<'a, T> ArenaVec<'a, T> {
    pub fn new_in(arena: &'a Arena) -> Self {
        Self {
            arena,
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    pub fn with_capacity_in(cap: usize, arena: &'a Arena) -> Self {
        let ptr = arena.alloc_raw_array::<T>(cap);
        Self {
            arena,
            ptr,
            len: 0,
            cap,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            self.ptr.as_ptr().add(self.len).write(value);
        }
        self.len += 1;
    }

    pub fn clear(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len));
        }
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    fn grow(&mut self) {
        let new_cap = match self.cap {
            0 => 4,
            current => current
                .checked_mul(2)
                .expect("ArenaVec capacity overflowed"),
        };

        let new_ptr = self.arena.alloc_raw_array::<T>(new_cap);
        if self.len != 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len);
            }
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast::<MaybeUninit<T>>(),
                self.cap - self.len,
            )
        }
    }
}

impl<T> Drop for ArenaVec<'_, T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Index<usize> for ArenaVec<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T> IndexMut<usize> for ArenaVec<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ArenaVec<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

#[allow(dead_code)]
impl<T> ArenaVec<'_, T> {
    pub(crate) fn spare_capacity(&mut self) -> &mut [MaybeUninit<T>] {
        self.spare_capacity_mut()
    }
}
