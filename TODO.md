# TODO

## Current Milestone

- [x] Scaffold the initial Rust crate layout.
- [x] Implement a low-level `Arena` with bump allocation, alignment, checkpoints, rewind, and clear.
- [x] Implement `TempArena` as an RAII rollback guard over arena checkpoints.
- [x] Implement `ArenaVec<T>` as an arena-backed growable contiguous buffer.
- [x] Add tests for allocation, alignment, rewind, destructor behavior, zero-sized types, and `ArenaVec<T>` growth.

## Next

- [ ] Decide whether `Arena` should stay fixed-capacity or grow by chaining chunks.
- [ ] Add fallible allocation APIs alongside the panic-on-exhaustion convenience methods.
- [ ] Add slice/string helpers for common arena use cases.
- [ ] Decide whether `ArenaVec<T>` should support `insert`, `remove`, and `swap_remove`.
- [ ] Add a separate handle-based storage type instead of overloading `ArenaVec<T>` with stable-identity semantics.
- [ ] Design a `SlotArena<T>` or similar pool for engine-style entity/component workloads.
- [ ] Add benchmarks against `Vec<T>` and other arena crates.
- [ ] Expand the README into an actual design document once the APIs settle.
