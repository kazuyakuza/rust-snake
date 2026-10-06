# Adherence Report — Task 1 (step 4.5b): Failing Tests Reproducing the Circling-Reversal-Death Bug

- Date: 2026-10-06
- Workflow step: 4.5b Overall Plan Adherence — Task 1 only (per Critical Workflow)
- TODO: `.agent/todos/20261006/20261006-todo-1.md` — Task 1: "Write failing tests that reproduce the circling-death bug"
- Authoring plan: `.kilo/plans/20261006-direction-swap-reversal-failing-tests.md`
- Review plans: `.kilo/plans/20261006-direction-swap-reversal-review-fix.md`, `.kilo/plans/20261006-direction-swap-reversal-simplify.md`
- Artifacts inspected: `tests/direction_swap_reversal.rs`, `docs/testing.md`
- Branch: `fix/circling-reversal-death` — clean working tree (`git status --porcelain` empty)

## Verdict

**ADHERENT** — TODO Task 1 is satisfied, the shipped artifacts match the authoring plan §2.1 (with the pre-authorized S1 mut-drop), the verification output shape matches plan §4.3, and the git scope contains exactly the Task 1/4.3/4.4 workflow deliverables. All deviations found are classified **acceptable** (§8). No corrective action is required from the caller.

## 1. Evidence base

- `git log --oneline -10` — Task 1 range (post `803f6e1`): `751e25b` (plan), `68f8448` (failing tests), `6b9b2e6` (review+simplify plans), `3769dc9` (mut drop), `707d5de` (testing guide). Matches the caller's stated commit order exactly.
- `git diff 803f6e1..HEAD --stat` — only 5 files changed repo-wide: 3 plan files, `docs/testing.md` (+54), `tests/direction_swap_reversal.rs` (+179). Nothing under `src/`, no `Cargo.toml`, no `.cargo/`, no `compose.yaml`, no `Dockerfile`, no `.agent/` files — `src/` untouched requirement PROVEN by the full-range stat (it lists every changed path in the range).
- `git diff 68f8448..3769dc9` — the mut fix is a single-line change, exactly S1 (`let mut state = playing_game();` → `let state = playing_game();`), 1 insertion / 1 deletion.
- Docker verification: performed by the CALLER via Alpine VM (`cargo test --test direction_swap_reversal`): `5 passed; 5 failed; 0 ignored`, the 5 failures are the expected bug repros, 5 pins pass, clean output. This step is read-only and re-attests the caller's run rather than re-running it.
- `cargo --version` (host-toolchain empirical re-check attempted per discipline): blocked by the session permission policy in this read-only step. The "no local toolchain" claim is instead verified against plan §3 step 6 ("cargo NOT available = EXPECTED on this host") and `docs/BUILD.md` ("Rust / Cargo — NOT required on the host"). Noted as a limitation, not a deviation.

## 2. TODO Task 1 — requirement mapping

| TODO Task 1 requirement | Status | Evidence |
|---|---|---|
| New integration test file `tests/direction_swap_reversal.rs` | ✅ | File exists, committed in `68f8448`, imports via `snake::game::*` / `snake::terminal::*` |
| Domain-level test: `change_direction` multiple times before/within a tick + `advance_one_step`, sequences `[Up, Left]`, `[Up, Left, Down]`, full circle of four swaps applied between ticks; assert NOT GameOver + snake intact | ✅ | `apply_and_step` helper replicates the loop's per-tick behavior (plan §1.3.4); tests `two_key_burst…`, `three_key_burst_not_ending_in_reversal…`, `one_turn_per_tick_circles_back…`; `assert_snake_survived` asserts status/length/score/segment-uniqueness |
| Loop-level test through `game_loop::tick`, headless in-memory `Vec<u8>` renderer, surviving continuous circling (4+ swaps forming a loop) | ✅ | `HeadlessLoop` over `Renderer\<&mut Vec\<u8\>\>`; `loop_tick_survives_continuous_circling_for_two_revolutions` = 8 turns (two revolutions); 2 burst repros add loop-layer coverage |
| Run `cargo test --test direction_swap_reversal` (Docker/Alpine VM) and confirm the new tests FAIL | ✅ | Caller-attested run: 5 passed / 5 failed / 0 ignored; failures are exactly the expected repros (§4 below) |
| Commit the failing tests | ✅ | `68f8448 test: add failing regression tests for the circling reversal bug` — commits only `tests/direction_swap_reversal.rs` |

Prompt-literal scenario (b) `[Up, Left, Down]` cannot kill the snake on the verified real coordinates (head (10,12), neck (9,12), horizontal snake; the burst steps to (10,13) — a free cell). The authoring plan honestly re-roles it as a survival pin (`three_key_burst_not_ending_in_reversal_stays_alive`) while the genuine death repros are `[Up, Left]`, `[Right, Down, Left]`, and the two-tick interleave, all verified failing. The TODO's literal wording (assert NOT GameOver for those sequences) is satisfied verbatim; the reproduction requirement is satisfied by the verified failing set. ACCEPTABLE (documented in plan §1.2/§7 and in `docs/testing.md`).

## 3. Authoring plan §2.1 structural fidelity — line-by-line

- Module header: 4 `//!` lines — verbatim match to the §2.1 snippet. Zero other comments in the file.
- Imports: the six `use` lines, same order, all used (clean compiler output attests no warnings). ✅
- Constants: `INITIAL_HEAD` `(10,12)`, `INITIAL_SNAKE_LENGTH` 3. ✅
- Helpers: `playing_game`, `apply_and_step`, `assert_snake_survived` — verbatim, including all context strings. ✅
- 7 domain tests: names, order, bodies, and assertions match §2.1 exactly. ✅
- Loop section: `HeadlessLoop` struct + `new`/`tick` + 3 loop tests match §2.1 exactly, with the single pre-authorized S1 delta at line 122. ✅
- No additional files, no extra tests, no extra asserts introduced. ✅

## 4. Semantics — pinned assertions unchanged

- Exact-position pins intact and confined to the stable cases (plan §1.3 decision 3):
  - Circling pins end with `assert_eq!(game.snake().head(), INITIAL_HEAD)` — head back at `(10,12)` (lines 89, 178). ✅
  - Plain turn pin: `(11,12)` then `(11,13)` via `INITIAL_HEAD + Direction::Right.offset()` (+ `Down.offset()`), plus `current_direction() == Down` (lines 96–102). ✅
  - `[Right, Down]` one-tick turn pin: `current_direction() == Down` and head `INITIAL_HEAD + Down.offset()` = `(10,13)` (lines 110–111) — the deliberate Task 2 constraint (a same-direction no-op must not consume the per-tick turn allowance). UNCHANGED. ✅
- Fix-dependent cases (all three repros + the `[Up, Left, Down]` pin) assert only status/length/score/segment-uniqueness — no position or direction asserts, and no `change_direction` bool-return asserts anywhere in the file (plan §1.3 decisions 1–2). ✅
- `start_playing()` is invoked before any step/tick via `playing_game()` (plan §1.3 decision 6). ✅
- No food interactions anywhere; pins keep `score() == 0` and length 3 (plan §1.3 decision 7). ✅

## 5. Expected Docker output shape (plan §4.3)

Caller-attested: summary `5 passed; 5 failed; 0 ignored` — matches §4.3's shape; the 5 failing names are exactly the §4.3 failure set (`two_key_burst_within_one_tick_must_not_step_onto_the_neck`, `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck`, `two_tick_interleaved_burst_must_not_reenter_the_body`, `loop_tick_survives_a_two_key_burst_in_one_tick`, `loop_tick_survives_a_three_key_burst_ending_in_reversal`) and the 5 passing names are exactly the §4.3 pin set. "Clean output" confirms the §4.3 no-warnings expectation post-mut-fix (review-fix verification steps 1–3 satisfied; simplify plan §4 steps 2–4 satisfied). The expected panic-message shapes are encoded verbatim in the file (assert context strings match §4.3). Section unresolved: none.

## 6. Git scope and commit hygiene

- Commits limited to plan/test/docs files; `src/`, `Cargo.toml`, `.cargo/`, lockfiles, Docker infra: untouched (proven by full-range `--stat`). ✅
- Commit messages for the two §5-authorized commits match the plan verbatim (`751e25b`, `68f8448`). ✅
- `target/`/`dist/` never appear; `.gitignore` compliance holds; working tree clean. ✅
- TODO file NOT marked `[DONE]` (correct — that is step 4.6). ✅
- No push performed (Step 5 ownership preserved). ✅

## 7. `docs/testing.md` factual accuracy vs the test file

| Claim in docs/testing.md | Verified against | Result |
|---|---|---|
| Tests run inside pinned Docker image `rust:1.98.1-slim-bookworm`; `CARGO_TARGET_DIR=/tmp/target` keeps `target/` in-container | `Dockerfile` line 8; authoring plan §4.2; compose convention | ✅ |
| Single-suite Docker command (exact command line) | Plan §4.2 command — byte-identical | ✅ |
| "no local Rust toolchain on the host (see BUILD.md)" | `docs/BUILD.md` ("NOT required on the host"); plan §3.6 expectation. Empirical `cargo --version` re-check blocked by session permissions (read-only step limitation) | ✅ (consistent with documented sources) |
| Initial 3-segment snake: head `(10, 12)`, heading `Right`, score `0` | `src/game/setup.rs` constants per plan §1.1; `INITIAL_HEAD` const in test file | ✅ |
| Layers: domain via `change_direction` + `advance_one_step`; loop via `terminal::game_loop::tick` + in-memory `Vec<u8>` renderer (`HeadlessLoop`); helpers named | Test file lines 22–27, 115–130 | ✅ |
| "currently split 5 / 5": five repros fail until fix, five pins pass | Caller-attested Docker run | ✅ |
| Status table (10 rows, layer + status per test) | All 10 names/layers/statuses match the file exactly; repro vs pin classification matches plan §1.2/§4.3 | ✅ |
| `[Right, Down]` pin wording ("legitimate double-turn toward a free cell, final direction `Down`") | Test lines 107–112; plan §1.2 row 7 | ✅ |
| Burst-ending-in-reversal wording ("ends opposite the last-moved direction must be rejected") | Plan §1.2 row 2 semantics | ✅ |
| Circling tests "return the head to `(10, 12)` without any self-overlap" | Test lines 88–89, 176–178; `assert_snake_survived` | ✅ |

No factual inaccuracies found in `docs/testing.md`. It is developer-facing documentation (not README / `docs/terminal-ui.md` / `context.md`), so it does not collide with Task 3's documented scope.

## 8. Deviations and classification

| # | Deviation | Classification | Rationale |
|---|---|---|---|
| D1 | Commit `3769dc9` message is `test: drop unused mut in headless loop builder`, while simplify plan §5 suggested `refactor: drop unused mut in direction_swap_reversal test driver` | **Acceptable** | Simplify plan §5 explicitly made commit timing/wording caller-owned ("If the caller instructs…"). The commit's staged content is exactly S1 (single-line mut drop, verified in the diff). Zero impact on artifacts or contract. |
| D2 | `docs/testing.md` (commit `707d5de`) exceeds the authoring plan §5's two authorized commits and the letter of §0 guard 5 ("exactly two artifacts") | **Acceptable** | Caller-sanctioned docs deliverable listed in the caller's own Task 1 execution summary; §0 guard 4 prohibited only Task 3's user-facing files (README / docs/terminal-ui.md / context.md), which were not touched. Markdown rule permits documentation creation by planner/docs role. Docs step content is factually accurate (§7). |
| D3 | Review/simplify plan files committed in `6b9b2e6` (beyond the §5 two commits) | **Acceptable** | Standard step 4.3 workflow deliverables, anticipated as downstream hooks in authoring plan §6; created by the review/simplify roles per the markdown rule. |
| D4 | Plan-internal §2.2 metric nits: header described as "5-line //! module header" (the snippet itself is 4 lines) and "≈177 lines" (actual 179) | **Acceptable** | Plan-internal estimate inconsistencies, not implementer deviations; §2.1 snippets are declared authoritative and the file matches them verbatim. No action required. |
| D5 | `ACC` noted: the literal TODO scenario (b) `[Up, Left, Down]` cannot reproduce a death on the real coordinates and ships as a survival pin instead of a failing repro | **Acceptable** | Covered by plan §1.2 honest re-role + §7 mapping; the TODO's failing-test requirement is fulfilled by the verified failing set; `docs/testing.md` documents the 5/5 split transparently. |
| D6 | Host `cargo --version` empirical re-check blocked by this session's permission policy | **Acceptable (limitation)** | Read-only step; the claim is independently corroborated by `docs/BUILD.md` and plan §3.6 and §4.1 ownership split. No artifact depends on it. |

No deviation requires caller-scheduled corrective action. No assertion was weakened; no scope was touched beyond the documented deliverables.

## 9. Rules compliance summary (test file)

- `max-lines-per-file`: 179 / 200. ✅
- `max-arguments-per-method`: every function/method ≤ 2 params (`HeadlessLoop` bundles state+buffer for this exact reason). ✅
- `max-lines-per-method`: longest body ~15 lines. ✅
- `max-depth`: nesting depth 2 throughout (per plan's already-reviewed determinations). ✅
- Single-section boolean conditions: the only boolean expression is a single unary call negation. ✅
- Self-documenting code / no-commented-code: only the 4-line //! header; zero commented code. ✅
- Prefer-private-members: `HeadlessLoop` fields private; helpers file-private. ✅
- Newline rule: real newlines (file reads normally, 179 numbered lines). ✅
- Assertions/magic numbers: named `INITIAL_SNAKE_LENGTH`; pervasive assertions. ✅

## Conclusion

Task 1 of `.agent/todos/20261006/20261006-todo-1.md` was implemented exactly as planned at both required layers, reproduces the bug on the unfixed code (verified by the caller's Docker run), pins the boundary contract for Task 2 (`[Right, Down]` pin intact), violates no rules, and keeps `src/` untouched. **ADHERENT.**

Adherence report saved to: `.kilo/plans/20261006-direction-swap-reversal-failing-tests-adherence.md`
