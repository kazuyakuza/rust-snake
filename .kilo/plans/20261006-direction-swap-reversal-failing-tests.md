# Implementation Plan — Task 1: Failing Tests Reproducing the Circling-Reversal-Death Bug

- Date: 2026-10-06
- Workflow step: Task 1, sub-step 4.1b (Analysis & Planning) of the Critical Workflow
- TODO file: `.agent/todos/20261006/20261006-todo-1.md` — Task 1: "Write failing tests that reproduce the circling-death bug"
- Global plan: `.kilo/plans/20261006-direction-swap-reversal-bug.md`
- Branch: `fix/circling-reversal-death` (already created in Step 2; branch creation is NOT part of this task)
- Executor of this plan: implementer sub-agent (JUNIOR, 50% restriction) in step 4.2
- Task type: NOT front-end related → no frontend spec involved

---

## 0. Scope Guards (HARD RULES)

1. **`src/` must NOT be touched in Task 1.** No file under `src/`, no `Cargo.toml`, no `.cargo/config.toml`, no existing test file may be modified. The ONLY code artifact of Task 1 is ONE new file: `tests/direction_swap_reversal.rs`.
2. The tests must compile against the CURRENT (buggy) code and assert the CORRECT (post-fix) behavior, so they MUST FAIL when run now.
3. No version bump in Task 1 (the 0.3.2 → 0.3.3 bump is merged into Task 2's 4.2 per the global plan).
4. No user-facing docs changes in Task 1 (README / docs/terminal-ui.md / context.md are Task 3's scope).
5. The implementer commits exactly two artifacts (see §5): the plan file and the test file. Nothing else.

---

## 1. Verified Facts & Corrections (read before anything else)

### 1.1 CORRECTION — initial snake coordinates (flagged to the caller)

The task prompt claimed the initial snake is head (41,40), neck (40,40), tail (39,40). That is WRONG. The verified source of truth is `src/game/setup.rs` (lines 9–13):

```rust
const INITIAL_SNAKE_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_BODY_AHEAD: Position = Position { x: 9, y: 12 };
const INITIAL_SNAKE_BODY_BEHIND: Position = Position { x: 8, y: 12 };
const INITIAL_DIRECTION: Direction = Direction::Right;
const INITIAL_FOOD_POSITION: Position = Position { x: 20, y: 12 };
```

All coordinates in the tests are therefore: **head (10,12), neck (9,12), tail (8,12), facing Right, food (20,12)**. This matches the coordinates already used by the existing suites (`tests/gameplay_flow.rs` line 74: `Position { x: 10, y: 12 }`, `tests/terminal_modules.rs` line 83: `Position { x: 10, y: 11 }` after one Up step).

### 1.2 Geometric analysis of each requested scenario (against the real coordinates)

Self-collision predicate (`src/game/collision.rs`): the head dies only when `next_head` lands on a segment in `segments[..len-1]` (head or neck for a 3-segment snake; the tail cell is excluded because it vacates in the same step). For the initial snake the only lethal cell reachable in one step from (10,12) is the **neck (9,12)**, reached exactly when the applied final direction is **Left** (opposite of the last actually-moved direction, Right).

| Scenario | Trace | Result on current code | Role in test file |
|---|---|---|---|
| One-tick burst `[Up, Left]` | Up accepted (current=Up); Left accepted (current=Left); step (10,12)→(9,12) = neck → GameOver | **FAILS (dies)** | Bug repro (required) |
| One-tick burst `[Right, Down, Left]` (triple ending in reversal) | Right no-op accept; Down accepted; Left accepted (Down.opposite()=Up) → current=Left; step → (9,12) = neck → GameOver | **FAILS (dies)** | Bug repro (replaces the broken literal scenario (b) as the genuine triple-burst death) |
| Two-tick interleave: tick 1 `[Up]`, tick 2 `[Left, Down]` | t1: step to (10,11); t2: Left accepted, Down accepted → current=Down; step (10,11)→(10,12) = pre-move neck → GameOver | **FAILS (dies at t2)** | Bug repro |
| One-tick burst `[Up, Left, Down]` (the prompt's literal scenario (b)) | current=Down after burst; step (10,12)→(10,13); body slice is [(10,12),(9,12)] → free cell → survives | **PASSES (survives)** | Survival pin only — it does NOT reproduce the death with the real horizontal snake (with the prompt's own (41,40) example, stepping Down to (41,41) is also off-body). Kept because the TODO names the sequence; it pins correct behavior. |
| Full circle one turn per tick: `[Up]`,`[Left]`,`[Down]`,`[Right]` over 4 ticks | head: (10,11) → (9,11) → (9,12) → (10,12); every step targets a vacated/free cell | **PASSES (survives)** | Sanity pin (TODO explicitly expects this to pin the good behavior) |
| Plain turn between ticks: `[Right]` (empty burst is equivalent) then `[Down]` | head: (11,12) → (11,13); current=Down | **PASSES** | Anti-regression pin: normal 90° turns must keep working after the fix |
| One-tick double-key turn `[Right, Down]` (heads to a FREE cell) | current=Down; step (10,12)→(10,13), free | **PASSES** | Requirement pin: legit per-tick double turns remain allowed (global-plan constraint for Task 2) |
| Loop-level same bursts via `game_loop::tick` | `tick` applies all directions, advances one step, returns `state.status()` → `GameOver` for the two burst repros | **2 cases FAIL** | Bug repro at the loop layer |

Post-fix compatibility check (both candidate fix approaches from the global plan — (a) neck-geometry rejection in `change_direction`, (b) cap of one EFFECTIVE accepted change per tick): every failing case above survives under BOTH approaches, and every pin's asserted values are identical under BOTH approaches. The tests therefore encode exactly the required contract without coupling to the fix's internals.

### 1.3 Design decisions encoded for the test file (NON-NEGOTIABLE)

1. **Never assert `change_direction`'s `bool` return** in the new file. The fix may reject a candidate at apply-time or at move-time; only survival matters here. (`tests/direction.rs` already pins the current API return semantics; Task 2 will update those if needed.)
2. **Never assert the final head position/direction in fix-dependent cases** (all three failing repros and the `[Up, Left, Down]` pin): the post-fix position legitimately differs between fix approaches (e.g. `[Up, Left, Down]` ends at (10,11) under a cap fix but (10,13) under neck geometry). Assert only: status, length, score, segment uniqueness.
3. **Assert exact positions ONLY in the fully-stable pins**: circling final head (10,12), plain turn positions (11,12)/(11,13), and the `[Right, Down]` turn (current_direction==Down, head (10,13)). The last one intentionally constrains Task 2: a same-direction no-op call (Right while already Right) must NOT consume the per-tick turn allowance, otherwise "Right→Down remains allowed" breaks.
4. The domain-level tests replicate the loop's per-tick behavior manually: apply every buffered direction via `change_direction` (ignoring returns), then call `advance_one_step` once — exactly what `src/terminal/game_loop.rs::apply_directions` + `tick` do.
5. Loop-level tests go through the real `snake::terminal::game_loop::tick` with a `Renderer` over an in-memory `Vec<u8>` buffer (constructor verified: `Renderer::new(&mut buffer)` — the same proven pattern as `tests/gameplay_flow.rs` lines 68–72 and `tests/terminal_modules.rs` lines 74–80; `&mut Vec<u8>` implements `io::Write`).
6. `start_playing()` MUST be called before any step/tick — `advance_one_step` is a no-op while `WaitingToStart` (proven by `tests/movement_and_growth.rs::advance_does_nothing_before_playing`).
7. No food interactions occur anywhere in the file: the food sits at (20,12) and no traced path touches it, so `score() == 0` and length stays 3 in every test.

### 1.4 Conventions extracted from the existing suites

- Imports go through the library crate root: `use snake::game::direction::Direction;` etc. (`tests/direction.rs`, `tests/movement_and_growth.rs`).
- A `playing_game()` helper (name taken verbatim from `tests/movement_and_growth.rs` lines 13–17) builds `GameState::new(initial_setup())` + `start_playing()`.
- `Position` literals are written struct-by-struct (`Position { x: 10, y: 12 }`).
- Exactly ONE module doc-comment header (//!) at the top; no other comments anywhere in the file (`.kilo/rules/self-documenting-code.md`, `.kilo/rules/no-commented-code.md`).
- snake_case test names, 4-space indent, no trailing whitespace. File must stay ≤ 200 lines (`.kilo/rules/max-lines-per-file.md`).

---

## 2. Test File Design — `tests/direction_swap_reversal.rs`

### 2.1 Complete content specification

The implementer writes the file exactly as specified below. The snippets are the authoritative structure; minor local formatting (line wrapping of already-shown lines) is the only latitude granted.

**Module header (lines 1–5):**

```rust
//! Regression tests for the rapid-direction-swap reversal bug: several
//! arrow presses drained inside ONE tick must never steer the head onto
//! its own body. The failing tests reproduce the bug against the unfixed
//! domain and loop layers and must turn green with the Task 2 fix.
```

**Imports and constants (exactly these six `use` lines and two consts):**

```rust
use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;

const INITIAL_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_LENGTH: usize = 3;
```

**Shared helpers (placed directly after the constants):**

```rust
fn playing_game() -> GameState {
    let mut game = GameState::new(initial_setup());
    game.start_playing();
    game
}

fn apply_and_step(game: &mut GameState, directions: &[Direction]) {
    for &direction in directions {
        game.change_direction(direction);
    }
    game.advance_one_step();
}

fn assert_snake_survived(game: &GameState, context: &str) {
    assert_ne!(
        game.status(),
        GameStatus::GameOver,
        "{context}: the snake must stay alive"
    );
    assert_eq!(game.snake().length(), INITIAL_SNAKE_LENGTH, "{context}: length unchanged");
    assert_eq!(game.score(), 0, "{context}: score unchanged");
    let segments = game.snake().segments();
    for (index, segment) in segments.iter().enumerate() {
        assert!(
            !segments[..index].contains(segment),
            "{context}: segment {index} overlaps an earlier segment"
        );
    }
}
```

Rules compliance of the helpers: 2 params each (max-arguments rule); ≤ 50 lines; nesting depth 2 (fn → for → assert); no complex boolean conditions; self-documenting names.

**Domain-level tests (7 `#[test]` functions, in this exact order):**

```rust
#[test]
fn two_key_burst_within_one_tick_must_not_step_onto_the_neck() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up, Direction::Left]);
    assert_snake_survived(&game, "burst [Up, Left] inside one tick");
}

#[test]
fn three_key_burst_ending_in_reversal_must_not_step_onto_the_neck() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Right, Direction::Down, Direction::Left]);
    assert_snake_survived(&game, "burst [Right, Down, Left] inside one tick");
}

#[test]
fn two_tick_interleaved_burst_must_not_reenter_the_body() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up]);
    assert_snake_survived(&game, "after the [Up] tick");
    apply_and_step(&mut game, &[Direction::Left, Direction::Down]);
    assert_snake_survived(&game, "after the [Left, Down] burst tick");
}

#[test]
fn three_key_burst_not_ending_in_reversal_stays_alive() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up, Direction::Left, Direction::Down]);
    assert_snake_survived(&game, "burst [Up, Left, Down] inside one tick");
}

#[test]
fn one_turn_per_tick_circles_back_to_the_start_cell() {
    let mut game = playing_game();
    let circling_turns = [
        [Direction::Up],
        [Direction::Left],
        [Direction::Down],
        [Direction::Right],
    ];
    for turn in &circling_turns {
        apply_and_step(&mut game, turn);
        assert_snake_survived(&game, "while circling one turn per tick");
    }
    assert_eq!(game.snake().head(), INITIAL_HEAD);
}

#[test]
fn right_angle_turn_between_ticks_still_turns() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[]);
    assert_eq!(game.snake().head(), INITIAL_HEAD + Direction::Right.offset());
    apply_and_step(&mut game, &[Direction::Down]);
    assert_eq!(
        game.snake().head(),
        INITIAL_HEAD + Direction::Right.offset() + Direction::Down.offset()
    );
    assert_eq!(game.current_direction(), Direction::Down);
    assert_snake_survived(&game, "after a plain turn between ticks");
}

#[test]
fn double_key_turn_within_one_tick_toward_free_cells_still_turns() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Right, Direction::Down]);
    assert_eq!(game.current_direction(), Direction::Down);
    assert_eq!(game.snake().head(), INITIAL_HEAD + Direction::Down.offset());
    assert_snake_survived(&game, "after the [Right, Down] one-tick turn");
}
```

**Loop-level section — driver struct then 3 `#[test]` functions:**

```rust
struct HeadlessLoop {
    state: GameState,
    buffer: Vec<u8>,
}

impl HeadlessLoop {
    fn new() -> HeadlessLoop {
        let mut state = playing_game();
        HeadlessLoop { state, buffer: Vec::new() }
    }

    fn tick(&mut self, directions: &[Direction]) -> GameStatus {
        let mut renderer = Renderer::new(&mut self.buffer);
        tick(&mut self.state, directions, &mut renderer).expect("headless tick succeeds")
    }
}

#[test]
fn loop_tick_survives_a_two_key_burst_in_one_tick() {
    let mut game = HeadlessLoop::new();
    let status = game.tick(&[Direction::Up, Direction::Left]);
    assert_ne!(
        status,
        GameStatus::GameOver,
        "loop tick [Up, Left]: should not lose while swapping quickly in one tick"
    );
    assert_snake_survived(&game.state, "loop tick burst [Up, Left]");
}

#[test]
fn loop_tick_survives_a_three_key_burst_ending_in_reversal() {
    let mut game = HeadlessLoop::new();
    let status = game.tick(&[Direction::Right, Direction::Down, Direction::Left]);
    assert_ne!(
        status,
        GameStatus::GameOver,
        "loop tick [Right, Down, Left]: should not lose while swapping quickly in one tick"
    );
    assert_snake_survived(&game.state, "loop tick burst [Right, Down, Left]");
}

#[test]
fn loop_tick_survives_continuous_circling_for_two_revolutions() {
    let mut game = HeadlessLoop::new();
    let circling_turns = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];
    for &turn in &circling_turns {
        let status = game.tick(&[turn]);
        assert_ne!(
            status,
            GameStatus::GameOver,
            "loop tick circling turn {turn:?}: should not lose while circling"
        );
        assert_snake_survived(&game.state, "loop tick circling");
    }
    assert_eq!(game.state.snake().head(), INITIAL_HEAD);
}
```

Notes on the loop section:
- `HeadlessLoop` bundles the state + render buffer so no function exceeds 2 params. `Renderer::new(&mut self.buffer)` is the proven `&mut Vec<u8>` pattern.
- `tick` inside `impl HeadlessLoop` calls the imported free function `snake::terminal::game_loop::tick` — no ambiguity (methods resolve only via dot syntax).
- The double revolution (8 ticks = 4+ full direction swaps forming a loop) satisfies the TODO's loop-level "continuous circling" requirement.

### 2.2 Expected file metrics

- Exactly 10 `#[test]` functions (7 domain-level + 3 loop-level).
- ≈ 177 lines → under the 200-line limit.
- Zero comments other than the 5-line //! module header.
- Zero new dependencies (crossterm not needed here; `rand`/`crossterm` stay untouched in `Cargo.toml`).

---

## 3. Step-by-Step Implementation Steps (for the implementer, step 4.2)

Execute in order; STOP at the first step that fails and report back to the caller.

1. **Verify repo state.** Run `git status` (expect clean) and `git branch --show-current` (expect `fix/circling-reversal-death`). If not clean or not on that branch: STOP and report.
2. **Read `.gitignore`.** Confirm `target/` and `dist/` are ignored and nothing matches a new `tests/direction_swap_reversal.rs` or `.kilo/plans/20261006-direction-swap-reversal-failing-tests.md`.
3. **Confirm scope.** Verify `src/` contains no uncommitted changes (`git status --porcelain src` must output nothing). No file under `src/` may be created/edited in this task.
4. **Write the test file** `tests/direction_swap_reversal.rs` exactly per §2.1. Do NOT create any other file.
5. **Self-check the file** against §2.2: 10 tests, ≤ 200 lines, only the //! header, the six imports all used, snake_case names. Fix formatting only (wrapping), never semantics.
6. **Check for a local toolchain.** Run `cargo --version`.
   - If cargo is NOT available (EXPECTED on this host): do NOT attempt any build/test; skip to step 9 and note "local verification impossible — caller must run the Docker verification" in the report.
   - If cargo IS available: continue with steps 7–8.
7. (Only with local toolchain) **Compile-check:** `cargo test --test direction_swap_reversal --no-run`. Must exit 0 with no warnings. On any compile error: fix the file to match §2.1 exactly and retry once; if still failing, STOP and report.
8. (Only with local toolchain) **Run the target:** `cargo test --test direction_swap_reversal`. Expected result: 5 passed / 5 failed, with the failure names from §4.3. Record the output verbatim for the report.
9. **Report back** to the caller: file created, self-check results, whether local verification ran, and (if run) the exact pass/fail breakdown. The CALLER owns the Docker/VM verification (§4.2) because the implementer has no access to the `alpine-vm` MCP.
10. **Commit** per §5 (only after the caller confirms the Docker run reproduced the failures, OR immediately if the caller instructs to commit before verification).

---

## 4. Execution & Verification (test runs)

### 4.1 Ownership split (EXPLICIT — from the caller's task instructions)

- The implementer sub-agent does NOT have the `alpine-vm` MCP and runs on PowerShell only. The implementer runs `cargo test --test direction_swap_reversal` ONLY if a local toolchain exists (§3 step 6). Otherwise the implementer stops after writing + self-checking the file and reports back.
- The CALLER (Planner) then executes the Docker verification below through the `alpine-vm` MCP (`vm_status` first, then `vm_run_command`).

### 4.2 Docker commands (for the CALLER, via `alpine-vm` MCP)

Precondition: `vm_status` reports the Alpine VM running and SSH reachable. If the VM is down: PAUSE and ask the user to start it (global-plan note).

Single test-target verification run (the required Task 1 run):

```text
docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal
```

Command notes (all verified against the repo):
- Prefix `docker` — within the `alpine-vm` MCP allowlist; single command, no chaining.
- `rust:1.98.1-slim-bookworm` is the image pinned by `Dockerfile`; it is already cached in the VM (the compose build used it). If it must be pulled, network access from the VM is proven (2026-10-05 runs).
- `/rust-snake` is the VM-side path of the shared host→VM project folder; mounting it into a container is proven (the 2026-10-05 release build mounted the same folder via compose volumes `.:/project`).
- Tests run on the Linux HOST target (no mingw/windows target needed for `cargo test`) — the same way the 2026-10-05 run executed 17 tests green.
- `CARGO_TARGET_DIR=/tmp/target` matches the compose convention: Linux build artifacts stay inside the container, never in the shared checkout.
- First run downloads `rand 0.8` / `crossterm 0.29` per `Cargo.lock`; allow a generous MCP timeout (≥ 600000 ms). Re-runs repeat the download (ephemeral container) — acceptable.
- Optional compile-only variant (if a fast syntax check is wanted): append `--no-run` to the same command.

Full-suite command (REFERENCE ONLY — NOT part of Task 1; Task 2 runs it):

```text
docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test
```

### 4.3 Expected outcome of the Task 1 run (MUST-FAIL confirmation)

Compilation: succeeds, exit code 0 from cargo (the test binary builds; failures are assertion failures, not compile errors). No warnings expected (all six imports are used).

Test result summary line (shape):

```text
test result: FAILED. 5 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
```

The 5 names listed under `failures:` must be exactly:

```text
    loop_tick_survives_a_three_key_burst_ending_in_reversal
    loop_tick_survives_a_two_key_burst_in_one_tick
    three_key_burst_ending_in_reversal_must_not_step_onto_the_neck
    two_key_burst_within_one_tick_must_not_step_onto_the_neck
    two_tick_interleaved_burst_must_not_reenter_the_body
```

Per-test panic shapes (assertion messages as encoded in §2.1):

- `two_key_burst_within_one_tick_must_not_step_onto_the_neck` panics with
  `burst [Up, Left] inside one tick: the snake must stay alive` and
  `assertion \`left != right\` failed` / `left: GameOver` / `right: GameOver`.
- `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck` panics with
  `burst [Right, Down, Left] inside one tick: the snake must stay alive` (same GameOver/GameOver shape).
- `two_tick_interleaved_burst_must_not_reenter_the_body` survives the first assert and panics on the second with
  `after the [Left, Down] burst tick: the snake must stay alive` (same GameOver/GameOver shape).
- `loop_tick_survives_a_two_key_burst_in_one_tick` panics with
  `loop tick [Up, Left]: should not lose while swapping quickly in one tick` (GameOver/GameOver shape).
- `loop_tick_survives_a_three_key_burst_ending_in_reversal` panics with
  `loop tick [Right, Down, Left]: should not lose while swapping quickly in one tick` (GameOver/GameOver shape).

The 5 passing pins (must be listed as `ok`):

```text
    double_key_turn_within_one_tick_toward_free_cells_still_turns
    loop_tick_survives_continuous_circling_for_two_revolutions
    one_turn_per_tick_circles_back_to_the_start_cell
    right_angle_turn_between_ticks_still_turns
    three_key_burst_not_ending_in_reversal_stays_alive
```

Success criteria for Task 1's verification: the run compiles AND the 5 repro tests FAIL with the exact messages above AND the 5 pins pass. The bug is thereby reproduced at BOTH layers (domain + loop), as the TODO requires.

Failure-handling rules:
- If any of the 5 repro tests unexpectedly PASSES: STOP, do not "fix" the test, report to the caller (a setup assumption is broken — e.g. coordinates or collision semantics changed).
- If a pin unexpectedly FAILS on current code: STOP and report the same way.
- If compilation fails: STOP and report (no creative fixing beyond matching §2.1 verbatim).

---

## 5. Git Actions (step 4.2 scope)

Branch: `fix/circling-reversal-death` (pre-existing). No branch creation/switch, no push (push is restricted to Step 5 of the workflow).

Before each commit: run `git status`, confirm staged set matches ONLY the intended file, and re-check `.gitignore` compliance (gitignore-compliance rule). `target/` and `dist/` must never appear staged.

1. Commit the plan artifact (this file):
   - `git add .kilo/plans/20261006-direction-swap-reversal-failing-tests.md`
   - `git commit -m "docs: add implementation plan for the direction-swap failing tests"`
2. Commit the failing tests:
   - `git add tests/direction_swap_reversal.rs`
   - `git commit -m "test: add failing regression tests for the circling reversal bug"`
3. Nothing else is staged or committed. In particular: no `src/` file, no `Cargo.toml`, no TODO file (the `[DONE]` mark belongs to step 4.6), no docs.

---

## 6. Downstream Hooks (informational, not executed in 4.2)

- **4.3 code-reviewer / code-simplifier:** review `tests/direction_swap_reversal.rs` against §2. The intentional failures MUST NOT be "fixed" by weakening assertions. Simplification latitude: none beyond formatting.
- **4.4 docs-specialist:** the //! module header already documents the file; only polish/extend that header if needed (AI-agent guidance). No inline comments elsewhere (self-documenting-code rule). No README/docs changes (Task 3).
- **4.5b adherence (architector):** verify the file matches §2 exactly and the verification output matches §4.3.
- **4.6 task completion:** append `[DONE]` to the Task 1 `###` heading in `.agent/todos/20261006/20261006-todo-1.md` and commit.
- **Task 2 coupling:** the new tests are the acceptance criteria of the domain fix. Task 2's fix must satisfy §1.3 decision 3 (the `[Right, Down]` pin) — a naive per-tick cap that counts the same-direction no-op as a change would break `double_key_turn_within_one_tick_toward_free_cells_still_turns` and violates the global plan's "Right→Down remains allowed" constraint.

---

## 7. Verification Checklist vs TODO Task 1 (one-to-one)

| TODO Task 1 requirement | Covered by |
|---|---|
| New integration test file `tests/direction_swap_reversal.rs` reproducing the bug at both layers | §2.1 (10 tests: domain + loop) |
| Domain-level test: `change_direction` applied multiple times before/within a tick (`advance_one_step`), sequences like `[Up, Left]`, triple burst, full-circle pattern, asserting NOT GameOver + snake intact | `two_key_burst_within_one_tick_must_not_step_onto_the_neck`, `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck`, `two_tick_interleaved_burst_must_not_reenter_the_body`, `three_key_burst_not_ending_in_reversal_stays_alive`, `one_turn_per_tick_circles_back_to_the_start_cell` |
| "snake must remain intact (no self overlap beyond the moving head)" | `assert_snake_survived` segment-uniqueness loop + length/score asserts |
| Loop-level test through `game_loop::tick` (headless, in-memory `Vec<u8>` renderer) with swap sequences; snake must survive continuous circling (4+ swaps forming a loop) | `HeadlessLoop` + `loop_tick_survives_a_two_key_burst_in_one_tick`, `loop_tick_survives_a_three_key_burst_ending_in_reversal`, `loop_tick_survives_continuous_circling_for_two_revolutions` (8 turns) |
| Run `cargo test --test direction_swap_reversal` (Docker in the Alpine VM via MCP) and confirm the new tests FAIL | §4.1 (ownership split) + §4.2 (exact command) + §4.3 (expected 5 failed / 5 passed) |
| Commit the failing tests | §5 commit 2 |

Prompt-required scenario mapping: (a) → repro 1; (b) → kept as `three_key_burst_not_ending_in_reversal_stays_alive` (honest role: survival pin — see §1.2) with its failing role fulfilled by the verified `[Right, Down, Left]` and two-tick interleave repros; (c) → `one_turn_per_tick_circles_back_to_the_start_cell` + loop double revolution; (d) → `right_angle_turn_between_ticks_still_turns` + `double_key_turn_within_one_tick_toward_free_cells_still_turns`.

---

## 8. Risk Notes

1. Coordinate drift risk: any future change to `src/game/setup.rs` constants invalidates the exact-position pins. The pins reference the verified constants of 2026-10-06 (head (10,12), neck (9,12), tail (8,12), food (20,12)).
2. Fix-coupling risk: contained by §1.3 decisions 1–2 (no bool returns, no positions in fix-dependent cases). Only the `[Right, Down]` pin intentionally constrains the fix (decision 3).
3. VM availability risk: if `vm_status` fails, the caller pauses and asks the user to start the VM (per global plan). No local fallback exists.
4. Ephemeral cargo cache: each Docker run re-downloads dependencies. Slow but proven; do not add caches or helper scripts (no-helper-script convention, `compose.yaml` header).
