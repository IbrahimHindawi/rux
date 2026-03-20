# rux

Rust custom standard library.

## Synopsis

An arena-based Rust standard library optimized for game engine development.

`extern/haikal` is the main C library inspiration:
- C monomorphizer core: `extern\haikal\src\meta.c`
- Arena allocator: `extern\haikal\src\meta_arena\gen\saha.h`
- Arena-based data structures: `extern\haikal\src\meta_arena`
- Data structure tests: `extern\haikal\src\main.c`

This repo aims to translate the C code above into Rust while preserving its low-level strengths and improving safety, clarity, and long-term maintainability.

## Current Status

The project now has an initial Rust crate with:
- `Arena`
- `TempArena`
- `ArenaVec<T>`

The current design direction is:
- allocator primitives use explicit ownership and lifetimes
- higher-level engine storage can use handles or indices where that improves ergonomics
- Rust generics replace the C monomorphization/code generation layer rather than reproducing it directly

## Arena Backend

On Windows, `Arena` uses `VirtualAlloc` directly.

The allocator currently:
- reserves one contiguous virtual memory region up front
- commits pages lazily as allocations advance
- keeps a stable base pointer for the lifetime of the arena
- supports checkpoints, rewind, clear, and temporary rollback scopes through `TempArena`

This mirrors the intended power of the original C arena more closely than a normal heap allocation.

## Current Scope

Right now the focus is the memory layer and the first contiguous container:
- `Arena`
- `TempArena`
- `ArenaVec<T>`

Planned next steps are tracked in `TODO.md`.

## Safety Model

Compared to the original C design, the current Rust version buys a lot in safety and API clarity:
- Rust generics replace the C monomorphization/code generation layer
- `ArenaVec<T>` is tied to the lifetime of the arena it allocates from
- access goes through Rust references and slices instead of raw caller-managed pointer arithmetic
- `TempArena` makes rollback-scoped allocation explicit instead of relying on convention

This is still a pragmatic arena design, not the most restrictive possible Rust model.

Current tradeoff:
- `Arena` uses interior mutability so arena-backed containers can grow ergonomically
- `Arena` and `ArenaVec<T>` are intentionally non-dropping and reject droppable Rust types
- `ArenaVec<T>` grows geometrically by allocating a new larger buffer in the same arena and copying elements forward
- old buffers remain in arena-owned memory until rewind or clear
- manual arena rewind or clear can invalidate existing arena-backed containers if the caller does it at the wrong time

That means the current design is much safer than the original C approach, but it does not yet try to make every invalid post-rewind use impossible at compile time.
