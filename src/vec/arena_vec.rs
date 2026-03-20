use std::marker::PhantomData;
use std::mem::{needs_drop, MaybeUninit};
use std::ops::{Index, IndexMut};
use std::ptr::{self, NonNull};
use std::slice;

use crate::arena::{Arena, ArenaScope};

pub struct ArenaVec<'a, T> {
    arena: NonNull<Arena>,
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> ArenaVec<'a, T> {
    pub fn new_in(scope: &'a ArenaScope<'a>) -> Self {
        assert_vec_supported::<T>();
        Self {
            arena: scope_arena(scope),
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    pub fn with_capacity_in(cap: usize, scope: &'a ArenaScope<'a>) -> Self {
        assert_vec_supported::<T>();
        let ptr = scope.alloc_raw_array::<T>(cap);
        Self {
            arena: scope_arena(scope),
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
        assert_vec_supported::<T>();
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            self.ptr.as_ptr().add(self.len).write(value);
        }
        self.len += 1;
    }

    pub fn clear(&mut self) {
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

        let new_ptr = unsafe {
            self.arena
                .as_ptr()
                .as_mut()
                .expect("arena pointer was null")
        }
        .alloc_raw_array::<T>(new_cap);
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

#[inline]
fn assert_vec_supported<T>() {
    assert!(
        !needs_drop::<T>(),
        "rux::ArenaVec does not support droppable element types"
    );
}

#[inline]
fn scope_arena(scope: &ArenaScope<'_>) -> NonNull<Arena> {
    scope.arena_ptr()
}
