#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::{Arena, ArenaVec, String8};

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
    fn temp_arena_rewinds_allocations() {
        let arena = Arena::new(1024);
        let checkpoint = arena.checkpoint();
        let used_before = arena.used();

        {
            let temp = arena.temp();
            let _ = temp.alloc([1_u32, 2, 3, 4]);
            let _ = temp.alloc(77_u32);
            assert!(arena.used() > used_before);
        }

        assert_eq!(arena.checkpoint(), checkpoint);
    }

    #[test]
    fn clear_rewinds_to_zero() {
        let arena = Arena::new(1024);
        let _ = arena.alloc([1_u8; 32]);
        let _ = arena.alloc([2_u8; 32]);

        arena.clear();

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
    fn arena_rejects_droppable_types() {
        let arena = Arena::new(1024);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = arena.alloc(String::from("not supported"));
        }));

        assert!(result.is_err());
    }

    #[test]
    fn arena_vec_rejects_droppable_types() {
        let arena = Arena::new(1024);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut vec = ArenaVec::new_in(&arena);
            vec.push(String::from("not supported"));
        }));

        assert!(result.is_err());
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

    #[test]
    fn string8_keeps_trailing_nul_and_excludes_it_from_length() {
        let arena = Arena::new(1024);
        let string = String8::from_str_in("gin", &arena);

        assert_eq!(string.len(), 3);
        assert_eq!(string.as_bytes(), b"gin");
        assert_eq!(string.as_bytes_with_nul(), b"gin\0");
    }

    #[test]
    fn string8_append_and_clear_work() {
        let arena = Arena::new(1024);
        let mut string = String8::new_in(&arena);

        string.append_str("he");
        string.append_bytes(b"llo");
        string.append_byte(b'!');

        assert_eq!(string.as_bytes(), b"hello!");
        assert_eq!(string.as_c_str().to_bytes_with_nul(), b"hello!\0");

        string.clear();

        assert_eq!(string.len(), 0);
        assert_eq!(string.as_bytes(), b"");
        assert_eq!(string.as_bytes_with_nul(), b"\0");
    }
}
