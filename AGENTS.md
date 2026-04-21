# AGENTS.md

Guidance for AI coding agents working on this repository.

## Project Structure

Rust workspace with 17 crates under `packages/`. Each domain (encryption,
filter, worktree, repository, recipient, ssh_identity) has a logic crate and a
`models` sub-crate containing pure serde data types. The `cli` crate
orchestrates everything. Two alias binaries (`git-sshript`, `git-sshrypt`)
delegate to the CLI.

## Verification (Required After Every Change)

Run these commands in order after making any code changes. All three must pass
before considering work complete:

1. `cargo fmt` -- format all code first
2. `cargo clippy --all-targets` -- must produce zero warnings
3. `cargo test` -- all tests must pass

**Critical:** Always run `cargo fmt` before `cargo clippy`. Formatting can
expand compressed lines and push functions over the `too_many_lines` threshold.
Never rely on manual line compression to satisfy clippy -- if `cargo fmt`
undoes your fix, refactor the code properly (extract helper functions, etc.).

## Lint Configuration

Every `lib.rs` enables strict lints:

```rust
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
```

Key implications:

- **`too_many_lines`** -- functions must stay under 100 content lines. Extract
  helpers rather than compressing formatting.
- **`missing_errors_doc`** -- all public functions returning `Result` need a
  `# Errors` doc section.
- **`cargo_common_metadata`** -- every crate needs `readme = "README.md"` in
  its `Cargo.toml`.
- **`needless_pass_by_value`** -- prefer `&str` / `&[T]` / `Option<&str>` over
  owned parameters where possible.

## Code Patterns

- Use config structs instead of many function parameters when a function would
  exceed 7 args or 3 bools (see `RevokeUserOptions`, `MigrateOptions`).
- Model crates (`*/models/`) contain only serde-serializable types with no
  logic.
- Helper functions extracted from long functions should be private (`fn`, not
  `pub fn`) and placed adjacent to the function they were extracted from.

## Debugging performance

Span-level instrumentation is available via the [`profiling`](https://crates.io/crates/profiling)
crate. Use it to investigate latency in any command (for example why
`unlock --soft` is slower than expected) without guessing.

Build with the `profile-trace` feature and set `GIT_SSHRIPPED_TRACE=1` at
run time:

```bash
cargo build --release --features git_sshripped_cli/profile-trace
GIT_SSHRIPPED_TRACE=1 ./target/release/git-sshripped <subcommand> 2>trace.log
```

Benchmarks live in `packages/cli/benches/unlock.rs` (criterion harness).
They are opt-in via `BENCH_REPO` and `GIT_SSHRIPPED_BIN` env vars so CI stays
cheap. See README.md's _Debugging performance_ section for full usage.

When adding new subprocess spawns or hot helpers, wrap them in
`profiling::scope!("name")` so the trace output stays comprehensive. The
macro compiles to a no-op when the `profile-trace` feature is off.
