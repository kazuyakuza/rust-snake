# Fix Plan — Task 1 Code Review (step 4.3)

## Scope

Only `tests/direction_swap_reversal.rs` is affected. No `src/` changes. No plan/TODO/doc edits.

## Findings

The test file is structurally correct: 10 tests (5 repro + 5 pins), all imports used, API names verified, 179 lines, no commented-out code, snake_case naming, helpers ≤ 2 args, nesting depth ≤ 2, and the expected 5-pass/5-fail split is reproduced.

One deviation from the plan's §3.7 compile requirement ("Must exit 0 with no warnings"):

- Line 122 declares `let mut state = playing_game();` inside `HeadlessLoop::new()`. The binding is never mutated; it is only moved into the struct. `rustc` will emit an `unused_mut` warning. This was copied verbatim from plan §2.1, but the plan also requires a warning-free build.

## Required Change

File: `tests/direction_swap_reversal.rs`

Replace:

```rust
        let mut state = playing_game();
```

With:

```rust
        let state = playing_game();
```

This removes the unnecessary `mut` and eliminates the warning without changing behavior.

## Verification Steps

1. Run a compile-only check and confirm zero warnings:

   ```text
   docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal --no-run
   ```

2. Run the test target and confirm the 5-pass/5-fail split is unchanged:

   ```text
   docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal
   ```

   Expected summary line:

   ```text
   test result: FAILED. 5 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
   ```

3. Confirm the failing and passing test name sets still match plan §4.3 exactly.
