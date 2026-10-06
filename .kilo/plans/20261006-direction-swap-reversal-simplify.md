# Simplification Plan — Task 1 test file (20261006, step 4.3)

Reviewer: code-simplifier. Contract: TODO `.agent/todos/20261006/20261006-todo-1.md` Task 1 +
implementation plan `.kilo/plans/20261006-direction-swap-reversal-failing-tests.md` (authoring source).
Artifact reviewed: `tests/direction_swap_reversal.rs` (179 lines, commit `68f8448`) on branch
`fix/circling-reversal-death`. Verified VM run status: 5 passed / 5 failed (the split is the Task 1
acceptance contract; it must remain untouched).

No source file was modified by this step. This file is the only deliverable.

## 0. Verdict

SIMPLIFICATION REQUIRED — exactly one atomic, zero-semantics edit (S1 below): drop the unneeded `mut`
in `HeadlessLoop::new` to remove the `unused_mut` compiler warning. Everything else reviewed is
already minimal or is load-bearing for the plan's contract (see §2). No src/ change, no assertion
change, no structural change: the 5-fail / 5-pass split, all panic messages, and all pinned
positions stay byte-identical in behavior.

Scope authority note: failing-tests plan §6 grants the simplifier "none beyond formatting" latitude,
but §4.3 simultaneously states the expected outcome "No warnings expected (all six imports are used)".
S1 is the only way to reconcile the shipped file with that expectation, and the 4.3 caller task
explicitly authorizes the mut-drop. S1 therefore brings the file INTO compliance with plan §4.3; it
is a warning fix, not a design change.

## 1. Step S1 — `tests/direction_swap_reversal.rs` line 122

Exact single replacement. Before (lines 121–126 verbatim):

```rust
    fn new() -> HeadlessLoop {
        let mut state = playing_game();
        HeadlessLoop { state, buffer: Vec::new() }
    }

```

After:

```rust
    fn new() -> HeadlessLoop {
        let state = playing_game();
        HeadlessLoop { state, buffer: Vec::new() }
    }

```

Rationale: `state` is only moved into the struct literal; it is never mutated through this binding
(all later mutation flows through `&mut self` in `HeadlessLoop::tick`). rustc's `unused_mut` lint
therefore fires on line 122. `playing_game()` still needs its internal `let mut game`
(`start_playing` takes `&mut self`) — do NOT touch lines 16–20.

## 2. Candidates evaluated and REJECTED (do not act on these)

| # | Candidate | Verdict | Reason |
|---|---|---|---|
| 1 | `assert_snake_survived` loop → `windows(2)` or one-line `.all()` | REJECT | `windows(2)` checks only adjacent pairs and misses head-vs-tail overlap (`[A, B, A]`) — a semantics change to the "no self overlap" contract. A collapsed `assert!(...all...)` loses the per-index `{context}: segment {index} overlaps` panic detail. Current form is depth 2 (fn → for → assert), 2 params, well under 50 lines: rule-compliant already. |
| 2 | `apply_and_step` signature | REJECT | Exactly 2 params (the max), body depth 2, minimal. |
| 3 | `INITIAL_HEAD` literal duplicating setup constants | REJECT | `src/game/setup.rs` lines 9–13 constants are private (verified, no `pub`); the literal is the plan's deliberate coordinate pin (§1.1, §8.1). |
| 4 | Explicit `assert_ne!(status, GameOver, ...)` before `assert_snake_survived` in loop tests (looks duplicated — `assert_snake_survived` also checks status) | REJECT | The §4.3-expected panic messages of the 5 repro failures are produced by these explicit asserts; removing them changes the planned failure shapes. In `loop_tick_survives_continuous_circling_for_two_revolutions` the per-turn `turn {turn:?}` context exists ONLY in that assert (the helper's context string is turn-agnostic), so it is diagnostically load-bearing. |
| 5 | Position/direction asserts inside the 5 pins | REJECT | Required verbatim by plan §1.3 decision 3 (exact-position pins, incl. the `[Right, Down]` pin that constrains the Task 2 fix). Each asserts a distinct dimension (status/length/score/position/direction). |
| 6 | `HeadlessLoop` struct → plain function | REJECT (medium redesign, outside 4.3 latitude) | The struct is the plan-mandated §2.1 design: it keeps the render buffer persistent across the 8 circling ticks (faithful to the real game loop) and keeps every function at ≤2 params; a free function holding the buffer would need 3 params (violates max-arguments rule). Per the caller's constraint this review must not alter the plan's design; if the caller wants it anyway it is a redesign for the architector, not a simplification. |
| 7 | 8-element circling array → `slice::repeat` / nested loops | REJECT | Explicit literal is the most readable form; nested loops would add a third nesting level; iterator/Vec variants add code. |
| 8 | `.expect("headless tick succeeds")` (line 128) | KEEP | Pre-accepted by the task; consistent with the existing suites' pattern. |
| 9 | Long single-line asserts (35, 56, 72, 127, 147) | REJECT | They match the §2.1 authoritative snippets verbatim; re-wrapping is churn with zero quality gain and risks deviating from the plan's pinned text. |

## 3. Execution rules for the implementer (50%-restricted)

1. Apply S1 only: ONE exact-string `edit` on `tests/direction_swap_reversal.rs` — replace
   `let mut state = playing_game();` with `let state = playing_game();` (single occurrence in the
   file; if the edit tool reports multiple or zero matches, STOP and report to caller).
2. Do not touch any other line, any other file, anything under `src/`, `Cargo.toml`, the TODO file,
   or any plan/doc file.
3. No cargo on this host is expected; if `cargo --version` fails, do NOT attempt builds. Report the
   edit as done and let the caller verify.

## 4. Verification (caller, via `alpine-vm` MCP — same ownership split as failing-tests plan §4.1)

1. `vm_status` must report running/SSH-reachable first.
2. Compile check:
   `docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal --no-run`
   Expected: exit 0, and the output MUST NOT contain `unused_mut` / "variable does not need to be mutable".
3. Behavior split unchanged:
   `docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal`
   Expected exactly: `5 passed; 5 failed`, with the same 5 failing names and 5 passing names listed
   in failing-tests plan §4.3, and the same panic messages. Any deviation → STOP and report.
4. Line-count check after the edit: file stays 179 lines (S1 removes a 3-character word, not a line);
   ≤ 200-rule unaffected.

## 5. Git actions (caller-owned timing; not executed by this step)

If the caller instructs the implementer to commit after verification: stage ONLY
`tests/direction_swap_reversal.rs` (run `git status` first; no `target/`, no `dist/`, per
gitignore-compliance rule) with message
`refactor: drop unused mut in direction_swap_reversal test driver`.
Push and TODO `[DONE]` marking remain with the parent workflow steps; this plan does not authorize them.
