#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::{Arena, ArenaVec};

    #[derive(Debug)]
    struct DropCounter {
        drops: Rc<Cell<usize>>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    #[repr(align(64))]
    struct Align64(u8);

    #[test]
    fn arena_alloc_round_trip() {
        let arena = Arena::new(1024);
        let value = arena.alloc(123_u32);
        let pair = arena.alloc((7_u16, 9_u16));

        assert_eq!(*value, 123);
        assert_eq!(*pair, (7, 9));
        assert!(arena.used() >= std::mem::size_of::<u32>() + std::mem::size_of::<(u16, u16)>());
    }

    #[test]
    fn arena_honors_alignment() {
        let arena = Arena::new(1024);
        let value = arena.alloc(Align64(1));
        let ptr = value as *mut Align64 as usize;

        assert_eq!(ptr % 64, 0);
        assert_eq!(value.0, 1);
    }

    #[test]
    fn temp_arena_rewinds_and_drops_values() {
        let drops = Rc::new(Cell::new(0));
        let arena = Arena::new(1024);
        let checkpoint = arena.checkpoint();
        let used_before = arena.used();

        {
            let temp = arena.temp();
            let _ = temp.alloc(DropCounter {
                drops: Rc::clone(&drops),
            });
            let _ = temp.alloc(77_u32);
            assert!(arena.used() > used_before);
        }

        assert_eq!(drops.get(), 1);
        assert_eq!(arena.checkpoint(), checkpoint);
    }

    #[test]
    fn clear_drops_registered_values() {
        let drops = Rc::new(Cell::new(0));
        let arena = Arena::new(1024);
        let _ = arena.alloc(DropCounter {
            drops: Rc::clone(&drops),
        });
        let _ = arena.alloc(DropCounter {
            drops: Rc::clone(&drops),
        });

        arena.clear();

        assert_eq!(drops.get(), 2);
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn arena_vec_grows_and_preserves_contents() {
        let arena = Arena::new(4096);
        let mut vec = ArenaVec::with_capacity_in(1, &arena);

        for value in 0..16 {
            vec.push(value);
        }

        assert_eq!(vec.len(), 16);
        assert!(vec.capacity() >= 16);
        assert_eq!(vec.as_slice(), &(0..16).collect::<Vec<_>>()[..]);
    }

    #[test]
    fn arena_vec_drops_elements_on_drop() {
        let drops = Rc::new(Cell::new(0));
        let arena = Arena::new(4096);

        {
            let mut vec = ArenaVec::new_in(&arena);
            vec.push(DropCounter {
                drops: Rc::clone(&drops),
            });
            vec.push(DropCounter {
                drops: Rc::clone(&drops),
            });
        }

        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn zero_sized_allocations_do_not_consume_capacity() {
        let arena = Arena::new(128);
        let before = arena.used();

        let unit = arena.alloc(());
        let marker = arena.alloc([(); 8]);

        assert_eq!(*unit, ());
        assert_eq!(*marker, [(); 8]);
        assert_eq!(arena.used(), before);
    }
}
