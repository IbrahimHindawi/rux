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
