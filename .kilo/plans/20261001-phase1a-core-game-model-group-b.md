# Implementation Plan — Phase 1A, Group B (TODO Tasks 5–8)

**Source:** `.agent/todos/20261001/20261001-todo-2.md` — tasks 5, 6, 7, 8 (+ "Implementation Constraints" + "Out of Scope" bind all code).
**Scope of this plan is step 4.1b ONLY (analysis & planning) for Group B.** Global plan decision 10: this TODO has NO front-end tasks; 4.1a/4.5a do not apply. There is no "front-end group" to propagate to.
**Global binding plan:** `.kilo/plans/20261001-phase1a-core-game-model.md` — its Technical & Architecture Decisions are NOT re-discussed here; this plan specializes them into exact code shapes for Group B.
**Built on Group A output** (`a33e878`, `ced52e7`, adherence: `ADHERENT`): existing files are extended, NEVER restructured. `rand` is declared in `Cargo.toml` but Group B does not consume it (food placement = Group C, task 9).

---

## 0. TODO Task Details (quoted, canonical)

### Task 5 — Implement the Initial Game State
- Initial Snake must contain **exactly three segments**: "2 body segments + 1 head".
- Initial direction: **Right**.
- Initial score: **0**.
- Snake starts at a **valid position inside the board**.
- Initial food: **inside the playable area**, **not overlapping the Snake**.
- "The exact initial coordinates are an implementation detail."

### Task 6 — Implement Direction Handling
- Rule enforced: "The Snake cannot immediately change direction to the direction directly opposite its current direction."
- Invalid: Right→Left, Left→Right, Up→Down, Down→Up. Valid: the 8 perpendicular changes listed in the TODO.
- "The exact API is an implementation decision, but the rule should live in the game/domain logic rather than being coupled to terminal keyboard handling."

### Task 7 — Implement Snake Movement
- Per movement step: (1) calculate next head from current direction; (2) add the new head position; (3) move the remaining body segments accordingly; (4) "Preserve the current Snake length unless a growth operation is requested."
- "The movement logic should be independent from terminal rendering and keyboard input."
- "It should be possible for the game logic to advance one movement step without requiring an interactive terminal."

### Task 8 — Implement Snake Growth
- Growth = increase length by **exactly one segment**.
- "The new head is added normally" / "The tail is not removed for the movement in which food was consumed."
- "The implementation should avoid duplicating movement logic specifically for food consumption if a simple `grow`/`should_remove_tail` approach can keep the behavior clear."

### Constraints & Out of Scope (bind everything below)
- No engine, ECS, rendering, terminal, keyboard/input, networking, persistence, configuration systems, unnecessary abstractions. Domain stays terminal-free. Out of scope also means: NO rand-based placement (task 9 = Group C), NO consumption/scoring (task 10), NO collision detection (tasks 11–12), NO status transitions (task 13), NO tests (task 14 = Group D), NO README (task 15 = Group D), no `Cargo.lock`, no Docker files.

---

## 1. Encoded Decisions (one per formerly-open point — no open choices)

### D1. Where initial-state construction lives
A single `pub fn initial_setup() -> GameStateSetup` placed in `src/game/state.rs`, consumed by the already-existing `GameState::new(setup: GameStateSetup)`. NOT a new `GameState::initial()` constructor and NOT a new file. Rationale: Group A already built the `GameStateSetup` parameter object for exactly this purpose; the initial-state factory belongs beside the constants it uses (same file, same module), keeping `GameState::new` untouched.

### D2. Exact initial coordinates and food
All as named constants in `src/game/state.rs` (no magic numbers):
```rust
const INITIAL_SNAKE_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_BODY_AHEAD: Position = Position { x: 9, y: 12 };
const INITIAL_SNAKE_BODY_BEHIND: Position = Position { x: 8, y: 12 };
const INITIAL_DIRECTION: Direction = Direction::Right;
const INITIAL_FOOD_POSITION: Position = Position { x: 20, y: 12 };
```
- Head-first ordering: segments vector passed to `Snake::new` is `[head, ahead, behind]` = `[(10,12), (9,12), (8,12)]` — snake occupies x = 8..10 on row 12, pointing Right (head at the front).
- Validity (satisfies task 5): all three segments have `0 ≤ x ≤ 39, 0 ≤ y ≤ 24` ✓; food (20,12) is in-bounds ✓ and overlaps no segment ✓ (snake is x 8..10, food x 20).
- **Initial food is FIXED: no `rand` in Group B.** Global plan assigns random placement logic to Group C's task 9; task 5's requirement is only "inside area + not overlapping", which a fixed coordinate satisfies and keeps Group B fully deterministic/testable. `rand` is consumed starting with task 9's `place_food`-style logic, NOT here.

### D3. Direction-change API
Add ONE method to `GameState` in `state.rs`:
```rust
pub fn change_direction(&mut self, new_direction: Direction) -> bool
```
- Lives on `GameState` (not on `Snake`): the TODO speaks of "the Snake's current direction" but the game state owns `current_direction`; the snake moves in one dimension only and holds no direction field — adding one to `Snake` would duplicate state. Domain location ✓ (not terminal-coupled).
- Return type: **`bool`** (`true` = change accepted, `false` = rejected). JUSTIFICATION (encoded): a rejected reversal is a routine, expected player-input outcome — not an error — so `Result` adds error-handling ceremony for normal behavior; silent-ignore would hide the outcome from Group D tests. A `bool` is the smallest information-carrying type (test asserts: `Right→Up` returns `true` and flips `current_direction`; `Right→Left` returns `false` and leaves `current_direction` unchanged).
- Enforcement, in domain code, using Group A's single opposite API:
```rust
pub fn change_direction(&mut self, new_direction: Direction) -> bool {
    if is_immediate_reversal(self.current_direction, new_direction) {
        return false;
    }
    self.current_direction = new_direction;
    true
}

fn is_immediate_reversal(current: Direction, candidate: Direction) -> bool {
    candidate == current.opposite()
}
```
- Single-section boolean conditions rule ✓ (named predicate extracted; same `return`-expression conjunction convention as Group A's plan §3.8 is authorized if any appears).
- Edge case decided: `change_direction` to the SAME direction is accepted (`candidate == current` is not an opposite) and makes it a no-op field assignment — this matches the TODO's rule list, which only forbids opposites. NOT special-cased further.
- The rule does NOT auto-apply per movement; it is exactly the gate above (Group C/1B wiring decides when the game calls it — out of Group B scope).

### D4. Movement mechanics on the Vec-backed head-first snake
Add to `src/game/direction.rs` the next-head math (TODO task 7 step 1), using the named step constants (`SINGLE_GRID_STEP`, `ZERO_GRID_STEP` — exact final form shown in §2.3):
```rust
pub fn offset(self) -> Position { /* final body per §2.3 */ }
```
- **Terminal-grid orientation (encoded):** `y` grows downward, so `Up` DECREASES y by 1. `Position` needs `use crate::game::position::Position;` added to `direction.rs` imports.
- Not on `Position` (Group A plan §3.4 explicitly reserved direction→delta math for Group B and kept `Position` methodless); not a separate "delta" type — `Position` IS the grid-space offset type here (a one-step offset is a relative position), avoiding unnecessary abstraction.

Add to `src/game/snake.rs` ONE mutating operation (the "move remaining body accordingly" of step 3 + trivial insert-then-pop mechanics):
```rust
pub fn advance(&mut self, next_head: Position, should_remove_tail: bool)
```
- Encoded final body:
```rust
self.segments.insert(0, next_head);
if should_remove_tail {
    self.segments.pop();
}
```
- Mechanics decision: **`insert(0, next_head)` + `pop()` tail removal**, NOT swap-based and NOT `truncate`. Rationale encoded: with `Position: Copy` and max length 1000 (40×25), the O(n) shift is irrelevant by the global plan's own reasoning; insert+pop is the plainest, most obviously-correct form for a learning project (brief §19 keep-it-simple; TODO constraint favors readable Rust over clever tricks).
  (insert keeps `[0]` = head contract; pop drops the tail tip; body otherwise shifts automatically by index — exactly steps 2–4 of task 7.)
- Growth integration (task 8): **`should_remove_tail: bool` flag on `advance`, per TODO task 8 and global plan decision 9** — ONE movement routine serves both scenarios. `advance(next_head, true)` = normal step (length preserved, step 4 of task 7). `advance(next_head, false)` = the food-consumption step: new head added normally, tail NOT removed → length grows by exactly +1. There is NO separate `grow()` method duplicating insert logic — the TODO explicitly prefers the flag approach.
- Two params ≤ 2-args rule ✓. Method body ≈ 6 lines, depth 1 ✓.
- Method name `advance` (verb; avoids shadowing std `move` keywords concerns and reads as "move forward one cell with the given head placement").

### D5. Game-level single-step operation
Add to `src/game/state.rs`:
```rust
pub fn advance_one_step(&mut self) {
    let next_head = self.snake.head() + self.current_direction.offset();
    self.snake.advance(next_head, true);
}
```
- **Name: `advance_one_step`** (game-level; distinguishes from `Snake::advance`).
- `Position + Position` requires adding a `std::ops::Add` impl OR free addition. Pick (encoded): a derive-free, hand-written `std::ops::Add` impl on `Position` in `position.rs` — the idiomatic Rust move, exercises the learning objective (traits), and makes `head + offset` read naturally. Encoded exact code:
```rust
use std::ops::Add;

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
```
- Encoded decision: the `Add` impl above is the exact shape; `head + offset` is the intended usage in `advance_one_step`. No helper methods, no exotic trait paths.
- `should_remove_tail: true` is the DEFAULT in `advance_one_step`. Growth wiring to food consumption (passing `false` exactly when food is eaten) is **Group C's task 10** — declared here as an explicit scope fence, see §6. Task 8's requirement is fully covered by the mechanism (`advance(..., false)` exists, is deterministic, and Group D can call it directly without a terminal); the trigger-to-food binding is consumption logic, owned by task 10.
- Effect on body semantics: `advance_one_step` does NOT touch `score`, `food`, or `status` — collision/consumption/status checks are tasks 9–13 (Group C). It is one pure, terminal-free domain step, satisfying task 7's "advance one movement step without requiring an interactive terminal" by direct call (including from tests).
- `GameStateSetup.status` note: `GameState::new` seeds `WaitingToStart` (Group A unchanged); `advance_one_step` does not check status — Group C's task 13 owns transitions. Out of Group B scope.

### D6. GameState mutability model
All mutations are methods taking `&mut self` on `GameState` in `state.rs`:
- `change_direction(&mut self, new_direction: Direction) -> bool` (task 6)
- `advance_one_step(&mut self)` (task 7)
Group B adds NO setters for `score`, `food`, or `status`, and NO getters beyond Group A's five accessors. The snake stays private; `GameState` reaches its mutators through those two `&mut self` methods only.

### D7. File placement
NO new files. Extend existing Group A files only:

| File | Group A size | Δ lines (est.) | Proj. total | Fits 200 cap |
|---|---|---|---|---|
| `src/game/state.rs` | 79 | +45 | ~125 | ✓ |
| `src/game/snake.rs` | 29 | +12 | ~42 | ✓ |
| `src/game/direction.rs` | 18 | +18 | ~36 | ✓ |
| `src/game/position.rs` | 5 | +10 | ~15 | ✓ |
| `src/main.rs`, `src/game.rs`, `src/game/food.rs`, `Cargo.toml` | — | 0 | — | untouched |

Projected largest file ~125 lines (≤200 total, ≤125 non-comment heuristic borderline acceptable: the ~125 estimate INCLUDES doc comments, so the non-comment count stays well under 125). All method bodies ≤ 10 lines. Rules: depth max 1; args ≤ 2 everywhere (`change_direction`: 1 arg; `advance`: 2 args; `initial_setup`: 0).

---

## 2. Exact Contents per File

### 2.1 `src/game/state.rs` — additions (after the existing code, no reordering, no removals)

Named constants (inserted with the existing `const` cluster):
```rust
const INITIAL_SNAKE_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_BODY_AHEAD: Position = Position { x: 9, y: 12 };
const INITIAL_SNAKE_BODY_BEHIND: Position = Position { x: 8, y: 12 };
const INITIAL_DIRECTION: Direction = Direction::Right;
const INITIAL_FOOD_POSITION: Position = Position { x: 20, y: 12 };
```

New free function + initial-setup factory (encoded placement: directly after `is_inside_board`, keeping state-creation logic above `GameStatus` in reading order):
```rust
pub fn initial_setup() -> GameStateSetup {
    GameStateSetup {
        snake: Snake::new(Vec::from([INITIAL_SNAKE_HEAD, INITIAL_SNAKE_BODY_AHEAD, INITIAL_SNAKE_BODY_BEHIND])),
        food: Food::new(INITIAL_FOOD_POSITION),
        direction: INITIAL_DIRECTION,
    }
}

fn is_immediate_reversal(current: Direction, candidate: Direction) -> bool {
    candidate == current.opposite()
}
```
- Snake segments literal order `[head, body, tail-tip]` preserves the head-first invariant from `snake.rs`'s module doc.
- `is_immediate_reversal` is file-private (only `change_direction` calls it).

New methods appended to the existing `impl GameState` block (getter block stays first — getters then mutators, in TODO task order: task 6 before task 7):
```rust
impl GameState {   // existing block, append below `status()`
    pub fn change_direction(&mut self, new_direction: Direction) -> bool {
        if is_immediate_reversal(self.current_direction, new_direction) {
            return false;
        }
        self.current_direction = new_direction;
        true
    }

    pub fn advance_one_step(&mut self) {
        let next_head = self.snake.head() + self.current_direction.offset();
        self.snake.advance(next_head, true);
    }
}
```

### 2.2 `src/game/snake.rs` — addition (one method appended to existing `impl Snake`)
```rust
impl Snake {
    // existing new/head/segments/length unchanged, then:
    pub fn advance(&mut self, next_head: Position, should_remove_tail: bool) {
        self.segments.insert(0, next_head);
        if should_remove_tail {
            self.segments.pop();
        }
    }
}
```

### 2.3 `src/game/direction.rs` — additions
```rust
use crate::game::position::Position;

const ZERO_GRID_STEP: i32 = 0;
const SINGLE_GRID_STEP: i32 = 1;
```
and inside the existing `impl Direction` (after `opposite`):
```rust
pub fn offset(self) -> Position {
    match self {
        Direction::Up => Position { x: ZERO_GRID_STEP, y: -SINGLE_GRID_STEP },
        Direction::Down => Position { x: ZERO_GRID_STEP, y: SINGLE_GRID_STEP },
        Direction::Left => Position { x: -SINGLE_GRID_STEP, y: ZERO_GRID_STEP },
        Direction::Right => Position { x: SINGLE_GRID_STEP, y: ZERO_GRID_STEP },
    }
}
```
(Encoded decision: BOTH constants are used — zero-magic-number purity, consistent with Group A's `MIN_AVAILABLE_COORDINATE` precedent. `Up` DECREASES y (terminal-grid orientation: y grows downward — §1 D4). All match arms exhaustive. If a reviewer flags these consts as ceremony, that objection is pre-decided: the constants stand.)

### 2.4 `src/game/position.rs` — addition
```rust
use std::ops::Add;

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
```
(Updates the existing derive attrs in NO way; `PartialEq` etc. remain.)

### 2.5 Untouched files
`Cargo.toml` (rand stays declared-unconsumed), `src/main.rs`, `src/game.rs`, `src/game/food.rs`.

---

## 3. High-Level Approach (ordered)

1. Git: confirm the current branch is `feat/phase1a-core-game-model` (branch exists from workflow Step 2; verified before plan write: `git status` reports `On branch feat/phase1a-core-game-model`). NO branch creation in this group.
2. Implement in dependency order (leaf-up, mirroring Group A's method): `src/game/position.rs` (Add impl) → `src/game/direction.rs` (offset + step constants) → `src/game/snake.rs` (advance) → `src/game/state.rs` (constants + `initial_setup` + `is_immediate_reversal` + the two `GameState` methods).
3. Read-only self-verification with §5 checklist after each file (no compilation — toolchain absent by design).
4. Commits (see §4), one logical change each.

---

## 4. Git Handling

- Branch: stay on `feat/phase1a-core-game-model`. No push, no merge, no `main` writes (workflow Step 5 owns those).
- Encoded decision (one plan, no alternatives): implement all files first, then make exactly three commits in this order, staging only the listed paths:
  1. `feat: add offset math to map directions onto the grid` — files: `src/game/position.rs`, `src/game/direction.rs`.
  2. `feat: implement snake advance with growth flag` — tasks 7–8 — file: `src/game/snake.rs`.
  3. `feat: add initial state, direction rule, and one-step advance to game state` — tasks 5–7 — file: `src/game/state.rs`.
- Before each commit: `git status` + read `.gitignore`; stage ONLY files that match no ignore pattern (`src/*.rs` never matches `target/` or `dist/` patterns — no `target/` exists since nothing compiles).

---

## 5. Verification (no compiler — manual checks, ordered)

1. **Module wiring closed-loop:** no new modules added → re-check that `src/game.rs` still lists exactly 5 `pub mod` lines matching `src/game/*.rs` case-sensitively.
2. **Use-path consistency:** `use crate::game::position::Position;` compiles-by-reading in `direction.rs`; `std::ops::Add` import present in `position.rs`; NO new `use` needed in `snake.rs` (Position already imported); `state.rs` already imports `Direction`, `Food`, `Position`, `Snake`.
3. **Syntax read-through:** delimiter balance; `#[derive]` untouched; `self` receiver only where encoded (`offset(self)`, `opposite(self)`); `&mut self` on `change_direction`/`advance_one_step`/`advance`; `&self` on getters.
4. **TODO task 5 coverage audit:** snake length exactly 3 via segments `[head(10,12), body(9,12), body(8,12)]` ✓ 2 body + 1 head ✓; direction Right ✓; score 0 ✓ (`INITIAL_SCORE` untouched); all segments in `0..=WIDTH-1`/`0..=HEIGHT-1` ✓; food (20,12) inside and non-overlapping ✓; coordinates are named constants, not inlined literals.
5. **TODO task 6 coverage audit:** all 4 invalid pairs rejected (`candidate == current.opposite()` covers exactly Right→Left, Left→Right, Up→Down, Down→Up); all 8 perpendicular pairs accepted; rule lives in `state.rs` with zero terminal/keyboard coupling; return-type contract documented as §1 D3 (`bool`).
6. **TODO task 7 coverage audit:** step (1) next head = `head() + current_direction.offset()`; step (2) `insert(0, next_head)`; step (3) tail shift via index semantics + `pop()`; step (4) length preserved when `should_remove_tail == true`; no terminal/input dependency; single-step advance callable headlessly (`GameState::advance_one_step` on a value constructed via `GameState::new(initial_setup())` — callable without any I/O).
7. **TODO task 8 coverage audit:** `Snake::advance(next_head, false)` yields `length == before + 1` (new head in `[0]`, tail retained in `[last]`); exactly +1, not +2; single movement routine shared by both paths — no duplicated insert logic, no separate `grow()` method.
8. **Rule audit:** file line counts (projected table §1 D7); method lengths ≤ 50; args ≤ 2; depth ≤ 2; private-by-default (the only `pub` items are: existing Group A surface + `initial_setup`, `change_direction`, `advance_one_step`, `advance`, `offset`, `Add` impl — each required by cross-module function); single-section conditions (`is_immediate_reversal` extracted; the `if should_remove_tail` check is single-section); no commented-out code; named constants (no bare ±1/±0 where `SINGLE_GRID_STEP`/`ZERO_GRID_STEP` apply); real newlines, no literal `\n`.
9. **gitignore compliance:** after commits, `git status` clean (except pre-existing untracked TODO-3/TODO-4 files and deleted `.kilo/plans/.gitkeep` — NOT owned by this step; leave untouched); nothing ignored staged.
10. **Exclusions respected:** no `#[cfg(test)]`, no tests, no rand usage, no collision/food/score/status-merge logic, no Docker files, no README/project-info edits.

---

## 6. Explicit Non-Goals for Group B (scope fence)

- NO food placement, NO rand consumption (task 9 = Group C). `Cargo.toml` untouched; `rand` remains declared-but-unused.
- NO consumption/score increment, NO food replacement (task 10). The `should_remove_tail: false` trigger is NOT wired to food detection here.
- NO boundary collision (task 11) — `advance_one_step` may compute an out-of-board next head and applies it mechanically; the loss reaction is Group C. (Off-board coordinates like `(−1, 12)` or `(40, 12)` are representable — `i32` was chosen by Group A exactly for this transient.)
- NO self-collision (task 12), NO game-state transitions (task 13 — `status` stays `WaitingToStart` on every Group B code path).
- NO tests (task 14 = Group D). Code must be shape-compatible with the test checklist above (deterministic, headless-callable).
- NO README (task 15), NO `.agent/project-structure.md` edits (step 4.4 owns docs), NO project-info edits (Step 6).
- NO changes to `src/main.rs`, `src/game.rs`, `src/game/food.rs`.

---

## 7. Notes for Later Steps (handoff snippets — do not act on them in this step)

- **4.3 (code-reviewer/code-simplifier):** pre-decided points that are OUT of fix bounds without a plan amendment: `bool` return of `change_direction`; `should_remove_tail: bool` flag over a `grow()` method; `insert(0,…)+pop()` over swap-based movement; `Position` `Add` impl; `initial_setup()` in `state.rs` over a `GameState::initial()` constructor; fixed initial food over rand; `Up` = y−1 orientation; the `ZERO_GRID_STEP`/`SINGLE_GRID_STEP` constants.
- **4.4 (docs-specialist):** minimal module docs only where semantics warrant them (candidates: a one-line doc comment on `advance` stating the head-first + tail-drop semantics; `state.rs` module doc may gain the fixed-initial-food fact). Update `.agent/project-structure.md` only if descriptions change — file LIST unchanged (no new files).
- **4.6 (implementer):** append `[DONE]` to task headings 5, 6, 7, 8 ONLY (byte-preserving edit; overwrite-todo-file-prevention rule), commit.
- **Group C handoff:** task 10 wires consumption → `snake.advance(next_head, false)`; task 11/12 receive `next_head`/segments read access via existing getters; task 13 owns transitions. `advance_one_step(&mut self)` currently hardcodes `true` for `should_remove_tail`; Group C will refactor the step to derive the flag from a food-eaten check (`self.snake.head() + offset == food.position()`) — this future refactor is EXPECTED and must not be pre-empted now.

---

## 8. Cross-Check Against TODO Tasks 5–8

| TODO requirement | Where satisfied in this plan |
|---|---|
| 5: exactly 3 segments (2 body + 1 head) | §2.1 constants; segment vector `[head, body, body]` |
| 5: initial direction Right | `INITIAL_DIRECTION` |
| 5: initial score 0 | `GameState::new` seeds `INITIAL_SCORE` (Group A, untouched) |
| 5: snake starts valid inside board | (10,12), (9,12), (8,12) all within 0..=39 / 0..=24 |
| 5: food inside area, not overlapping snake | (20,12), disjoint from x 8..10 row 12 |
| 5: exact coordinates = implementation detail | D2 documents the chosen values + constants |
| 6: opposite-reversal rule enforced | §1 D3 `is_immediate_reversal` via `opposite()`; all 4 invalid pairs covered |
| 6: perpendicular changes valid | not blocked; same-direction accepted as no-op |
| 6: rule in domain logic, not terminal-coupled | `GameState::change_direction` in `state.rs`; zero I/O |
| 6: exact API is an implementation decision | D3 encodes exact signature + `bool` return + justification |
| 7: next head from current direction | `head() + current_direction.offset()` |
| 7: add new head, move remaining body | `insert(0, next_head)` + index shift + `pop()` |
| 7: preserve length unless growth requested | `should_remove_tail: true` → length unchanged |
| 7: independent from terminal/keyboard | pure domain methods, headless-callable |
| 7: advance one step without interactive terminal | `GameStateNew(load) → advance_one_step()` path in §5.6 |
| 8: grow by exactly +1 segment | `advance(…, false)`: head in, tail retained |
| 8: new head added normally, tail not removed that step | same flag path |
| 8: no duplicated movement logic (`grow`/`should_remove_tail` preference) | single `Snake::advance` routine; no `grow()` method exists |
| Implementation Constraints (no engine/ECS/terminal/input/etc.) | §6 scope fence; plain structs/methods only |
| Out of Scope (terminal/interactive/Docker/rand-scoping/test/README) | §6 + §5.10 |
| Global plan rulings (rand-only dep, no tests authored, rule caps, private-by-default) | §1 D2/D6/D7 + §2 + rule audit |

Plan verified against all four TODO tasks and both TODO constraint sections — correct as written.
