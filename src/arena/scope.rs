use std::marker::PhantomData;
use std::ptr::NonNull;

use super::{Arena, Checkpoint};

pub struct ArenaScope<'a> {
    arena: NonNull<Arena>,
    _marker: PhantomData<&'a mut Arena>,
}

impl<'a> ArenaScope<'a> {
    pub(crate) fn new(arena: &'a mut Arena) -> Self {
        Self {
            arena: NonNull::from(arena),
            _marker: PhantomData,
        }
    }

    pub fn capacity(&self) -> usize {
        self.arena().capacity()
    }

    pub fn used(&self) -> usize {
        self.arena().used()
    }

    pub fn remaining(&self) -> usize {
        self.arena().remaining()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        self.arena().checkpoint()
    }

    pub fn rewind(&mut self, checkpoint: Checkpoint) {
        self.arena_mut().rewind(checkpoint);
    }

    pub fn reset(&mut self) {
        self.arena_mut().reset();
    }

    pub(crate) fn alloc_raw_array<T>(&self, len: usize) -> NonNull<T> {
        self.arena_mut().alloc_raw_array::<T>(len)
    }

    pub(crate) fn arena_ptr(&self) -> NonNull<Arena> {
        self.arena
    }

    fn arena(&self) -> &Arena {
        unsafe { self.arena.as_ref() }
    }

    fn arena_mut(&self) -> &mut Arena {
        unsafe {
            self.arena
                .as_ptr()
                .as_mut()
                .expect("arena pointer was null")
        }
    }
}
