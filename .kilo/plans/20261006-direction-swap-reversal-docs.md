# Plan — Task 3: Update documentation and project context (direction-swap reversal fix)

- **Workflow position:** Critical Workflow step 4.1b (analysis + planning) for Task 3 of TODO `.agent/todos/20261006/20261006-todo-1.md`.
- **Repo:** `C:\repo\rust-snake`. **Branch:** `fix/circling-reversal-death`, HEAD `27e1e92`, clean tree at planning time.
- **Implementer profile:** junior developer under 50% restriction. Execute ONLY the numbered steps below. No source-code edits, no `cargo`/`docker` runs, no push, no TODO-file edits (the workflow's later step handles `[DONE]` marks).
- **TODO Task 3 lines being implemented:**
  1. Update the game-rules documentation (README gameplay/rules section, `docs/terminal-ui.md` if it documents reversal rules) to state the corrected rule: the snake can never begin a move going the direction opposite to the direction it moved in the previous tick (segment-2-based rule), whichever buffer-drain policy is used.
  2. Update `.agent/project-info/context.md` (Recent Changes + Implementation Status) describing the bug, fix, and tests.

---

## 0. Verified facts (single truth source for every replacement sentence)

Each fact was checked against the working tree at HEAD `27e1e92` before this plan was written:

- **Fix commit:** `658e2b2` touches ONLY `src/game/state.rs` (+38/-1). `GameState::advance_one_step` (line 133) now calls `resolve_impossible_reversal()` (line 137) BEFORE computing `next_head` (line 138).
- **Resolution logic (`src/game/state.rs:112-129`):** `direction_of_last_move()` derives the actually-moved direction from the neck→head geometry (`NECK_SEGMENT_INDEX = 1`, segments ordered head-first; always `Some` for the ≥3-segment snake). If the pending `current_direction` equals that last-moved direction's `opposite()`, it is reset to the last-moved direction — so a move can never BEGIN opposite the previous actual move, no matter how many presses were buffered/drained.
- **Apply-time API unchanged:** `GameState::change_direction` (lines 104–110) still rejects a direction directly opposite the current pending direction and returns a `bool`. Its own doc-comment acknowledges the press-drain gap handled by `resolve_impossible_reversal`.
- **Legit-turn pins (all green in `tests/direction_swap_reversal.rs`):** `right_angle_turn_between_ticks_still_turns` (one turn per tick), `double_key_turn_within_one_tick_toward_free_cells_still_turns` (`[Right, Down]` inside one tick), `one_turn_per_tick_circles_back_to_the_start_cell` and `loop_tick_survives_continuous_circling_for_two_revolutions` (circling returns the head to `(10, 12)`).
- **Regression suite:** `tests/direction_swap_reversal.rs` contains exactly 10 `#[test]` functions — 5 bug repros + 5 pins — split 5/5 as documented.
- **Full suite:** 71 `#[test]` functions across 9 integration files, ALL GREEN (verified by counting on disk + the Task-2 VM Docker full-suite run). Counts: `direction.rs` 13, `terminal_modules.rs` 13, `collision.rs` 9, `direction_swap_reversal.rs` 10, `movement_and_growth.rs` 7, `initial_state.rs` 7, `food_consumption_scoring.rs` 4, `food_placement.rs` 4, `gameplay_flow.rs` 4.
- **Version:** `Cargo.toml` line 3 = `0.3.3` (bumped `a040737`; `Cargo.lock` `82b900f`).
- **Commits on the branch (Task 1–2):** `68f8448` (failing tests), `6b9b2e6` (task-1 review/simplification plans), `3769dc9` (test fix), `707d5de` (`docs/testing.md` created), `7b483f6`/`25e66cc` (task-1 adherence/done), `7224d26` (task-2 plan), `a040737` + `82b900f` (version 0.3.3), `658e2b2` (the fix), `4a61bd3`/`27e1e92` (task-2 adherence/done).
- **`dist/snake.exe` on disk predates the fix** (last built at 0.3.2 in the 2026-10-05 half-block workflow). No doc may claim `dist/snake.exe` currently contains the 0.3.3 logic.

---

## 1. README.md — exact edits (3 locations)

### 1.1 Game Rules & Controls bullet (line 29) — the corrected rule

**Current (verbatim, line 29):**
```markdown
- Controls: arrow keys change direction; movement continues between ticks; immediate reversal into itself is rejected.
```

**Replace that single bullet with these two bullets:**
```markdown
- Controls: arrow keys change direction; movement continues between ticks; the snake can never begin a move going the direction opposite to the direction it actually moved in the previous move — no matter how many direction presses are buffered between moves, a quick multi-swap input can never reverse the snake onto its own body (the head can never step onto the cell its neck occupies).
- 90-degree turns still work: one turn per move, or a double-turn drained within a single move toward free cells.
```

### 1.2 Terminal UI section, tick pipeline parenthetical (line 38)

**Current fragment (verbatim, line 38):**
```markdown
applies them through the domain's `change_direction` (which still rejects immediate reversals), advances one step,
```

**Replace with:**
```markdown
applies them through the domain's `change_direction` (immediate reversals are still rejected at apply time, and `advance_one_step` additionally resolves a buffered burst that would end up opposite the last moved direction before the move begins), advances one step,
```

### 1.3 Input bullet wording (line 41)

**Current fragment (verbatim, line 41):**
```markdown
Immediate-reversal rejection is **not** done here — it stays in the domain rules under `src/game`, which remain the single source of truth for movement.
```

**Replace with:**
```markdown
Reversal enforcement is **not** done here — it stays in the domain rules under `src/game` (`change_direction` apply-time rejection plus the step-time impossible-reversal resolution in `advance_one_step`), which remain the single source of truth for movement.
```

### TOC check (README lines 7–14)

The Table of Contents exists. Sections 1.1–1.3 add no heading (the `- 90-degree turns…` line is part of the same `## Game Rules & Controls` list), remove no heading, and rename no heading → **TOC stays untouched; verify anchors still resolve.**

---

## 2. docs/terminal-ui.md — exact edits (3 locations, all reversal-related)

### 2.1 Design Principles bullet «Reversal is a domain concern» (lines 43–45)

**Current (verbatim):**
```markdown
- **Reversal is a domain concern.** Rejecting an immediate reversal belongs to
  `src/game`, not the terminal layer — see the "no duplicated rules" constraint
  in the Phase 1B TODO. The input layer only reports the requested direction.
```

**Replace with (keep the same 2-space continuation indent):**
```markdown
- **Reversal is a domain concern.** Reversal enforcement belongs to
  `src/game`, not the terminal layer — see the "no duplicated rules" constraint
  in the Phase 1B TODO. The input layer only reports the requested direction.
  The domain enforces this in two places: `change_direction` rejects an
  immediate reversal when a press is applied, and `advance_one_step` resolves
  an impossible reversal before the move begins — several presses drained
  within one tick were each legal against the intermediate direction, yet the
  burst can end opposite the direction actually moved in the previous move
  (`resolve_impossible_reversal`, anchored on the neck→head geometry).
```

### 2.2 «How `main.rs` Connects It» tick pipeline (lines 121–124)

**Current (verbatim):**
```markdown
3. A `Renderer` shares the handle's stream via `output()`, then
   `run_playing_loop` runs each tick: `drain_arrow_directions()` →
   `change_direction` (the domain rejects reversals) → `advance_one_step()` →
   `Renderer::render(&state)` → sleep the remainder of the tick, returning once
   the status leaves `Playing`.
```

**Replace the two middle lines only; the arrow chain itself stays identical:**
```markdown
3. A `Renderer` shares the handle's stream via `output()`, then
   `run_playing_loop` runs each tick: `drain_arrow_directions()` →
   `change_direction` (the domain rejects apply-time reversals) →
   `advance_one_step()` (which first resolves any impossible buffered
   reversal against the last moved direction) →
   `Renderer::render(&state)` → sleep the remainder of the tick, returning once
   the status leaves `Playing`.
```

### 2.3 «How to Validate Manually» reversal bullet (line 168)

**Current (verbatim):**
```markdown
- Arrow keys steer the snake; a press that would reverse into itself is ignored.
```

**Replace with:**
```markdown
- Arrow keys steer the snake; a single press opposite the current direction is
  ignored, and rapid multi-key swapping drained within one tick can never
  reverse the snake onto its own body — quick `Up`/`Left` bursts and continuous
  circling must not end the game, while plain 90-degree turns still steer.
```

### Explicitly out of scope in this file (do NOT edit)

The **Status section (lines 9–18) and the test-count claims** (lines 13–14, 152–157, and the "Headless Tests (Group D)" section) are stale — they say the tests are "authored only" and that execution "arrives with the Docker build phase", while 71 tests now execute green. That staleness is about test-execution history, not the reversal rule or input behavior, and is therefore outside TODO Task 3's line. Recorded for the caller in §7. Lines 141–143 ("reversal rejection" in the tick hook list) and line 45 remain factually true → leave untouched.

---

## 3. docs/testing.md — staleness decision + minimal corrections

**Decision: UPDATE.** TODO Task 3 does not list `docs/testing.md`, but the caller delegated this decision explicitly. The file currently asserts in present tense that five tests "fail until the direction-handling fix lands" and labels them `Fails until fix`; with Task 2 landed (`658e2b2`) and all 10 suite tests green in the VM, those statements are factual errors that would mislead any future agent into treating the bug as open. Stale "currently failing" claims are worse than none; the correction is three small text edits with zero behavioral risk. Justification encoded here per the caller's instruction.

### 3.1 Split-status paragraph (lines 30–32)

**Current (verbatim):**
```markdown
The suite is currently split 5 / 5: five tests are bug repros that **fail** until
the direction-handling fix lands, and five are regression pins that pass now and
must stay green.
```

**Replace with:**
```markdown
The suite is split 5 / 5: five tests are bug repros that were authored to fail
against the pre-fix domain and loop code, and five are regression pins. With the
step-time impossible-reversal resolution in `advance_one_step` (version 0.3.3),
all ten tests pass and must stay green.
```

### 3.2 Table Status column (lines 36–40 — the five repro rows)

**Current cells (verbatim):**
```markdown
| `two_key_burst_within_one_tick_must_not_step_onto_the_neck` | domain | Fails until fix |
| `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck` | domain | Fails until fix |
| `two_tick_interleaved_burst_must_not_reenter_the_body` | domain | Fails until fix |
| `loop_tick_survives_a_two_key_burst_in_one_tick` | loop | Fails until fix |
| `loop_tick_survives_a_three_key_burst_ending_in_reversal` | loop | Fails until fix |
```

**Change only the last column of these five rows to `Bug repro (green since 0.3.3)`.**
The five `Passes (pin)` rows (lines 41–45) remain unchanged.

### 3.3 Boundary-pins paragraph (lines 50–51)

**Current fragment (verbatim):**
```markdown
while the failing `..._three_key_burst_ending_in_reversal_...` tests pin that a
burst ending opposite the last-moved direction must be rejected.
```

**Replace with:**
```markdown
while the bug-repro `..._three_key_burst_ending_in_reversal_...` tests pin that a
burst ending opposite the last-moved direction must be resolved before the move
begins.
```

**Explicitly out of scope:** the `//!` file-header comment of `tests/direction_swap_reversal.rs` (lines 2–4, "The failing tests reproduce the bug … must turn green with the Task 2 fix") — that is a source-file comment describing the authoring-time design, not documentation; editing it is outside Task 3 (no source edits) and factually harmless (the tests did reproduce the bug when authored).

---

## 4. `.agent/project-info/context.md` — exact edits

Line numbers refer to the file as read at planning time (76 lines). Follow the file's existing bullet style: bold lead-in, dense factual prose, backticked paths/commit SHAs, newest-first in Recent Changes, oldest-first in Implementation Status.

### 4.1 Current Work Focus

**(a) Insert ONE new top bullet above line 5** (the file's first focus bullet), with this exact content:

```markdown
- **Rapid-direction-swap reversal fix workflow (TODO `.agent/todos/20261006/20261006-todo-1.md`) completed through its docs task.** Tasks 1–2 (accepted `[DONE]`): the circling-death bug — bursts of arrow presses drained within one tick (e.g. `[Up, Left]` on the initial 3-segment snake) left the pending direction opposite the actually-moved direction, so the snake stepped onto its own neck → `GameOver` — is fixed in `src/game/state.rs`: `advance_one_step` resolves an impossible reversal (`resolve_impossible_reversal`, neck→head geometry) before computing the next head position, so the snake can never begin a move going the direction opposite to the direction it actually moved in the previous move; quick multi-swap inputs can never reverse onto the body; legit 90-degree turns (one per move, or a double-turn within one move toward free cells) still work; `change_direction`'s apply-time rejection is unchanged; the terminal layer is untouched. Regression suite `tests/direction_swap_reversal.rs` (10 test functions); the full suite — 71 test functions across nine files — executed green in the Alpine VM Docker. Version `0.3.3`. Task 3 updated the game-rule docs (README, `docs/terminal-ui.md`, `docs/testing.md`) and this file per the planner plan `.kilo/plans/20261006-direction-swap-reversal-docs.md`.
```

**(b) Replace the stale "Next up" bullet (line 7).**

**Current (verbatim, line 7):**
```markdown
- Next up: see **Immediate Next Steps** below — manual Windows validation of the rebuilt `dist/snake.exe` (half-block packed 82×43 frame, fits standard consoles) and optional full `cargo test` execution.
```

**Replace with:**
```markdown
- Next up: see **Immediate Next Steps** below — close the workflow (merge `fix/circling-reversal-death` to `main` and push to `origin`), optionally rebuild `dist/snake.exe` inside the VM's Docker so the Windows artifact includes version 0.3.3 (the on-disk exe predates the reversal fix), then manually validate on Windows.
```

### 4.2 Recent Changes — insert ONE new first bullet (newest-first order)

Insert directly above line 14 (currently the "Board + rendering workflow (2026-10-05, …)" bullet), with this exact content. The `<DOC-SHA>` placeholder is filled in sub-step 4.5 after the commits in §6 exist:

```markdown
- **Rapid-direction-swap reversal fix workflow (2026-10-06, TODO `.agent/todos/20261006/20261006-todo-1.md`)** executed via the Critical Workflow on branch `fix/circling-reversal-death` (HEAD `27e1e92` at planning of its docs task; not yet merged to `main` — merge state to be recorded in the next context update). Task 1: pinned the bug with `tests/direction_swap_reversal.rs` (`68f8448`; 10 test functions — 5 failing bug repros + 5 pins, committed while failing, reproduced the circling death of the report). Task 2: fixed the reversal-rejection logic in `src/game/state.rs` (`658e2b2`) — `advance_one_step` now calls `resolve_impossible_reversal` before computing the next head position, re-anchoring a buffered burst that ends opposite the actually-moved direction, keeping `change_direction`'s apply-time rejection unchanged (`a040737` bumps the version to 0.3.3, `82b900f` refreshes `Cargo.lock`). Verification: the full suite — 71 test functions across nine integration files, including all 10 of the new regression suite — executed green in the Alpine VM Docker on the Linux host target. Task 3 (docs step): README game-rules bullet and terminal-layer wording, the `docs/terminal-ui.md` reversal principle/pipeline/manual-validation wording, the `docs/testing.md` statuses corrected to all-green, and this file updated (`<DOC-SHA>`).
```

### 4.3 Implementation Status — insert ONE dated bullet

Insert after line 59 (the bullet beginning "**Implemented (2026-10-05 fix workflow, TODO `.agent/todos/20261005/20261005-todo-2-DONE.md`):**") and before line 60 (the standing note "The domain logic in `src/game/` remains the **untouched source of truth**…" stays last), with this exact content:

```markdown
- **Implemented (2026-10-06 fix workflow, TODO `.agent/todos/20261006/20261006-todo-1.md`):** the step-time impossible-reversal resolution in `src/game/state.rs` — `advance_one_step` calls `resolve_impossible_reversal` before computing the next head position; the pending direction is compared against the direction actually moved in the previous move (derived from the neck→head geometry) and reset to it when it would send the snake back onto its own body, so a move can never begin opposite the last-moved direction no matter how many direction presses are buffered, while `change_direction`'s apply-time rejection and the terminal layer stay unchanged. Regression suite `tests/direction_swap_reversal.rs` (10 test functions — 5 bug repros + 5 pins); the full nine-file suite, 71 test functions, executed green in the VM's Docker. Version `0.3.3`.
```

### 4.4 Immediate Next Steps — replace stale items (lines 68–70)

**Current items (verbatim):**
```markdown
1. Manually validate gameplay on Windows by running the rebuilt `dist/snake.exe` (82×43 frame — fits standard consoles, no window resizing needed) against the Definition of Done checklist in `brief.md` §18 plus the 2026-10-05 fixes (whole frame visible without scrolling; contiguous colored snake; visually equal vertical/horizontal speed); report any runtime or visual issues back into a fix TODO.
2. Optionally execute the remaining authored tests (`cargo test` full run — 17 of 59 executed green so far; the Phase 1A suite is still run-only-pending) in the VM's Docker in a later phase.
3. User-side (outside this repo): optionally give `C:\repo\vm-ssh-mcp` a remote (`git remote add origin ...`) if it should be backed up.
```

**Replace with exactly this list:**
```markdown
1. Close the fix workflow when its remaining steps finish: merge `fix/circling-reversal-death` to `main`, push to `origin`, delete the feature branch, and record the merge in the next context update.
2. Manually validate gameplay on Windows by rebuilding `dist/snake.exe` in the VM's Docker (the on-disk exe is version 0.3.2 and predates the 0.3.3 reversal fix; run `docker compose run --rm build`) against the Definition of Done checklist in `brief.md` §18 plus the 2026-10-05 rendering fixes and the 2026-10-06 fix behaviors (rapid multi-press direction swaps can never make the snake double back onto itself or end the game while circling; a single press opposite the last direction is still ignored; legit 90-degree turns still steer); report any runtime or visual issues back into a fix TODO.
3. User-side (outside this repo): optionally give `C:\repo\vm-ssh-mcp` a remote (`git remote add origin ...`) if it should be backed up.
```

Justification for the item-2 removal: the TODO's old item 2 ("optional remaining tests… 17 of 59… still run-only-pending") is factually closed — the full 71-test suite has been executed green during Task 2 — and the same fact is recorded in 4.1/4.2/4.3. No other Immediate Next Steps text changes.

### 4.5 Placeholder substitution (mandatory, after §6 commits)

After the commits of §6 exist, run `git log --oneline -4` on the branch and replace the `<DOC-SHA>` placeholder in the 4.2 Recent Changes bullet with the real short SHA of the public-docs commit ("docs: state the corrected reversal rule and all-green test status"). Edit only that token; do not reword the bullet.

---

## 5. Verification sub-steps (caller — read-only, no cargo/docker)

Doc-only task: no Docker runs and no Cargo execution are required or authorized. After the implementer finishes, the caller runs these textual checks (via the Grep tool or single `git grep` commands) and must get the exact outcomes:

| # | Grep (pattern, paths) | Expected |
| --- | --- | --- |
| V1 | `"immediate reversal into itself is rejected"` → `README.md` | 0 matches (replaced per §1.1) |
| V2 | `"which still rejects immediate reversals"` → `README.md` | 0 matches (per §1.2) |
| V3 | `"Immediate-reversal rejection"` → `README.md` | 0 matches (per §1.3) |
| V4 | `"can never begin a move"` → `README.md`, `docs/terminal-ui.md` | ≥1 match in each file (§1.1, §2.1) |
| V5 | `"90-degree turns still work"` → `README.md` | 1 match (§1.1) |
| V6 | `"authored to fail"` → `docs/testing.md` | ≥1 match (§3.1) |
| V7 | `"Fails until fix"` and `"**fail** until"` → `docs/testing.md` | 0 matches total (§3.1/§3.2) |
| V8 | `"Bug repro (green since 0.3.3)"` → `docs/testing.md` | exactly 5 matches (§3.2) |
| V9 | `"the failing"` → `docs/testing.md` | 0 matches (§3.3) |
| V10 | `"(the domain rejects reversals)"` → `docs/terminal-ui.md` | 0 matches (§2.2) |
| V11 | `"a press that would reverse into itself is ignored"` → `docs/terminal-ui.md` | 0 matches (§2.3) |
| V12 | `"resolve_impossible_reversal"` → `docs/terminal-ui.md`, `.agent/project-info/context.md` | ≥1 match in each file |
| V13 | `"20261006"` → `.agent/project-info/context.md` | ≥3 matches (4.1 focus bullet, 4.2 recent-changes bullet, 4.3 status bullet) |
| V14 | `"17 of 59"` → `.agent/project-info/context.md` | 0 matches (4.4 replacement) |
| V15 | `<DOC-SHA>` → `.agent/project-info/context.md` | 0 matches after sub-step 4.5 runs |
| V16 | `git status` and `git status --porcelain` | clean tree, nothing staged, after §6 |
| V17 | `git check-ignore` on `README.md`, `docs/terminal-ui.md`, `docs/testing.md`, `.agent/project-info/context.md` | none ignored (gitignore-compliance rule satisfied for staging) |

The Markdown-generation rule is respected: this plan is authored only by the planner; the implementer edits the four doc files listed here, nothing else.

---

## 6. Commit plan

Repository commit-message style observed on this branch: lowercase `type: summary`, no trailing period. Human-verification actions preceded by allowed read commands; no chained/sub-command lines.

1. **Commit 0 — this plan file** (work pattern as in prior tasks, e.g. `7224d26`):
   ```text
   git add .kilo/plans/20261006-direction-swap-reversal-docs.md
   git commit -m "docs: add task 3 docs plan for the direction-swap reversal fix"
   ```
2. **Commit 1 — public docs** (after steps 1.1–2.3 and 3.1–3.3 are edited):
   ```text
   git add README.md docs/terminal-ui.md docs/testing.md
   git commit -m "docs: state the corrected reversal rule and all-green test status"
   ```
3. **Commit 2 — project context** (after step 4 edits; run before or after the 4.5 placeholder fill is acceptable, see sub-step order below):
   ```text
   git add .agent/project-info/context.md
   git commit -m "docs: record the direction-swap reversal fix in project context"
   ```
4. **Static conventions per commit:**
   - Run `git status` FIRST; stage ONLY the enumerated files (gitignore-compliance rule).
   - Do NOT amend or rebase any existing commit (the branch stays `27e1e92`-descended; prior commits must never be touched).
   - Do NOT push, do NOT merge, do NOT edit the TODO file (the `[DONE]` cycle belongs to a later workflow step).
   - Commit order: plan file first (Commit 0), then public docs (Commit 1), then project context (Commit 2); fill the `<DOC-SHA>` placeholder (sub-step 4.5) after Commit 1 exists and before Commit 2, so Commit 2 is committed last with the already-substituted token.
   - `dist/` and `target/` are gitignored and remain untouched by this doc-only task; none of the staged files match ignore patterns (verified by V17).
   - If a commit hook or check fails, fix the file, re-stage, and create a NEW commit; never amend the failed commit.

---

## 7. Flags & questions returned to the caller (planner-level read-only observations; NOT implementer actions)

- **Q1 — `.agent/project-structure.md` accuracy rule conflict:** `.kilo/rules/project-structure.md` mandates the structure map "accurately reflect the current project structure", but line 30 still says "**Eight** headless integration test files" and omits `tests/direction_swap_reversal.rs` (Task 1 created it that way; the map is currently out of date). Correcting it is one line-level count change plus one added row, verifiable, but is outside the explicit Task-3 file list. **Planner's recommendation:** approve adding the row/count fix as a fifth sub-step (single line edit + one added row, listed in the caller's TODO 4.1 approval so the restricted implementer acts only under explicit authority). Planner cannot self-authorize this scope expansion, so the caller decides; docs-only change, still no cargo/docker needed.
- **Q2 — README tests-section staleness (out of Task 3's listed sub-section):** line 46 ("Two headless integration test files… All fifteen new tests…") and the "Project Structure" test enumeration at line 75 (`six Phase 1A files plus tests/gameplay_flow.rs and tests/terminal_modules.rs from Phase 1B`) omit `tests/direction_swap_reversal.rs`, and `README.md` does not link `docs/testing.md` anywhere (it does link `docs/BUILD.md` and `docs/terminal-ui.md`). Recommend a small follow-up docs pass fixing the enumeration and adding a `docs/testing.md` cross-link from the README Project-Structure list; both fall outside Task 3's stated README scope (the gameplay/rules section), so they are flagged rather than implemented.
- **Q3 — `docs/terminal-ui.md` Status section staleness:** lines 9–18 claim tests "are authored only" and that the Docker build phase (Phase 2) hasn't produced any execution, which is now false in both directions. Out of Task 3's reversal-rule scope; flagged so a future docs-control task can refresh the status text once and keep this task surgical.
- **Q4 — `tests/direction_swap_reversal.rs` `//!` header** ("…must turn green with the Task 2 fix") reads as a stale task-time note now that Task 2 is done, but source comments are outside the docs workflow; recommend leaving untouched permanently (the comment describes the authoring act, which is historical fact).

The plan as written follows each rule in the caller's prompt: only the four listed doc files are edited; the `docs/testing.md` decision is encoded with justification; the verification sub-steps are pure greps; the commit plan is 1+2 commits (plan file, then docs, then context) with meaningful typed messages and no TODO/`.gitignore` violations; nothing proposes a cargo or Docker run for a doc-only task.

---

## 8. Caller (Planner) decisions on Q1–Q4 — APPROVED SCOPE ADDITIONS

Decided 2026-10-06 by the Planner Agent after user approval of the global plan ("Approve Global and Tasks Plans"). The flag section above is retained verbatim for traceability; the decisions below OVERRIDE its "flagged rather than implemented" defaults for Q1–Q3.

- **Q1 — APPROVED (mandated by `.kilo/rules/project-structure.md`).** The structure map must accurately reflect the current structure. Add to Commit 1's scope: `.agent/project-structure.md` — change "Eight headless integration test files" to "Nine headless integration test files" (verify exact current wording first) and add one row/line listing `tests/direction_swap_reversal.rs` (10 direction-swap regression tests) next to the existing test rows, matching the map's row style.
- **Q2 — APPROVED (factual accuracy; minimal).** Add to Commit 1's scope: `README.md` — (1) update the test enumeration so it includes `tests/direction_swap_reversal.rs` (10 direction-swap reversal tests; total suite count refreshed to the actual 71 test functions across 9 files — verify each number against `tests/` before writing); (2) add a cross-link to `docs/testing.md` in the README's docs/structure list, styled like the existing `docs/BUILD.md` / `docs/terminal-ui.md` links. NO other README prose changes.
- **Q3 — APPROVED (factual accuracy; minimal, surgical).** Add to Commit 1's scope: `docs/terminal-ui.md` Status section (lines ~9–18) — replace ONLY the false claims that tests are "authored only"/never executed with the current facts: the suite is executed in the Alpine VM Docker; full suite green (71 test functions, 0 failed) as of 2026-10-06; keep every other sentence intact. If the section cites counts ("17 of 59"), refresh them to the actual counts (71 functions across 9 files).
- **Q4 — REJECTED (leave untouched).** The `tests/direction_swap_reversal.rs` `//!` header stays as-is: it documents the authoring act (historical fact) and source comments are outside this docs task.
- **Commit message updates:** Commit 1's message becomes `docs: state the corrected reversal rule, refreshed statuses, and test enumeration` (covers rules text + status corrections + enumeration). Commit 0 and Commit 2 unchanged. The V-grep verification list gains: `grep -n "authored only" docs/terminal-ui.md` must return no stale claim rows about never-executed tests; `grep -n "Eight" .agent/project-structure.md` must return no test-count row; README must contain a `docs/testing.md` link.
