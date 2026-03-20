use std::ops::{Deref, DerefMut};

use super::{Arena, ArenaScope, Checkpoint};

pub struct TempArena<'a> {
    scope: ArenaScope<'a>,
    checkpoint: Checkpoint,
    active: bool,
}

impl<'a> TempArena<'a> {
    pub(crate) fn new(arena: &'a mut Arena) -> Self {
        let checkpoint = arena.checkpoint();
        Self {
            scope: ArenaScope::new(arena),
            checkpoint,
            active: true,
        }
    }

    pub fn checkpoint(&self) -> Checkpoint {
        self.checkpoint
    }

    pub fn commit(mut self) {
        self.active = false;
    }
}

impl<'a> Deref for TempArena<'a> {
    type Target = ArenaScope<'a>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl<'a> DerefMut for TempArena<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

impl Drop for TempArena<'_> {
    fn drop(&mut self) {
        if self.active {
            self.scope.rewind(self.checkpoint);
        }
    }
}
