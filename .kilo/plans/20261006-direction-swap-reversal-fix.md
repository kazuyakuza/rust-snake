# Implementation Plan — Task 2: Fix the Reversal-Rejection Logic (Quick Swaps Never Reverse Into the Body)

- Date: 2026-10-06
- Workflow step: Task 2, sub-step 4.1b (Analysis & Planning) of the Critical Workflow
- TODO file: `.agent/todos/20261006/20261006-todo-1.md` — Task 2: "Fix the reversal-rejection logic so quick swaps never reverse into the body"
- Global plan: `.kilo/plans/20261006-direction-swap-reversal-bug.md`
- Task 1 plan (tests, now committed): `.kilo/plans/20261006-direction-swap-reversal-failing-tests.md`
- Branch: `fix/circling-reversal-death` (working tree clean at `7b483f6` — verified). Branch creation is NOT part of this task; push is restricted to Step 5.
- Executor of this plan: implementer sub-agent (JUNIOR, 50% restriction) in step 4.2; the CALLER owns all Docker test runs (implementer has PowerShell only, no cargo).
- Task type: NOT front-end related (4.1b only).
- Spec of record: `tests/direction_swap_reversal.rs` (10 tests) + the ENTIRE pre-existing `tests/` suite. Tests are NEVER edited in this task.

---

## 0. Scope Guards (HARD RULES)

1. **The only source file touched is `src/game/state.rs`.** No other file under `src/` may be created or edited (`src/terminal/game_loop.rs` stays byte-identical; the approach requires no terminal-layer change).
2. **`tests/` must NOT be touched.** If any pre-existing test contradicts the fix, STOP and report to the caller instead of editing tests.
3. User-facing docs (README, docs/terminal-ui.md, `.agent/project-info/context.md`) are Task 3 — untouched here. Only doc-comments inside changed `src/` files are updated in this task.
4. Version bump `0.3.2` → `0.3.3` in `Cargo.toml` (workflow step 3 merged into this task per the global plan), committed right after the fix commit.
5. Exactly two source artifacts are committed by this task: the edit of `src/game/state.rs` and the edit of `Cargo.toml` (plus this plan file itself).
6. No branch creation/switch, no push, no TODO-file edit (the `[DONE]` mark belongs to step 4.6).

---

## 1. Research Summary (facts verified against the code)

### 1.1 Current code semantics

`src/game/state.rs` (142 lines, current):

- `change_direction(new_direction) -> bool` rejects ONLY `new_direction == current_direction.opposite()`; otherwise sets `current_direction = new_direction` and returns `true`.
- `advance_one_step()` computes `next_head = head + current_direction.offset()`, then: boundary check → `enter_game_over`; `collision::collides_with_body(next_head, segments)` (tail-aware: `segments[..len-1]`) → `enter_game_over`; else move (+food/score/growth).
- `GameState` fields: `snake: Snake`, `food: Food`, `current_direction: Direction`, `score: i32`, `status: GameStatus`. No last-moved-direction state exists today.

`Snake` (`src/game/snake.rs`): head-first `Vec<Position>`; `segments[0]` = head, `segments[1]` = neck (the cell the head occupied in the previous step). `GameState::advance` always makes the neck the previous head position → invariant: **`head − neck` is exactly one grid step along the direction of the previous actual move** (also true at t=0: init layout head (10,12), neck (9,12) is exactly behind the initial direction Right).

`src/terminal/game_loop.rs`: `tick` applies ALL drained directions via `change_direction`, then calls `advance_one_step()` once. This layer is NOT touched by the fix.

Initial state (`src/game/setup.rs`, verified): head (10,12), neck (9,12), tail (8,12), direction Right, food (20,12); board 80×80.

### 1.2 The bug (confirmed by Task 1's verified Docker run: 5 failed / 5 passed in `tests/direction_swap_reversal.rs`)

A burst of presses drained inside one tick can leave `current_direction` 90°-rotated away from a 180°-of-last-move direction with no single press being rejected: `[Up, Left]` → Left steps onto the neck (9,12); `[Right, Down, Left]` → Left steps onto the neck; tick `[Up]` then tick `[Left, Down]` → Down steps onto the cell the head vacated one tick ago.

### 1.3 Candidate evaluation (final approach LOCKED below; no alternatives left open)

| Candidate | Verdict | Evidence |
|---|---|---|
| (a) `change_direction` rejects any candidate whose offset would move the head onto the neck (**apply-time** neck check) | **REJECTED** | Contradicts FOUR pre-existing tests in `tests/direction.rs` (spec, unmodifiable): `up_changes_to_left` (line 24), `down_changes_to_left` (line 40), `left_changes_to_up` (line 56), `left_rejects_right` (line 88). All of them apply two successive `change_direction` calls with NO step in between (head stays (10,12), neck (9,12)) and assert the second 90° call returns `true` — e.g. `assert!(game.change_direction(Up)); assert!(game.change_direction(Left));` — while an apply-time neck check would reject `Left` (head+Left == neck). Task-1 plan §1.3 decision 1 already noted direction.rs pins the apply-API return semantics. |
| (b) new `last_moved_direction` field validated in `change_direction` (**apply-time** last-moved check) | **REJECTED** | Same contradiction: at its first application the geometry-derived last-moved direction RIGHT makes `Left` rejectable at apply time → the same 4 direction.rs tests break. Its step-time variant is strictly a special case of the locked design. Adding a state field would also need wiring in `GameState::new` and an update inside `advance_one_step` — more state, weaker invariant. |
| (c) loop layer caps one change per tick (`game_loop::apply_directions`) | **REJECTED as the fix; terminal file untouched** | `HeadlessLoop`/domain repro tests drive `change_direction` directly — a loop-layer cap leaves all 3 domain repro tests failing. Also a per-tick cap that counts same-direction no-ops breaks the `[Right, Down]` pin `double_key_turn_within_one_tick_toward_free_cells_still_turns`. Domain fix required regardless; (c) adds nothing the locked design lacks. |
| **FINAL — step-time geometric resolution in the domain** | **LOCKED** | See §2. `change_direction` stays byte-identical; `advance_one_step` re-evaluates the pending direction against head/neck geometry and resolves an impossible pending reversal to the direction of the previous actual move. Zero new fields, zero public API changes, no terminal-layer change. |

Note why the caller's feared trace does NOT occur under the locked design: the caller traced `[Left, Down]` (tick 2) reaching `Down` as the final pending direction stepping onto the neck. The resolution trigger is evaluated at STEP time against the CURRENT geometry — at that moment head (10,11), neck (10,12), so pending `Down` is detected as landing on the neck and is resolved before any step (trace in §4.3).

---

## 2. Final Design (LOCKED — single approach)

### 2.1 Rule (the "segment-2 rule", matching TODO line 23 and 29)

The snake can never begin a move going the direction opposite to the direction it actually moved in the previous move — whichever buffered-drain policy is used. Because the neck cell IS the previous head position, this is pure geometry: no new state field is tracked; the last actually-moved direction is derived from `head − neck` at each step.

### 2.2 Mechanism

1. `GameState::change_direction` — **unchanged** (rejects only `candidate == current_direction.opposite()`). All return-value semantics pinned by `tests/direction.rs` are preserved.
2. `GameState::advance_one_step` — one new line before computing `next_head`: resolve an impossible pending reversal. If `current_direction` equals the opposite of the geometrically-derived last-moved direction (equivalently: `head + current_direction.offset() == neck`), the pending direction is an impossible reversal (buffered bursts can reach it without any single press being rejected by `change_direction`); the domain resolves it to the last-moved direction (continue straight), so the move opposite the previous move never begins.
3. When `snakes` length < 2 (no neck exists) the guard is skipped (`None`) — defensive no-op for degenerate setups; every real path has length ≥ 3.
4. The general self-collision death rule (`brief §8`: head onto own body segment except the vacating tail) is UNCHANGED: the fix removes only the reversal-flip class (`collides_with_body` and boundary checks keep running after the guard).

### 2.3 Key equivalence (why zero new state is sound)

For any snake of length ≥ 2 after a successful move: `neck = previous head = head − last_moved.offset()`, so `last_moved.offset() == head − neck`; conversely at t=0 the initial layout satisfies the same identity with the setup direction (head (10,12), neck (9,12), direction Right ⇒ head−neck = (1,0) = Right.offset()). Therefore `candidate == last_moved.opposite()` ⇔ `head + candidate.offset() == neck`. The geometry check IS the last-moved check. Same-direction re-application consumes nothing per tick (no allowance exists at all), satisfying the `[Right, Down]` pin.

---

## 3. Exact Code Changes — `src/game/state.rs` ONLY

### 3.1 Module doc header (lines 7–8 today) — replace the sentence

Replace:

```rust
//! head, food, direction, score, and `GameStatus`; `advance_one_step` resolves a single
//! move: boundary and self collisions, food consumption, and status transitions.
```

with (wording: keep the existing first part of the sentence intact, only extend the list):

```rust
//! move: the impossible-reversal pending-direction guard, boundary and self
//! collisions, food consumption, and status transitions.
```

### 3.2 New constant (next to the other consts, after `const SCORE_INCREMENT: i32 = 1;`)

```rust
const NECK_SEGMENT_INDEX: usize = 1;
```

### 3.3 New private free function (below `fn is_immediate_reversal`, before the `GameStatus` enum)

```rust
fn direction_from_neck_to_head(neck: Position, head: Position) -> Option<Direction> {
    let (step_x, step_y) = (head.x - neck.x, head.y - neck.y);
    match (step_x, step_y) {
        (1, 0) => Some(Direction::Right),
        (-1, 0) => Some(Direction::Left),
        (0, 1) => Some(Direction::Down),
        (0, -1) => Some(Direction::Up),
        _ => None,
    }
}
```

Semantics: the direction the head actually moved when it stepped from `neck` onto `head` — i.e. the last actually-moved direction. `None` for degenerate geometry (no neck, or neck not one step from the head): in that case no reversal into the neck is possible and the guard must do nothing.

### 3.4 Two new private methods inside `impl GameState` (place them directly after `change_direction`, before `advance_one_step`)

```rust
fn direction_of_last_move(&self) -> Option<Direction> {
    let neck = self.snake.segments().get(NECK_SEGMENT_INDEX).copied()?;
    direction_from_neck_to_head(neck, self.snake.head())
}

/// A buffered burst may leave the pending direction opposite the direction of
/// the previous actual move: no single press was rejected (each was compared
/// against the intermediate pending direction), yet the combined outcome steps
/// straight back onto the body. The pending direction is re-resolved to the
/// last moved direction so that move never begins.
fn resolve_impossible_reversal(&mut self) {
    let Some(last_moved_direction) = self.direction_of_last_move() else {
        return;
    };
    if self.current_direction == last_moved_direction.opposite() {
        self.current_direction = last_moved_direction;
    }
}
```

(If the implementer's rustc rejects `let ... else` style for any reason, use the equivalent early-return form: `if self.direction_of_last_move() ... { return; }` — the caller-approved form is the one above.)

### 3.5 `advance_one_step` — insert the guard call

```rust
pub fn advance_one_step(&mut self) {
    if !self.is_playing() {
        return;
    }
    self.resolve_impossible_reversal();
    let next_head = self.snake.head() + self.current_direction.offset();
    if collision::is_outside_board(next_head) {
        self.enter_game_over();
        return;
    }
    if collision::collides_with_body(next_head, self.snake.segments()) {
        self.enter_game_over();
        return;
    }
    let will_consume = self.food.occupies(next_head);
    self.snake.advance(next_head, !will_consume);
    if will_consume {
        self.score += SCORE_INCREMENT;
        self.respawn_food();
    }
}
```

Everything except the single added `self.resolve_impossible_reversal();` line stays identical.

### 3.6 `change_direction` — body unchanged; doc-comment extended

```rust
/// Steer the snake, rejecting an immediate reversal into itself. Returns
/// `true` when the direction changed, or `false` when `new_direction` is
/// directly opposite the current one (the direction is then left unchanged).
///
/// Presses drained between two steps can still leave the pending direction
/// opposite the last actually-moved direction; `advance_one_step` resolves
/// such an impossible reversal before stepping.
```

### 3.7 Resulting file metrics

- `src/game/state.rs`: 142 → ~170 lines (≤ 200 ✓). Max function body ≤ 50 lines ✓. Indentation depth ≤ 2 ✓. Every new function takes self + ≤ 2 params ✓. All booleans single-section ✓.

---

## 4. Full Trace Tables — how the locked design satisfies every test

Conventions: S0 = initial snake [(10,12),(9,12),(8,12)], pend Right, head (10,12), neck (9,12); `derived last-moved` = direction_from_neck_to_head; V(x,y) notation is shorthand for `Position { x, y }` literals.

### 4.1 Repro `two_key_burst_within_one_tick_must_not_step_onto_the_neck` (currently FAILS → must go GREEN)

| Step | pend before | apply/resolve | pend after | geometry |
|---|---|---|---|---|
| apply Up | Right | Up ≠ Right.opposite()=Left → accept | Up | head+Up=(10,11) not neck — guard not involved at apply time |
| apply Left | Up | Left ≠ Up.opposite()=Down → accept (unchanged API) | Left | (no step yet) |
| advance_one_step | Left | guard: derived last-moved = (head−neck) = (1,0) → Right; Left == Right.opposite() → **RESOLVE pend ← Right** | Right | step: (10,12)→(11,12); body slice [(10,12),(9,12)]; free → move; survive |

Result: alive, length 3, score 0, no overlap → GREEN.

### 4.2 Repro `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck` (currently FAILS → GREEN)

| Step | pend before | apply/resolve | pend after |
|---|---|---|---|
| apply Right | Right | Right ≠ Left → accept (no-op reapply, consumes nothing) | Right |
| apply Down | Right | Down ≠ Left → accept | Down |
| apply Left | Down | Left ≠ Up → accept | Left |
| advance_one_step | Left | guard: derived last-moved = Right; Left == Right.opposite() → **RESOLVE ← Right** | Right |

Step: (10,12)→(11,12) free → survive → GREEN.

### 4.3 Repro `two_tick_interleaved_burst_must_not_reenter_the_body` (currently FAILS → GREEN) — the caller's flagged case, traced tick by tick

Tick 1: `apply_and_step(&mut game, &[Direction::Up])`
| Step | pend before | apply/resolve | pend after | geometry |
|---|---|---|---|---|
| apply Up | Right | Up ≠ Left → accept | Up | — |
| advance_one_step | Up | guard: derived = Right; Up ≠ Left → no resolution | Up | step (10,12)→(10,11); body slice [(10,12),(9,12)]; free → move |

State after tick 1: snake [(10,11),(10,12),(9,12)], pend Up, head (10,11), **neck (10,12)**, derived last-moved = (head−neck) = (0,−1) → Up. `assert_snake_survived` passes.

Tick 2: `apply_and_step(&mut game, &[Direction::Left, Direction::Down])`
| Step | pend before | apply/resolve | pend after | geometry |
|---|---|---|---|---|
| apply Left | Up | Left ≠ Down → accept | Left | (no step yet — head still (10,11)) |
| apply Down | Left | Down ≠ Right → accept | Down | (no step yet) |
| advance_one_step | Down | guard: derived last-moved = Up; Down == Up.opposite() → **RESOLVE pend ← Up** | Up | step (10,11)→(10,10); body slice [(10,11),(10,12)]; free → move |

Result: alive, length 3, score 0, no overlap → GREEN. The pending `Down` (which would have stepped onto the neck (10,12)) never begins — this is exactly the caller's required "final-applied-direction validated at the step with fallback" branch, proven sufficient.

### 4.4 Pin `three_key_burst_not_ending_in_reversal_stays_alive` (currently PASSES → must stay GREEN)

apply Up (accept, pend Up) → apply Left (accept, pend Left) → apply Down (accept, pend Down) → guard: derived = Right; Down ≠ Left → no resolution → step (10,12)→(10,13); body slice [(10,12),(9,12)] → free → move. **Final direction Down, head (10,13) — free cell — exactly the caller's described expected outcome.** Alive → GREEN.

### 4.5 Pin `one_turn_per_tick_circles_back_to_the_start_cell` (currently PASSES → GREEN)

| Tick | dir buffered | guard check (derived last-moved → opposite) | resolution | step | snake after |
|---|---|---|---|---|---|
| 1 | Up | Right → Left; Up ≠ Left | none | (10,12)→(10,11) | [(10,11),(10,12),(9,12)] |
| 2 | Left | Up → Down; Left ≠ Down | none | (10,11)→(9,11) | [(9,11),(10,11),(10,12)] |
| 3 | Down | Left → Right; Down ≠ Right | none | (9,11)→(9,12) | [(9,12),(9,11),(10,11)] |
| 4 | Right | Down → Up; Right ≠ Up | none | (9,12)→(10,12) | [(10,12),(9,12),(9,11)] |

Every single turn is 90°; the guard never triggers; head returns to (10,12) = INITIAL_HEAD → GREEN.

### 4.6 Pin `right_angle_turn_between_ticks_still_turns` (currently PASSES → GREEN)

tick1 (empty burst): guard no-op (pend Right ≠ derived-opposite Left) → step to (11,12) ✓. tick2 [Down]: apply Down accept; guard: derived = Right; Down ≠ Left → none → step (11,12)→(11,13) ✓; `current_direction == Down` ✓; head == INITIAL_HEAD + Right.offset() + Down.offset() ✓ → GREEN.

### 4.7 Pin `double_key_turn_within_one_tick_toward_free_cells_still_turns` (currently PASSES → GREEN)

apply Right (no-op reapply, NOT counted as any change — no allowance exists in the design) → apply Down accept → guard: derived = Right; Down ≠ Left → none → step (10,12)→(10,13) ✓; `current_direction == Down` ✓; head == INITIAL_HEAD + Down.offset() ✓ → GREEN.

### 4.8 Pin `loop_tick_survives_continuous_circling_for_two_revolutions` (currently PASSES → GREEN)

Same 8 single-direction ticks as §4.5 plus the same 4 again. Traced (t5–t8): Up → (10,11); Left → (9,11); Down (derived Up, no trigger) → (9,12); Right (derived Down... note after t7 the neck is (9,11), derived = (9,12)−(9,11) head−neck = (0,1) → Down; Right ≠ Up → none) → (10,12). All survive; final head (10,12) ✓ → GREEN. (Each tick's guard input remains a 90° turn — the per-tick circling contract is untouched.)

### 4.9 Loop repros `loop_tick_survives_a_two_key_burst_in_one_tick` / `loop_tick_survives_a_three_key_burst_ending_in_reversal` (currently FAIL → GREEN)

`tick` applies the burst through `change_direction` (loop file untouched), then `advance_one_step` — identical domain outcomes to §4.1 / §4.2: pend Left resolved to Right, step (11,12), status Playing → GREEN.

### 4.10 Pre-existing suite compatibility

- `tests/direction.rs` (13 tests): untouched API, all green (`left_rejects_right` etc. exercise only `change_direction`, never `advance_one_step` with a mismatched pend).
- `tests/movement_and_growth.rs`, `tests/collision.rs`, `tests/food_consumption_scoring.rs`, `tests/food_placement.rs`, `tests/initial_state.rs`: straight-Right movement, boundary, pure predicates — no pending reversal is ever formed; all green.
- `tests/terminal_modules.rs` (11): `tick_applies_directions_before_one_step_and_rejects_reversal` applies [Up, Down] → Down rejected by the unchanged apply-time rule → pend Up → step (10,11) unchanged ✓; others use empty/Down bursts ✓. `vertical_snake_game` (2-segment snake, dir Right, neck (10,13)) is RENDER-ONLY — never stepped, so the derived-direction geometry never runs on it ✓.
- `tests/gameplay_flow.rs` (4): hooked 5-segment snake — traces: tick[] (11,10) → tick[Up] → (11,9) → tick[Left] → (10,9) all 90° turns, no guard trigger; fatal tick[Down]: derived = Left (neck (11,9)); Down ≠ Right → none → lands (10,10) ∈ body slice → GameOver with head (10,9) — all assertions unchanged ✓; boundary and entity-eating flows unaffected ✓.

TOTAL expected: 59 pre-existing + 10 new = 69 test functions, ALL GREEN.

---

## 5. Implementation Steps (for the implementer, step 4.2) — execute in order, STOP at first failure

1. **Verify repo state.** `git status` → must be clean; `git branch --show-current` → must be `fix/circling-reversal-death`. `git status --porcelain src tests Cargo.toml` → must be empty. Otherwise STOP and report.
2. **Commit this plan file** (artifact of 4.1b, already saved by the architector): `git add .kilo/plans/20261006-direction-swap-reversal-fix.md` then `git commit -m "docs: add implementation plan for the direction-swap reversal fix"`.
3. **Re-read `src/game/state.rs`** before editing (edit tool requires a prior read).
4. **Edit `src/game/state.rs`** exactly per §3: module-header sentence, `NECK_SEGMENT_INDEX` const, `direction_from_neck_to_head`, `direction_of_last_move`, `resolve_impossible_reversal`, the added call in `advance_one_step`, and the extended `change_direction` doc-comment. No other function, field, or file changes. Keep `is_immediate_reversal` as-is.
5. **Self-check** the edited file: line count ≤ 200; no commented-out code; every name self-documenting; `cargo`-free formatting sanity (no trailing whitespace, 4-space indent). If any check fails, fix formatting only, never semantics.
6. **Report to the caller** requesting the Docker verification runs (§6). Do NOT commit `src/` yet — commits happen after the caller confirms the runs (mirrors Task-1's verified order).
7. **After the caller confirms BOTH Docker runs are green** (§6.3/§6.4), commit the fix: `git add src/game/state.rs`, `git commit -m "fix: never begin a move opposite the last moved direction"`.
8. **Version bump**: edit `Cargo.toml` line `version = "0.3.2"` → `version = "0.3.3"` (single field, nothing else). Then `git add Cargo.toml`, `git commit -m "chore: bump version to 0.3.3"`. (If `Cargo.lock` contains the old `snake` version line — `name = "snake"` / `version = "0.3.2"` package entry — regenerate-dependent behavior: do NOT hand-edit `Cargo.lock`; report to the caller instead; a host-side cargo is not available and the Docker test runs may rewrite it — leave `Cargo.lock` untouched by git stance: check `git status`; stage `Cargo.lock` ONLY if the Docker run rewrote it and it appears modified, committing it with the bump commit message.)
9. **STOP.** The implementer does NOT perform steps 4.3+ (review/simplification), 4.4 (further docs), 4.5/4.6 (adherence, `[DONE]`), or Step 5 (merge/push).

## 6. Test Verification (CALLER executes via `alpine-vm` MCP; implementer has no cargo)

### 6.1 Precondition

`vm_status` must report the VM running + SSH reachable; if down, PAUSE and ask the user to start it. All commands use the exact template proven in Task 1 (`.kilo/plans/20261006-direction-swap-reversal-failing-tests.md` §4.2): prefix `docker` (allowlisted), `/rust-snake` shared folder, Linux host target, `CARGO_TARGET_DIR=/tmp/target`, single command, generous timeout (≥ 600000 ms; first run re-downloads `rand`/`crossterm` per lockfile).

### 6.2 Read-only real check before runs (optional but recommended)

```text
cat /rust-snake/src/game/state.rs
```

Confirm the guard is present (`resolve_impossible_reversal` called before `next_head`) before spending a test run.

### 6.3 Run 1 — the 5 bug repros (REQUIRED FIRST)

```text
docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal
```

Expected: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — the 5 previously failing repros (§4.1–4.3, §4.9 of this plan) go green WITHOUT touching the tests; the 5 pins stay green.

### 6.4 Run 2 — FULL suite (REQUIRED SECOND; only if Run 1 is fully green)

```text
docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test
```

Expected: every test passes — 59 pre-existing `#[test]` functions across the 8 pre-existing files + 10 new in `tests/direction_swap_reversal.rs` = 69 test functions total, `test result: ok` in each target. This is the project's first-ever full-suite green run.

### 6.5 Failure-handling rules (HARD)

- If ANY test fails in Run 1: STOP. Do NOT edit `tests/`. Report the failing names + panic output to the caller for analysis against §4 traces (a trace step divergence must be identified by the architector).
- If a PRE-EXISTING test fails in Run 2 (any file other than the 10-new): STOP AND REPORT — per the caller's explicit instruction, do not edit tests to make a contradicted expectation pass. The fix design depends on the traces of §4.10 holding; a contradiction means an assumption is wrong and an architector decision is required.
- If compilation fails: STOP and report (implementation error — re-derive against §3 exactly).
- Warnings from `cargo test` on `src/`: none expected; any `dead_code` warning on the new helpers would indicate an edit was placed incorrectly — treat as a failure (STOP and report).

---

## 7. Git Summary (step-4.2 scope only; push restricted to Step 5 of the workflow)

| # | Action | Message |
|---|---|---|
| 1 | `git add .kilo/plans/20261006-direction-swap-reversal-fix.md` | `docs: add implementation plan for the direction-swap reversal fix` |
| 2 | `git add src/game/state.rs` | `fix: never begin a move opposite the last moved direction` |
| 3 | `git add Cargo.toml` (+ `Cargo.lock` only if the Docker run modified it — verify `git status` per the gitignore-compliance rule; `target/`/`dist/` must never be staged) | `chore: bump version to 0.3.3` |

Before each commit: run `git status`, re-read `.gitignore` (gitignore-compliance rule), stage ONLY the intended files (`git add <explicit paths>`; never `git add .`), confirm unstaged/untracked noise stays untracked.

---

## 8. Rules Compliance Checklist (self-check before the fix commit)

- keep-it-simple (brief §19): two small private helpers + one call site; no new fields, no new pub API, no modules.
- Domain centralization: all logic in `src/game/state.rs`; `src/terminal/` byte-identical.
- max 2 params: `direction_from_neck_to_head(neck, head)` = 2; methods take only self ✓.
- max 200 lines/file: ~170 ✓; max depth 2 ✓; single-section booleans ✓; self-documenting names ✓; minimal comments (short doc-comments on the new private items and the extended doc comments — no inline comments) ✓; no commented-out code ✓.
- Gitignore compliance: only `src/game/state.rs`, `Cargo.toml`, (conditionally) `Cargo.lock`, and the plan file ever staged; `target/`, `dist/` never staged ✓.

---

## 9. Verification Checklist vs TODO Task 2 (one-to-one)

| TODO Task 2 requirement (/ caller constraints) | Covered by |
|---|---|
| Snake can never end up moving opposite its actual last-moved direction, no matter how many swaps are buffered/applied within a tick | §2.1–§2.2 guard; traces §4.1–4.3, §4.9 |
| Without breaking legit per-tick double-turns like `Right→Down` | §4.7: pend Down survives untouched, dir/head asserts hold; same-direction no-op consumes nothing (§2.3) |
| Input between ticks remains allowed; `right_angle_turn_between_ticks_still_turns` | §4.6 |
| Reversal rejection stays centralized in the domain (`src/game/`) | All logic in `src/game/state.rs`; no nesting across layers (§2.2) |
| Terminal layer unchanged where possible | `src/terminal/game_loop.rs` byte-identical (§0.1, §5.4) |
| Circling over 8 ticks (2 revolutions) + head returns to (10,12) | §4.5, §4.8 |
| Re-run new tests until green; full suite must pass (all previously authored tests pass) | §6.3 (10/10), §6.4 (69/69); STOP rules §6.5 |
| Commit the fix | §7 commit 2 |
| Version bump 0.3.2 → 0.3.3 with/after the fix | §5.8, §7 commit 3 |
| Docs: src doc-comments only | §3.1, §3.6; user-facing docs deferred to Task 3 |

---

## 10. Risk Notes

1. **Trace-vs-implementation divergence** (highest risk): if any of §4's steps doesn't hold for the implementer's code, the caller must NOT "adjust" tests; the architector re-traces (likely a guard placement or `copied()?` mistake — compare against §3 verbatim).
2. **Cargo.lock version coupling**: the bump commit may leave `Cargo.lock`'s `snake` version stale until the next Docker build rewrites it. Rule §5.8 governs the only acceptable staging; anything beyond that goes back to the caller.
3. **VM availability**: if `vm_status` fails, pause and ask the user to start the VM (global-plan note); no local fallback (host has no cargo).
4. **Scope of the guard**: the fix removes only the reversal-flip death class; tight-coil self-collisions remain legal death outcomes (brief §8) and are NOT covered by these 10 tests/tests — no general "body scan" rule is added.
