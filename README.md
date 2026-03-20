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
