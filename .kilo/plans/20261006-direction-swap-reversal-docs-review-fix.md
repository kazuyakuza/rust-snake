# Fix Plan — Task 3 Docs Review (2026-10-06)

## Verdict basis

The Task 3 doc edits (README.md, docs/terminal-ui.md, docs/testing.md, .agent/project-structure.md, .agent/project-info/context.md) correctly reflect the committed reversal fix and the green test suite:

- `src/game/state.rs` matches the documented behavior: `change_direction` rejects apply-time reversals; `advance_one_step` calls `resolve_impossible_reversal` before computing `next_head`.
- Test counts verified by grep: 71 `#[test]` functions across 9 files, including 10 in `tests/direction_swap_reversal.rs`.
- `Cargo.toml` version is `0.3.3`.
- `git diff 27e1e92..HEAD --stat` touches only the planned doc/plan files; `src/`, `tests/`, `Cargo.*`, and the TODO file are untouched.
- `git status` is clean; none of the edited files are gitignored.
- All plan V-grep checks pass (with addendum B adjudication for the terminal-ui.md wording).

## Findings requiring a fix decision

Two stale claims remain that contradict the now-executed green suite. They are **not** in the exact files/lines the plan ordered edited, but they sit in project-info markdown and conflict with the Task 3 verification criterion "no markdown introduces claims contradicted by code/tests".

### 1. `.agent/project-info/context.md` — internal contradiction in Implementation Status

The section simultaneously contains:

- Line 57 (Phase 1B Group D): "Like all Phase 1A/1B tests they are **authored only, never executed** — no compilation has run in this workflow."
- Line 59 (2026-10-04 side effect): "The 59 authored test functions remain **never executed** — the build command performs the release build only."
- Line 62 (2026-10-06 fix workflow, added by Task 3): "...the full nine-file suite, 71 test functions, executed green in the VM's Docker."

These are timestamped historical notes, but within the same Implementation Status section they read as contradictory. Minimal fix: reword lines 57 and 59 to past tense or add a superseded-by note so they no longer assert a present "never executed" state.

### 2. `.agent/project-info/architecture.md` — stale test summary (out of Task 3 scope)

Line 141 still states:

> Tests: eight authored integration files under `tests/` (six Phase 1A + two Phase 1B), 59 `#[test]` functions total; **authored only, not yet executed** ...

Current state is 9 files, 71 functions, executed green in the VM as of 2026-10-06. This file was not part of the Task 3 file list, so fixing it is scope expansion; flag only.

## Proposed fix steps (tiny, junior-proof)

1. Open `.agent/project-info/context.md`.
2. In the Phase 1B Group D bullet (line 57), change:
   - From: "Like all Phase 1A/1B tests they are **authored only, never executed** — no compilation has run in this workflow."
   - To: "Like all Phase 1A/1B tests they were **authored only and not yet executed** in the Phase 1B workflow — execution arrived with the 2026-10-06 fix workflow (see Recent Changes)."
3. In the 2026-10-04 side-effect bullet (line 59), change:
   - From: "The 59 authored test functions remain **never executed** — the build command performs the release build only."
   - To: "The 59 authored test functions had not yet been executed at this point — the build command performed the release build only; execution arrived with the 2026-10-06 fix workflow (see Recent Changes)."
4. Run `grep -n "never executed\|authored only" .agent/project-info/context.md` and confirm the remaining occurrences are historical and no longer assert a current "never executed" state.
5. (Optional / out of Task 3 scope) Open `.agent/project-info/architecture.md` and refresh line 141 to: 9 integration test files, 71 `#[test]` functions, executed green in the Alpine VM Docker as of 2026-10-06.
6. Run `git status`, stage only the edited file(s), and commit with message `docs: reconcile historical test-execution claims with current green suite` (or split into two commits if both files are edited).

## Scope note

These edits are documentation-only and require no `cargo`/`docker` runs. If the caller decides the historical bullets are acceptable as-is and architecture.md is outside scope, this fix plan can be discarded.
