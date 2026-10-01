# Implementation Plan — Phase 1A, Group D (TODO Tasks 14–15)

**Source:** `.agent/todos/20261001/20261001-todo-2.md` — tasks 14, 15 (+ "Implementation Constraints" + "Out of Scope").
**Scope of this plan is step 4.1b ONLY (analysis & planning) for Group D.** Global plan decision 10: this TODO has NO front-end tasks; 4.1a/4.5a do not apply.
**Global binding plan:** `.kilo/plans/20261001-phase1a-core-game-model.md` — its Technical & Architecture Decisions are NOT re-discussed here; this plan specializes them into exact code shapes for Group D.
**Built on Groups A/B/C output** (adherence: ADHERENT for all). Current branch: `feat/phase1a-core-game-model` (verified). No branch creation, no push, no merge in this group.
**Toolchain constraint (global plan decision 1):** NO local Rust toolchain — tests are AUTHORED ONLY; no `cargo sketch`, no test execution. Execution of `cargo test` is deferred to the Docker phase (it will run lib tests + these integration tests). Verification here = manual read-through only.

**Current public surface reference (verified against actual files):** everything the tests need is already `pub` — see §1 D2 for the exact inventory.

---

## 0. TODO Task Details (quoted, canonical)

### Task 14 — Add Core Logic Tests
- "Add automated tests for deterministic game logic where practical."
- Coverage list (the most important rules):
  * Initial Snake length.
  * Initial score.
  * Direction changes.
  * Rejection of opposite directions.
  * Snake movement.
  * Snake growth.
  * Food consumption.
  * Score increment.
  * Food placement constraints.
  * Boundary collision.
  * Self collision.
- "Randomized behavior does not need exhaustive testing. Tests should focus on the guarantees the game logic provides."
- "The tests should not require an interactive terminal."

### Task 15 — Update Readme
- "Remove base-project notes from the README."
- "And update the file to specify only details of current project."

### Constraints & Out of Scope (bind everything below)
- No engine, ECS, rendering, terminal, keyboard/input, networking, persistence, configuration systems, unnecessary abstractions. Domain stays terminal-free. Tests are headless.
- Still out of scope: NO `Cargo.lock`, NO Docker files, NO terminal code, NO Phase 1B loop/input/render.
- Rule packs: `src/` files ≤ 200 lines (max-lines-per-file applies to `src/` ONLY — `tests/` files are OUTSIDE the cap, but every test file is kept small and focused anyway).

---

## 1. Encoded Decisions (one per formerly-open point — no open choices)

### D1. Test architecture (the binding structural decision) — `src/lib.rs` owns the module tree; `tests/` integration tests

Integration tests in `tests/` CANNOT import a binary crate. The Group C handoff recommended `tests/` (state.rs is 142 total / 96 code lines; inline task-14 tests would breach the 200-line cap). The sound, standard, SINGLE-source-of-truth option — PICKED:

- Create **`src/lib.rs`** containing exactly:
  ```rust
  pub mod game;
  ```

- Change **`src/main.rs`** to exactly:
  ```rust
  fn main() {}
  ```

- `src/game.rs` and the entire `src/game/*` tree: **UNCHANGED** (lib.rs declares `pub mod game;`, which pulls in `src/game.rs`, which declares the 8 submodules — identical wiring to today).
- `Cargo.toml`: **UNCHANGED.** With both `src/lib.rs` and `src/main.rs` present, Cargo auto-detects a lib target (`snake`) and a bin target (`snake`) — same package name for both is legal; no `[[bin]]`/`[lib]` sections needed. Do NOT add any manifest sections.
- Consequences (encode for the reviewer): `mod game;` disappears from `main.rs` → the binary no longer re-declares the module tree (no duplicate compilation). All game code is reachable ONLY through the lib crate (`snake::game::...`). Integration tests use `use snake::game::...` paths. Dead-code warnings DISAPPEAR organically: every lib item is `pub` or transitively used by a `pub` function (verified: `is_playing`, `respawn_food`, `is_board_full`, `random_board_position`, `head_to_body_slice`, `is_within_bounds`, `is_immediate_reversal` are each called from a `pub` item).
- REJECTED alternative (do not "fix" back): keeping `mod game;` in `main.rs` AND adding `lib.rs` — duplicate module-tree parsing; bin target regains dead_code warnings; two copies of the domain live in one test run.

### D2. Public-surface audit — ZERO visibility changes

Verified against the actual files; every item tests reference is ALREADY `pub`:

| Item needed by tests | File | Visible today? |
|---|---|---|
| `GameState::{new, snake, food, current_direction, score, status}` | `state.rs` | pub ✓ |
| `GameState::{change_direction, advance_one_step, start_playing, enter_game_over}` | `state.rs` | pub ✓ |
| `WIDTH`, `HEIGHT`, `MIN_AVAILABLE_COORDINATE`, `is_inside_board`, `GameStatus` | `state.rs` | pub ✓ |
| `Snake::{new, head, segments, length, advance}` | `snake.rs` | pub ✓ |
| `Food::{new, position, occupies}` | `food.rs` | pub ✓ |
| `Direction::{opposite, offset}` + derived `PartialEq` | `direction.rs` | pub ✓ |
| `Position { pub x, pub y }` + `Add` impl | `position.rs` | pub ✓ |
| `initial_setup()`, `GameStateSetup` (pub fields `snake`, `food`, `direction`) | `setup.rs` | pub ✓ |
| `collision::{is_outside_board, collides_with_body}` | `collision.rs` | pub ✓ |
| `food_placement::choose_food_position` | `food_placement.rs` | pub ✓ |

**NO `pub` additions, NO src/game edits of any kind. MINIMIZE-visibility requirement satisfied with zero edits.** `setup.rs`'s `INITIAL_*` constants stay private — tests must not depend on them (build every expected value from the public API instead, e.g. `initial_setup().snake.head()`); this keeps the initial coordinates an implementation detail per TODO task 5.

### D3. `advance_one_step` status gate — tests must call `start_playing()` first

`advance_one_step` starts with `if !self.is_playing() { return; }`. A `WaitingToStart` game silently skips ticks. EVERY test that drives movement/consumption/or GameOver through `GameState` MUST call `start_playing()` first. Direct `Snake`/predicate-level tests don't need it. `change_direction` has NO status gate (works on any status).

### D4. Test file split + determinism model (six files under `tests/`)

Files map to TODO task 14's bullets; integration tests = plain `#[test]` functions (NO `#[cfg(test)]` module wrappers in `tests/`):

| File | TODO bullets covered | Tests |
|---|---|---|
| `tests/initial_state.rs` | initial snake length, initial score (+ initial direction/food Guarantees) | 7 |
| `tests/direction.rs` | direction changes, rejection of opposite directions | 10 |
| `tests/movement_and_growth.rs` | snake movement, snake growth (+ the status gate) | 7 |
| `tests/food_consumption_scoring.rs` | food consumption, score increment, respawn | 4 |
| `tests/collision.rs` | boundary collision, self collision (unit + state level) | 12 |
| `tests/food_placement.rs` | food placement constraints (guarantee tests) | 4 |

Determinism model (decision, applying to every file):
- **State-level tests** build games ONLY via `GameState::new(initial_setup())` and drive them via `change_direction`/`advance_one_step` with step counts DERIVED from the public API (e.g. steps to eat = `food.position().x - snake.head().x`; steps to wall = `WIDTH - snake.head().x`). No coordinate literals from private constants are asserted; expected positions are expressed as captured-before values + `Direction::offset()` arithmetic. Fully deterministic.
- **Self-collision: unit-style direct tests of `collision::collides_with_body` with handcrafted slices** (decision per analysis: a state-level self-collision needs a snake of length ≥ 5 chasing its own body across ≥ 4 steps, which requires indeterministic food respawn to grow — impossible through the public API deterministically). The unit tests cover the exact predicate `advance_one_step` calls, including the tail-exclusion semantics (task 12's "account for the normal movement behavior of the tail").
- **Boundary: BOTH** unit tests of `collision::is_outside_board` (all edge cases: min/max per axis, off by one on both sides) AND state-level wall-death tests through `advance_one_step` (right wall + bottom wall), asserting the game STOPS at the wall without wrap (head never leaves the last cell).
- **Food placement: guarantee assertions only** (in-bounds, not occupied, `Some` when a free cell exists, `None` on a full board) — never exact coordinates; exercise the randomness with loops of repeated calls.

### D5. README approach (task 15) — target = game-only README

- KEEP (factual, current-project): title `# Rust Snake` + description paragraphs, `**Attention AI Agents**` block (links `AGENTS.md`), `About this Project`, `Game Rules & Controls`.
- REWRITE: `Build & Run` (status lines updated to Phase-1A-factual state), `Project Structure` (current game files + lib/tests; one-line pointer to AI-agent dirs), `Table of Contents` (regenerated for kept sections).
- REMOVE ENTIRELY (base-project/template notes): `Compatibility` (Kilo Code plugin versions, opencode Go subscription links, Grok/Gemini/vast.ai testing notes), `Prerequisites` (Git/AI-agent-handler requirements — the Docker build prerequisite is already covered by `Build & Run`), `Getting Started` (template onboarding steps), `The Critical Workflow` (duplicated from `.kilo/commands/critical-workflow.md`), `Agent Models`, `How to Start a Task` (both options), `AI Agent Plans`, `Troubleshooting` (MCP-plugin notes), and the closing footer note "*...workflow is actively maintained...*".
- REPLACE the removed sections with ONE short `AI Agents` section (see §2.10 exact content).
- Docker build workflow stays as a PLANNED-phase item, phrased factually ("planned for the next phase") — never described as already working. The Cargo project + core game logic stay described as present. No fabricated features.

### D6. Commit shape for Group D (step 4.2)

Exactly FOUR commits, in this order (see §4): lib refactor, tests, README, then 4.6's `[DONE]` marker commit. No other git actions.

---

## 2. Exact Contents per File

### 2.1 `src/lib.rs` — NEW (full file)
```rust
pub mod game;
```

### 2.2 `src/main.rs` — MODIFY (full file becomes)
```rust
fn main() {}
```
(The `mod game;` line is removed — the module tree moves to lib.rs per D1. Nothing else changes in `src/`.)

### 2.3 Shared test helpers (repeated per file by design — no shared helper module, no extra dependency; each file stays self-contained, junior-friendly)

```rust
fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn playing_game() -> GameState {
    let mut game = fresh_game();
    game.start_playing();
    game
}
```

Expected imports vary per file — encode exactly per file below; NO unused imports (each file imports exactly what it uses).

### 2.4 `tests/initial_state.rs` — NEW

Imports (exact):
```rust
use snake::game::direction::Direction;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, is_inside_board};
```

Helpers: `fresh_game()` only.

Tests (fn name → body directive):
1. `initial_snake_has_exactly_three_segments` — `let game = fresh_game(); assert_eq!(game.snake().length(), 3); assert_eq!(game.snake().segments().len(), 3);`
2. `initial_snake_head_leads_two_body_segments` — fresh_game: `head() == segments()[0]`; the other two are DIFFERENT cells and follow it (assert `segments()[1] != head()` and `segments()[2] != head()` and `segments()[1] != segments()[2]`).
3. `initial_score_is_zero` — `assert_eq!(fresh_game().score(), 0);`
4. `initial_direction_is_right` — `assert_eq!(fresh_game().current_direction(), Direction::Right);`
5. `initial_status_is_waiting_to_start` — `fresh_game().status() == GameStatus::WaitingToStart` (representational guarantee backing the "does not immediately begin moving" brief rule).
6. `initial_snake_is_inside_the_board` — every `segments()` position satisfies `is_inside_board(pos)`.
7. `initial_food_is_valid` — ONE test asserting both placement constraints of task 5: `is_inside_board(game.food().position())` AND `!game.snake().segments().contains(&game.food().position())`.

### 2.5 `tests/direction.rs` — NEW

Imports (exact — `GameStatus` is NOT used in this file, do not import it):
```rust
use snake::game::direction::Direction;
use snake::game::setup::initial_setup;
use snake::game::state::GameState;
```

Helpers (exact):
```rust
fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}
```

Valid-change tests. Each: fresh state, chain turns to establish the current direction, assert `change_direction(candidate)` returns `true` AND `current_direction() == candidate`:
1. `right_changes_to_up` — fresh; Up accepted → current_direction == Up.
2. `right_changes_to_down` — fresh; Down accepted.
3. `up_changes_to_left` — fresh; change(Up) then change(Left); both accepted; current == Left.
4. `up_changes_to_right` — fresh; change(Up) then change(Right); current == Right (Up→Right is NOT opposite).
5. `down_changes_to_left` — fresh; change(Down) then change(Left).
6. `down_changes_to_right` — fresh; change(Down) then change(Right).
7. `left_changes_to_up` — fresh; change(Up) then change(Left) then change(Up).
8. `left_changes_to_down` — fresh; change(Up) then change(Left) then change(Down).
9. `same_direction_change_is_accepted` — fresh; `change_direction(Direction::Right)` on a Right-facing game returns `true` and `current_direction()` stays `Right` (encoded edge case from Group B D3; candidate == current is not an opposite).

Rejection tests. Each: chain turns to establish the current direction, assert `change_direction(opposite)` returns `false` AND `current_direction()` is UNCHANGED:
10. `right_rejects_left` — fresh; change(Left) → false; current stays Right.
11. `left_rejects_right` — fresh; change(Up); change(Left); change(Right) → false; current stays Left.
12. `up_rejects_down` — fresh; change(Up); change(Down) → false; current stays Up.
13. `down_rejects_up` — fresh; change(Down); change(Up) → false; current stays Down.

(That is 13 tests; table in D4's row says 10 — the count is per-row indicative, the ABOVE list is the exact spec. All rows' bullet coverage is identical.)

### 2.6 `tests/movement_and_growth.rs` — NEW

Imports (exact):
```rust
use snake::game::direction::Direction;
use snake::game::setup::initial_setup;
use snake::game::snake::Snake;
use snake::game::state::GameState;
```

Helpers (exact):
```rust
fn playing_game() -> GameState { ... } // per §2.3
```

Movement tests (state level, deterministic, no coordinate literals):
1. `step_moves_head_one_cell_in_the_current_direction` — playing_game; capture `head_before = game.snake().head()`; `advance_one_step()`; assert `game.snake().head() == head_before + Direction::Right.offset()`.
2. `normal_step_preserves_snake_length` — playing_game; advance 5 steps in a loop; assert `game.snake().length() == 3` (still) — loop `for _ in 0..5`.
3. `body_follows_the_head` — playing_game; capture `let segments_before = game.snake().segments().to_vec();` (a `Vec<Position>`); advance once; assert `game.snake().segments()[1] == segments_before[0]` and `game.snake().segments()[2] == segments_before[1]` (every body segment moved accordingly; tail tip dropped).
4. `advance_does_nothing_before_playing` — fresh_game (NOT started); capture head; `advance_one_step()`; assert head unchanged AND length unchanged (the status gate is a guarantee protecting WaitingToStart).

Growth tests (unit level via public `Snake::advance`, deterministic by construction):
5. `growth_step_adds_exactly_one_segment` — build `let body = vec![Position { x: 0, y: 0 }, Position { x: -1, y: 0 }]; let mut snake = Snake::new(body);` then `snake.advance(Position { x: 1, y: 0 }, false);` → assert `snake.length() == 3` (was 2 → exactly +1).
6. `growth_step_keeps_the_previous_tail_tip` — same construction; after `advance(…, false)` assert `snake.segments().last()` equals the ORIGINAL last cell `(x: -1, y: 0)` (tail NOT removed on the growth step).
7. `normal_step_removes_the_tail` — same 2-segment snake; `snake.advance(Position { x: 1, y: 0 }, true)` → `length() == 2` (preserved) AND `segments().last()` == the OLD `(x: 0, y: 0)` (the old tail was dropped; the old head became the new tip's predecessor).

(The `x: -1, y: 0` literal here is a handcrafted unit fixture, not a board coordinate — allowed; `Position` fields are `pub` by Group A decision.)

### 2.7 `tests/food_consumption_scoring.rs` — NEW

Imports (exact):
```rust
use snake::game::setup::initial_setup;
use snake::game::state::{GameState, is_inside_board};
```

Helpers (exact):
```rust
fn playing_game() -> GameState { ... } // per §2.3

fn steps_to_reach_food() -> i32 {
    let setup = initial_setup();
    setup.food.position().x - setup.snake.head().x
}

fn game_that_has_eaten() -> GameState {
    let mut game = playing_game();
    let steps = steps_to_reach_food();
    for _ in 0..steps {
        game.advance_one_step();
    }
    game
}
```
(Sound by construction: the initial setup faces `Right`, and the head rows/food row coincide by design of `initial_setup` — verified: head (10,12), food (20,12), same `y`. Do NOT hardcode 10; the helper derives the count.)

Tests:
1. `head_reaching_the_food_consumes_it` — capture `food_cell = game.food().position()` BEFORE eating via fresh playing_game; then from `game_that_has_eaten()` assert `game.snake().head() == food_cell` (the head rests on the consumed cell — consumption happened exactly at arrival, not on a later step).
2. `eating_food_increments_score_by_exactly_one` — `assert_eq!(game_that_has_eaten().score(), 1);`
3. `eating_food_grows_the_snake_by_exactly_one_segment` — `assert_eq!(game_that_has_eaten().snake().length(), 4);` (3 + 1).
4. `new_food_after_consumption_is_valid` — from `game_that_has_eaten()`: `is_inside_board(game.food().position())` AND `!game.snake().segments().contains(&game.food().position())` (respawned off the POST-move snake, in-bounds; position itself is random — assert guarantees only).

### 2.8 `tests/collision.rs` — NEW

Imports (exact):
```rust
use snake::game::collision::{collides_with_body, is_outside_board};
use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, HEIGHT, WIDTH, is_inside_board};
```
(If a specific test file ends up not using one of these — e.g. `initial_setup` IS used by the playing-game helper; `is_inside_board` used by the no-wrap asserts — verify each is referenced; drop any that is not, at implementation time, to avoid unused-import warnings.)

Helpers (exact): `playing_game()` (per §2.3).

Boundary unit tests:
1. `cells_inside_the_board_are_not_outside` — assert `is_outside_board(Position { x: 0, y: 0 }) == false` and `is_outside_board(Position { x: WIDTH - 1, y: HEIGHT - 1 }) == false`; cross-check the inverse delegation contract: `is_inside_board` on those same two cells returns `true` (assertions in one fn are fine).
2. `boundary_edges_of_every_axis_are_outside` — four asserted cases, all `is_outside_board(...) == true`: `Position { x: WIDTH, y: 0 }`, `Position { x: 0, y: HEIGHT }`, `Position { x: -1, y: 0 }`, `Position { x: 0, y: -1 }` (one step beyond each edge; &&-tests per case, no loop needed).

Boundary state-level tests:
3. `hitting_the_right_wall_ends_the_game_without_wraparound` — playing_game; capture `steps_until_wall = WIDTH - game.snake().head().x` (30 with the shipped setup); loop that many `advance_one_step()` calls; assert `game.status() == GameStatus::GameOver`; assert `game.snake().head().x == WIDTH - 1` (head STOPPED at the last playable column — the fatal step did not apply the move, proving no wrap-around and no overshoot).
4. `hitting_the_bottom_wall_ends_the_game_without_wraparound` — playing_game; `game.change_direction(Direction::Down)`; capture `steps_until_floor = HEIGHT - game.snake().head().y` (13); loop that many `advance_one_step()` calls; assert `game.status() == GameStatus::GameOver` AND `game.snake().head().y == HEIGHT - 1` (no wrap-down on the y axis).

Self-collision unit tests (handcrafted slices; `Position` literals):
5. `head_landing_on_a_body_segment_is_a_collision` — segments `vec![Position { x: 5, y: 5 }, Position { x: 4, y: 5 }, Position { x: 3, y: 5 }]; next_head = Position { x: 4, y: 5 }` → `collides_with_body(next_head, &segments) == true`.
6. `tail_cell_is_excluded_because_it_vacates` — same 3-segment fixture; `next_head = Position { x: 3, y: 5 }` (the CURRENT tail) → `collides_with_body(...) == false` (tail-exclusion semantics: the tail vacates its cell in the same step).
7. `head_front_cell_of_a_straight_snake_is_not_a_collision` — same fixture; `next_head = Position { x: 6, y: 5 }` (the cell ahead of the head) → false.
8. `empty_snake_cannot_collide` — `collides_with_body(Position { x: 0, y: 0 }, &[]) == false`.
9. `single_segment_snake_cannot_collide` — segments `vec![Position { x: 7, y: 7 }]`; `next_head = Position { x: 7, y: 7 }` → false (head cannot hit itself; a len-1 slice leaves an empty check window).

EXCLUDED by design (do NOT write): a "growth step never reports a tail collision" test — the tail exclusion is UNCONDITIONAL (Group C plan D5); growth-step safety is a documented invariant (food never overlaps the snake), NOT enforced by the predicate; testing an invariant outside the predicate's contract is out of scope.

**`tests/collision.rs` exact final test list (NINE `#[test]` fns, in this order):**
1. `cells_inside_the_board_are_not_outside`
2. `boundary_edges_of_every_axis_are_outside`
3. `hitting_the_right_wall_ends_the_game_without_wraparound`
4. `hitting_the_bottom_wall_ends_the_game_without_wraparound`
5. `head_landing_on_a_body_segment_is_a_collision`
6. `tail_cell_is_excluded_because_it_vacates`
7. `head_front_cell_of_a_straight_snake_is_not_a_collision`
8. `empty_snake_cannot_collide`
9. `single_segment_snake_cannot_collide`

(No other content in the file.)

### 2.9 `tests/food_placement.rs` — NEW

Imports (exact):
```rust
use snake::game::food_placement::choose_food_position;
use snake::game::position::Position;
use snake::game::state::{HEIGHT, WIDTH, is_inside_board};
```

Tests (guarantees only — never exact coordinates):
1. `free_board_yields_positions_inside_the_board` — loop 8 times over `choose_food_position(&[])`; every result is `Some(p)` with `is_inside_board(p)`.
2. `chosen_positions_never_overlap_occupied_cells` — occupied = `vec![Position { x: 1, y: 1 }, Position { x: 2, y: 2 }, Position { x: 3, y: 3 }]`; loop 20 calls; each result `Some(p)` with `p != ` any occupied cell (assert via `!occupied.contains(&p)`), and in-bounds.
3. `some_is_returned_until_the_last_free_cell_is_reached` — `let free = Position { x: WIDTH - 1, y: HEIGHT - 1 }; let occupied = all_board_positions(Some(free));` (999 positions, exactly one free cell); assert `choose_food_position(&occupied) == Some(free)` — deterministic: the ONLY free cell must be returned. Termination guaranteed with probability ~1 (expected ~1000 retries; fine in practice).
4. `full_board_returns_none_explicitly` — build all 1000 cells via the same nested-loop builder WITHOUT the skip (`for x in 0..WIDTH { for y in 0..HEIGHT { occupied.push(Position { x, y }); } }`); assert `None`. (The is_board_full pre-check fires BEFORE any loop iteration — task 9's "no infinite loop" guarantee, tested directly.)

Shared local helper for tests 3–4 (exact, above the tests):
```rust
fn all_board_positions(except: Option<Position>) -> Vec<Position> {
    let mut occupied = Vec::new();
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let candidate_position = Position { x, y };
            if Some(candidate_position) == except {
                continue;
            }
            occupied.push(candidate_position);
        }
    }
    occupied
}
```
(One-section skip condition via `Option` comparison — no boolean-conjunction. Tests 3–4 call `all_board_positions(Some(free))` / `all_board_positions(None)`.)

### 2.10 `README.md` — MODIFY per D5 (task 15)

Exact edit directives, section by section (line refs from the CURRENT file read during this analysis):

**DELETE:**
- Lines 25–40: entire `## Compatibility` section (Kilo Code plugin version notes, opencode subscription links, model-provider testing notes — base-project notes).
- Lines 42–49: entire `## Prerequisites` section (Git/AI-agent-handler prerequisites; Docker prerequisite already covered by Build & Run).
- Lines 99–107: entire `## Getting Started` section.
- Lines 109–141: entire `## The Critical Workflow` section (MERMAID graph + link to the workflow file — duplicated base-project process doc).
- Lines 143–152: entire `## Agent Models` section.
- Lines 154–178: entire `## How to Start a Task` section (both options + note).
- Lines 180–190: entire `## AI Agent Plans` section.
- Lines 192–198: entire `## Troubleshooting` section.
- Line 200–202: the closing horizontal rule + footer note.

**KEEP verbatim (content unchanged):**
- Lines 1–3: `# Rust Snake` title + intro paragraph (one factual adjustment is permitted inside line 3 if still true — it is: Docker compile → `dist/snake.exe`).
- Lines 5: `**Attention AI Agents:** ...` block.

**KEEP with edits:**
- `## Table of Contents` → regenerate to list exactly the kept sections below (linked anchors).
- `## About this Project` (lines 51–57) → keep all three paragraphs (all factual, project-only).
- `## Game Rules & Controls` (lines 59–68) → keep the bullet list verbatim (all facts from the brief; matches the implemented domain rules).
- `## Build & Run` (lines 70–76) → keep the three factual bullets (`docker compose run --rm build`, output path, run-on-Windows). REPLACE the status bullet (line 75) with exactly:
  ```text
  - Status: the Cargo project, the core game model, and its core logic tests are implemented; the Docker build environment is planned for the next phase (`Dockerfile` + `compose.yaml` do not exist yet).
  ```
  KEEP the final `dist/` git-ignored bullet (line 76).
- `## Project Structure` (lines 78–86) → REPLACE the intro sentence + the 5 tooling bullets with:
  ```text
  A compact Rust project; the game domain is split into small modules under `src/game/`:

  - [`Cargo.toml`](Cargo.toml): the Cargo manifest (package `snake`, dependency `rand`).
  - `src/lib.rs`: library entry; exposes the `game` module for tests and future phases.
  - `src/main.rs`: binary entry point (the interactive game loop lives in a later phase).
  - [`src/game.rs`](src/game.rs): `game` module root; declares the domain submodules.
  - `src/game/position.rs`, `src/game/direction.rs`, `src/game/snake.rs`, `src/game/food.rs`: core value types (grid cell, direction, head-first snake, food).
  - `src/game/setup.rs`, `src/game/state.rs`: initial setup + game state with the per-tick move/consume/collide driver.
  - `src/game/collision.rs`, `src/game/food_placement.rs`: death predicates and random free-cell food placement.
  - `tests/`: integration tests for the deterministic core logic (`cargo test` runs them; execution arrives with the Docker build phase).
  - Planned later: `Dockerfile`, `compose.yaml`, `Cargo.lock`, `dist/snake.exe`.

  AI agent integration (`.agent/`, `.kilo/`, `.opencode/`) is documented in [`AGENTS.md`](AGENTS.md).
  ```
  (Drop the `### Application Files (Planned)` sub-heading and its TOC child.)
- NEW `## AI Agents` section (replaces all removed workflow/marketing sections; placed after `Project Structure`), exact content:
  ```text
  ## AI Agents

  Before making changes, read [`AGENTS.md`](AGENTS.md) and follow the Critical Workflow (`.kilo/commands/critical-workflow.md`). Persistent project context lives in [`.agent/project-info/`](.agent/project-info/) — `brief.md` is the source of truth for scope.
  ```

Final section order: `# Rust Snake` intro → Attention AI Agents → Table of Contents → About this Project → Game Rules & Controls → Build & Run → Project Structure → AI Agents. NO other content.

---

## 3. High-Level Approach (ordered)

1. Git: confirm branch `feat/phase1a-core-game-model` (implementer re-checks `git branch --show-current`). NO branch creation.
2. Task 14 files, dependency order:
   a. Create `src/lib.rs` (§2.1); change `src/main.rs` to §2.2 → commit 1.
   b. Write `tests/initial_state.rs` → `tests/direction.rs` → `tests/movement_and_growth.rs` → `tests/food_consumption_scoring.rs` → `tests/collision.rs` → `tests/food_placement.rs` per §2.4–§2.9 → commit 2.
3. Task 15: apply the README edits per §2.10 → commit 3.
4. Read-only self-verification with §5 checklist (NO compilation — no local toolchain; also NO cargo commands even for check).
5. (Handled by caller workflow steps 4.3–4.6; see §7 handoffs.)

## 4. Git Handling

- Branch: stay on `feat/phase1a-core-game-model`. No push, no merge, no `main` writes (workflow Step 5 owns those).
- Exactly FOUR commits, in this order, staging ONLY the listed paths:
  1. `refactor: expose the game module through a library crate` — files: `src/lib.rs`, `src/main.rs`.
  2. `test: add core logic tests for the deterministic game rules` — files: all six `tests/*.rs`.
  3. `docs: remove base-project notes and describe the current project in the readme` — file: `README.md`.
  4. (step 4.6, by the implementer's completion sub-step) `docs: mark phase 1A group D tasks done` — file: the TODO file.
- Before each commit: run `git status`, read `.gitignore`; stage ONLY files matching no ignore pattern. Known pre-existing untracked items NOT owned by this step — leave untouched: `.agent/todos/20261001/20261001-todo-3.md`, `20261001-todo-4.md`, and the deleted `.kilo/plans/.gitkeep`.

## 5. Verification (no compiler — manual checks, ordered)

1. **Module wiring closed-loop (lib):** `src/lib.rs` = one `pub mod game;` line; `main.rs` = exactly `fn main() {}`; `src/game.rs` unchanged with its 8 `pub mod` lines matching `src/game/*.rs` one-to-one; `Cargo.toml` untouched (lib+bin auto-detection note from D1).
2. **Import-path validity (every test file):** each `use snake::game::…` path resolves through lib.rs → game.rs → the submodule (paths like `snake::game::state::GameState` — lib crate name `snake` per Cargo.toml package name); NO `crate::` paths inside `tests/` files (integration tests are a separate crate).
3. **No unused imports:** cross-check each file's `use` list against §2.4–§2.9 exactly; any import not referenced in that file's fns/helpers is removed.
4. **Status-gate audit:** EVERY test that calls `advance_one_step` uses a game on which `start_playing()` was called (movement ×3, consumption helper, wall ×2) — cite the `is_playing` gate; no test asserts movement from a `WaitingToStart` state except `advance_does_nothing_before_playing` (which asserts the gate itself).
5. **TODO task 14 coverage audit (bullet-by-bullet):** initial snake length → `initial_snake_has_exactly_three_segments` + head-leads test; initial score → `initial_score_is_zero`; direction changes → the 9 acceptance fns; rejection of opposites → the 4 rejection fns; snake movement → the 3 movement + gate fns; snake growth → the 3 `Snake::advance` fns; food consumption → `head_reaching_the_food_consumes_it`; score increment → `eating_food_increments_score_by_exactly_one` (+ growth-by-one fn); food placement constraints → the 4 `food_placement` fns; boundary collision → the 2 unit fns + 2 wall fns; self collision → the 5 unit fns. Randomized behavior tested ONLY via guarantees (in-bounds / not-occupied / Some-None) — no exact random coordinates asserted anywhere.
6. **Headless audit:** zero test reads stdin/terminal; no extern crate beyond the game lib; `tests/` files import nothing but `snake::game::*`.
7. **Style audit (tests):** self-documenting names; ≤ 2 args (none exceed); depth ≤ 2 in every loop/if; no commented-out code; assertions state the guarantee being tested (readable failure messages emerge from the names themselves; NO custom comment strings mandated — avoid adding comments unless a fixture genuinely needs one short line).
8. **README audit:** every section in the DELETE list gone; no leftover base-project wording (search for: "Kilo Code", "opencode", "Grok", "Gemini", "vast.ai", "workflow", "Critical Workflow", "Troubleshooting", "Prerequisites" outside `AGENTS.md` links); TOC matches sections; no feature claims beyond the brief (no "implemented Docker build" lies); `AGENTS.md` links still resolve; `.agent/project-info/` links still resolve.
9. **Rule audit (src/ side):** `src/lib.rs` 1 line, `src/main.rs` 2 lines — caps fine; `src/game/*` byte-untouched (verify with `git diff` showing ONLY those two src files in commit 1).
10. **gitignore compliance:** after each commit `git status` shows the expected clean state (`tests/` and `README.md`/`src/` match no ignore pattern); nothing ignored staged.

## 6. Explicit Non-Goals for Group D (scope fence)

- NO changes to `src/game/*` (zero edits — D2), NO changes to `Cargo.toml`, NO new dependencies (no test frameworks/assertion libs — plain built-in `#[test]` assertions).
- NO `cargo`/`cargo test` execution, NO `Cargo.lock` creation (Docker phase).
- NO state-level self-collision test (D4: deterministic construction impossible via public API — the unit-style predicate tests stand in for it).
- NO test for: board-full respawn behavior inside `advance_one_step` (requires the indeterministic eat-first path; the board-full behavior is covered by `full_board_returns_none_explicitly` at the placement level).
- NO phase-1B features (terminal, input, loop), NO project-info/context edits (Step 6), NO `.agent/project-structure.md` edits (step 4.4 owns).

## 7. Notes for Later Steps (handoff — do not act on them in this step)

- **4.3 (code-reviewer/code-simplifier):** pre-decided OUT of fix bounds without a plan amendment: lib.rs-over-dual-module-tree (D1); six-files split (D4); handcrafted-slice collision tests (D4); guarantee-only food-placement assertions (D4); per-file duplicated `playing_game`/`fresh_game` helpers (D3 rationale: self-contained files); status-gate usage pattern (D3); README paragraph-presence decisions (D5). The exact test-fn lists in §2.4–§2.9 are the spec; count discrepancies between D4's indicative table and the full lists resolve in favor of the FULL LISTS.
- **4.4 (docs-specialist):** update `.agent/project-structure.md`: add `src/lib.rs` (+ current `src/main.rs` line changes meaning: binary entry), add a `tests/` entry (integration tests for the deterministic core logic: `initial_state.rs`, `direction.rs`, `movement_and_growth.rs`, `food_consumption_scoring.rs`, `collision.rs`, `food_placement.rs`). `README.md` is owned by task 15 itself — the 4.4 pass should NOT re-edit it.
- **4.6 (implementer):** append `[DONE]` to task headings `14. Add Core Logic Tests` and `15. Update Readme` ONLY (byte-preserving edit; overwrite-todo-file-prevention rule), commit per §4 message 4.
- **Docker-phase note (next phase, NOT this plan):** `cargo test` will run the integration tests directly (lib target compiled automatically); expect zero dead_code warnings (audited in D1); `cargo build` produces the binary from the empty `main` until Phase 1B fills it.

## 8. Cross-Check Against TODO Tasks 14–15

| TODO requirement | Where satisfied in this plan |
|---|---|
| 14: automated tests for deterministic logic where practical | D4→D3 model: all six files deterministic via public API; step counts derived, never literal-coupled to private constants |
| 14: initial Snake length (3) | `initial_snake_has_exactly_three_segments` |
| 14: initial score (0) | `initial_score_is_zero` |
| 14: direction changes | 9 acceptance fns incl. same-direction no-op (§2.5) |
| 14: rejection of opposite directions | 4 rejection fns, one per opposite pair, each asserting unchanged direction (§2.5) |
| 14: snake movement | 3 movement fns (head math, length preservation, body following) |
| 14: snake growth | 3 `Snake::advance(…, false)` fns (exactly +1, tail retained; contrast fn for normal step) |
| 14: food consumption | `head_reaching_the_food_consumes_it` (arrival == consumption) |
| 14: score increment | `eating_food_increments_score_by_exactly_one` (exactly 1) + `eating_food_grows_the_snake_by_exactly_one_segment` + respawn validity fn |
| 14: food placement constraints | in-bounds / not-occupied / last-free-cell Some / full-board None (§2.9) |
| 14: boundary collision | unit edge cases + both state-level wall deaths with no-wrap assertions (§2.8) |
| 14: self collision | 5 unit fns over `collides_with_body` incl. tail-vacate exclusion (§2.8) |
| 14: randomized behavior NOT exhaustively tested — focus on guarantees | D4 guarantee-only random assertions; loops exercise but never pin random values |
| 14: tests must NOT require an interactive terminal | headless `advance_one_step`/predicates only; verification §5.6 |
| 15: remove base-project notes | §2.10 exhaustive DELETE list (Compatibility, Prerequisites, Getting Started, Critical Workflow, Agent Models, How to Start a Task, AI Agent Plans, Troubleshooting, footer) |
| 15: specify ONLY current-project details | §2.10 target section order + rewritten Project Structure/Build & Run; Docker kept as factual planned-phase item; no fabricated features |
| Constraints: keep it small, no unnecessary abstractions | no test helpers crate, no dependencies, plain asserts, six focused files |
| Group C handoff: test files under `tests/` since state.rs can't take inline tests | D1 lib.rs + `tests/` — the exact structural fix encoded; src/ cap untouched |

Plan verified against both TODO tasks and both TODO constraint sections — correct as written.
