use super::{Arena, Checkpoint};

pub struct TempArena<'a> {
    arena: &'a Arena,
    checkpoint: Checkpoint,
    active: bool,
}

impl<'a> TempArena<'a> {
    pub(crate) fn new(arena: &'a Arena) -> Self {
        Self {
            arena,
            checkpoint: arena.checkpoint(),
            active: true,
        }
    }

    pub fn checkpoint(&self) -> Checkpoint {
        self.checkpoint
    }

    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.arena.alloc(value)
    }

    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        self.arena.alloc_slice_copy(slice)
    }

    pub fn alloc_array_uninit<T>(&self, len: usize) -> &mut [std::mem::MaybeUninit<T>] {
        self.arena.alloc_array_uninit(len)
    }

    pub fn used(&self) -> usize {
        self.arena.used()
    }

    pub fn commit(mut self) {
        self.active = false;
    }
}

impl Drop for TempArena<'_> {
    fn drop(&mut self) {
        if self.active {
            self.arena.rewind(self.checkpoint);
        }
    }
}
