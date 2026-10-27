//! The `terminus` binary. Its code is in `lib.rs`.

// The binary is its own crate root, so `lib.rs`'s deny does not reach it.
#![deny(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

// musl's allocator is slow under threads. See mimalloc in the workspace Cargo.toml.
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    terminus::main()
}
