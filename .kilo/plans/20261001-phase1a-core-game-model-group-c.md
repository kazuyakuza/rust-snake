# Implementation Plan — Phase 1A, Group C (TODO Tasks 9–13)

**Source:** `.agent/todos/20261001/20261001-todo-2.md` — tasks 9, 10, 11, 12, 13 (+ "Implementation Constraints" + "Out of Scope" bind all code).
**Scope of this plan is step 4.1b ONLY (analysis & planning) for Group C.** Global plan decision 10: this TODO has NO front-end tasks; 4.1a/4.5a do not apply.
**Global binding plan:** `.kilo/plans/20261001-phase1a-core-game-model.md` — its Technical & Architecture Decisions are NOT re-discussed here; this plan specializes them into exact code shapes for Group C.
**Built on Group B output** (`a0029e0`, `b721a8c` + simplifier pass `4150310`; adherence: ADHERENT): existing files are extended, NEVER restructured — except the ONE pre-authorized split from the Group B simplifier handoff (state.rs pre-split before exceeding ~125 code lines), encoded here in §1 D1. `rand` is declared in `Cargo.toml` as `rand = "0.8"` and is consumed starting with this group.

**Current branch:** `feat/phase1a-core-game-model` (verified). No branch creation in this group.

---

## 0. TODO Task Details (quoted, canonical)

### Task 9 — Implement Food Placement
- The food must: **"Be inside the board." / "Not overlap any Snake segment."**
- "Food placement should use random selection."
- "The implementation should account for the possibility that a randomly selected position is already occupied by the Snake."
- "If the board has no free cells remaining, the implementation should handle that situation explicitly rather than entering an infinite loop."
- "The exact behavior when the entire board is occupied can be kept simple, since this is an edge case outside normal gameplay."

### Task 10 — Implement Food Consumption and Scoring
- "Food is consumed when: **Snake head position == Food position**"
- When food is consumed: "**Increase the score by exactly `1`.** / **Increase the Snake length by exactly `1`.** / **Generate a new valid food position.**"
- "The score must start at `0`."
- "No additional scoring system is required."

### Task 11 — Implement Boundary Collision Detection
- "The Snake loses when its head moves outside the valid board coordinates."
- "There must be **no wrap-around behavior**."
- "Keep collision detection independent from rendering and terminal behavior."

### Task 12 — Implement Self-Collision Detection
- "The Snake loses when the head occupies the same cell as one of its body segments."
- "The implementation should account for the normal movement behavior of the tail when determining whether a collision occurs."
- "The resulting logic should expose a simple way for the game state to determine whether the current movement results in Game Over."

### Task 13 — Define Game State Transitions
- "Introduce the minimum state information necessary to distinguish between the major game states": `Waiting to start | Playing | Game Over`.
- "The terminal UI for these states will be implemented later."
- "For this phase, only the underlying state representation and transitions required by the game logic need to exist."
- "Do not implement the terminal messages yet."

### Constraints & Out of Scope (bind everything below)
- No engine, ECS, rendering, terminal, keyboard/input, networking, persistence, configuration systems, unnecessary abstractions. Domain stays terminal-free.
- Out of scope remains: NO tests (task 14 = Group D), NO README (task 15 = Group D), no terminal messages/UI, no Docker files, no `Cargo.lock`.

---

## 1. Encoded Decisions (one per formerly-open point — no open choices)

### D1. Module split (THE pre-split, per Group B handoff) — exact files

Create TWO new modules and move the initial-setup block out of `state.rs`:

| File | Operation | What moves/lives there | Proj. total lines (code lines) |
|---|---|---|---|
| `src/game/setup.rs` | NEW | `GameStateSetup` struct + `initial_setup()` + the five `INITIAL_*` constants (moved verbatim from `state.rs`) | ~40 (~26) |
| `src/game/collision.rs` | NEW | Boundary predicate + body-collision predicate (tasks 11–12) | ~25 (~16) |
| `src/game/food_placement.rs` | NEW | Random free-cell selection (task 9) | ~35 (~24) |
| `src/game/state.rs` | MODIFY | drop moved setup block; keep dims/bounds/status/`GameState`; add consumption/scoring wiring + transitions (tasks 10, 13) | ~118 (~95) |
| `src/game/food.rs` | MODIFY | add `occupies` predicate (task 10) | ~19 (~14) |
| `src/game.rs` | MODIFY | 5 → 8 `pub mod` lines | 8 (8) |

`position.rs`, `direction.rs`, `snake.rs`, `main.rs`, `Cargo.toml`: untouched. All files ≤ 200 hard cap; largest code-line count ≈ 95 (state.rs), under the ~125 ideal. The `is_playing` gate, `respawn_food`, and `enter_game_over` etc. keep every method ≤ 50 lines.

- **Why `setup.rs` and not more moves:** the moved block is byte-identical content, no logic change; it buys ~22 code lines back in `state.rs` so Group C's additions fit the ideal cap. Split of concerns reads naturally: `setup.rs` = how a game begins; `state.rs` = how a game evolves.

### D2. rand API (task 9) — `rand::thread_rng()` local handle + `Rng::gen_range` inclusive

rand 0.8 API (verified against docs.rs rand 0.8.5 — NOT the renamed 0.9/0.10 `random_range`/`rng()` forms):
```rust
use rand::{rngs::ThreadRng, Rng};
```
- The generator is a **local variable inside the placement function** (`let mut board_rng = rand::thread_rng();`) — NOT a field on `GameState` (keeps `GameState` pure, keeps its `Clone` derive usable, and keeps `advance_one_step` free of generator state).
- Ranges: `board_rng.gen_range(MIN_AVAILABLE_COORDINATE..=WIDTH - 1)` and `..=HEIGHT - 1` — inclusive per-axis ranges, matching the canonical bounds predicate exactly.
- NOT `rand::random()` (its whole-type range for `i32` would require modulo → bias/casting ceremony) and NOT `std::random`-style hacks.

### D3. Food placement function shape (task 9) — exact signature, file, visibility

In `src/game/food_placement.rs`:
```rust
pub fn choose_food_position(occupied: &[Position]) -> Option<Position>
```
- **Injectable occupied set** (a position slice, not a `&GameState` reference): `choose_food_position(self.snake.segments())` is call-shaped AND Group D can pass a hand-built slice for deterministic tests (task 14: "Food placement constraints").
- **Free-cell count check FIRST (explicit no-free-cells handling):**
```rust
fn is_board_full(occupied: &[Position]) -> bool {
    occupied.len() >= TOTAL_BOARD_CELLS
}
```
with `const TOTAL_BOARD_CELLS: usize = (WIDTH * HEIGHT) as usize;` (importing `WIDTH`/`HEIGHT` from `state.rs`). When full → return `None` immediately; the loop below is never entered; **no infinite loop is possible**. When not full, the retry loop terminates with probability 1 (expected retries ≈ snake length / board cells).
- **Occupied-retry loop:**
```rust
let mut board_rng = rand::thread_rng();
loop {
    let candidate = random_board_position(&mut board_rng);
    if !occupied.contains(&candidate) {
        return Some(candidate);
    }
}
```
with helper `fn random_board_position(board_rng: &mut ThreadRng) -> Position` (1 arg) filling both axes via `gen_range` (D2).
- **Board-full behavior — picked (simplest sound option, per "kept simple"):** `None` means "no free cell"; the caller (`GameState::respawn_food`) then **keeps the existing food position unchanged** (no respawn). Rationale: the brief (§8) defines ONLY boundary and self-collision as loss conditions — turning board-full into a Game Over would invent a third loss rule; and food cannot stay "eaten-nowhere" because the board is necessarily snake-covered in that state, so keeping the just-eaten cell as the food cell is the minimal, non-crashing, non-looping behavior. This path is deterministic and directly testable by Group D via a fully-occupied slice.
- `random_board_position` is file-private; `choose_food_position` is `pub` (called from `state.rs`).

### D4. Consumption predicate (task 10) — `Food::occupies` in `food.rs`

Where the `head == food` comparison lives: on `Food` in `src/game/food.rs` (the type that owns the food's position):
```rust
pub fn occupies(&self, position: Position) -> bool {
    self.position == position
}
```
- NOT in `state.rs` (comparison-by-proxy with field access bloat) and NOT in `collision.rs` (eating is not a collision). One-line predicate, self-documenting call site: `self.food.occupies(next_head)`.

### D5. Collision predicates (tasks 11–12) — exact predicates, file, method names

Both death predicates live in `src/game/collision.rs` as pure free functions (terminal-free, no `GameState` dependency — only shared constants via `state.rs`):

**Boundary (task 11)** — next head outside `0..=WIDTH - 1` × `0..=HEIGHT - 1` → lose; no wrap:
```rust
pub fn is_outside_board(next_head: Position) -> bool {
    !is_inside_board(next_head)
}
```
It delegates to the EXISTING canonical `is_inside_board` in `state.rs` — the bounds rule stays centralized (Group A precedent), `collision.rs` only names the loss-negation. `i32` transient off-board values (e.g. `(−1, 12)`, `(40, 12)`) are representable; they hit this predicate, never wrap.

**Self-collision (task 12)** — next head equals a body segment AFTER the tail vacates (tail exclusion):
```rust
pub fn collides_with_body(next_head: Position, segments: &[Position]) -> bool {
    head_to_body_slice(segments).iter().any(|segment| *segment == next_head)
}
```
with the helper defined exactly as (no alternative forms):
```rust
fn head_to_body_slice(segments: &[Position]) -> &[Position] {
    if segments.is_empty() {
        return segments;
    }
    &segments[..segments.len() - TAIL_SEGMENT_COUNT]
}
```
with `const TAIL_SEGMENT_COUNT: usize = 1;` — named constant replacing the bare 1 (no-magic-numbers rule, same pattern as `SINGLE_GRID_STEP`).
- **Tail rule encoded:** the check runs BEFORE the move and against all segments EXCEPT the last one (the current tail), because in a normal step the tail vacates its cell the same tick (TODO task 12 "account for the normal movement behavior of the tail"). In a growth step the tail does NOT vacate — but that step is provably safe anyway: the growth step's new head lands on the food cell, and food never overlaps any snake segment (placement invariant of task 9 + initial state of task 5), so `next_head` cannot be the tail cell in that case. Hence excluding the last segment UNCONDITIONALLY is provably sound and uses ONE predicate shape (2 args, no extra flag param, no 3-arg violation).
- Segments with length 1 → empty slice → no self-collision possible (correct: head cannot hit itself).
- Method names and ownership: `collision::is_outside_board` and `collision::collides_with_body` are the "simple way for the game state to determine whether the current movement results in Game Over" (task 12), consumed by `advance_one_step` (D7).

### D6. GameStatus transitions (task 13) — representation reused, three encoded transitions

- Representation: the EXISTING `GameStatus` enum (`WaitingToStart | Playing | GameOver`, Group A) — NOT re-created, NOT extended. Minimum info ✓.
- **Initial status (picked):** `GameState::new` seeds `WaitingToStart` (Group A, unchanged). `initial_setup()` / `setup.rs` produce NO status — the setup object stays (snake, food, direction) exactly as it is. No new constructor.
- Transition methods, appended to the existing `impl GameState`:
```rust
pub fn start_playing(&mut self) {
    self.status = GameStatus::Playing;
}

pub fn enter_game_over(&mut self) {
    self.status = GameStatus::GameOver;
}
```
  - `WaitingToStart → Playing` is represented by `start_playing()`, called LATER by the terminal/start phase (brief §11 press-any-key). It has NO caller in this phase (Group D tests will call it; until compilation is in play this cannot produce a build error — the dead-code lint is expected and pre-decided, see §7).
  - `Playing/anything → GameOver` is `enter_game_over`, called by `advance_one_step` on either death predicate (D7).
  - No "restart" transition (brief §12: no restart system). No scores/food resets.
- **Tick gate (picked, logic-level only):** `advance_one_step` does nothing unless the game is playing:
```rust
if !self.is_playing() {
    return;
}
```
  with private predicate `fn is_playing(&self) -> bool { self.status == GameStatus::Playing }` — single-section condition ✓ (extraction rule), keeps the state machine honest (a `WaitingToStart` game cannot be advanced headlessly by accident; Phase 1B's loop ticks only after `start_playing`).
- NO terminal messages, NO rendering keys — pure representation + logic transitions (task 13 + Out-of-Scope).

### D7. Final `advance_one_step` — full per-tick order per brief §7

Exact final form, replacing the Group B body (`state.rs`):
```rust
pub fn advance_one_step(&mut self) {
    if !self.is_playing() {
        return;
    }
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
- Order matches brief §7: (2) calc next head → (3) boundary check → (4) self check → (5) move → (6/7) eat check → score/grow/new-food (input step 1 and render step 8 live in later phases; brief explicitly allows adjust). Consumption is checked BEFORE the move so the move can carry the growth flag — semantics equal to the brief's post-move check ("head moves onto the food cell").
- **Growth wiring (Group B handoff):** the flag flips off exactly in the consuming move — `self.snake.advance(next_head, !will_consume)` sends `should_remove_tail = false` ⇒ new head added normally, tail retained ⇒ length exactly +1 (TODO task 10 + task 8's shared-routine requirement; NO separate `grow()`).
- **Score:** `+= SCORE_INCREMENT` with `const SCORE_INCREMENT: i32 = 1;` added to state.rs's existing const cluster — exactly +1, no extra scoring. Score still starts at 0 (`INITIAL_SCORE`, Group A, untouched).
- **Respawn:**
```rust
fn respawn_food(&mut self) {
    let occupied: Vec<Position> = self.snake.segments().to_vec();
    if let Some(new_food_position) = food_placement::choose_food_position(&occupied) {
        self.food = Food::new(new_food_position);
    }
}
```
  - The snapshot `to_vec()` is REQUIRED: `choose_food_position(&self.snake.segments())` borrows `self` immutably while the `if let` body assigns `self.food` → borrow-check conflict (E0502). Copying the ≤1000-element segment slice trivializes it; a junior implementer would hit this exact error — the copy is mandatory, do not "simplify" it away. Returning `None` (board full) keeps the current food, per D3.
  - Always exactly one food on the board during play (brief §9): the initial/consumed food is never deleted; a respawn only OVERWRITES it with a fresh valid cell when a free cell exists.
- New imports needed in `state.rs`: `use crate::game::collision;` and `use crate::game::food_placement;` plus `use crate::game::setup::GameStateSetup;` (replacing the locally-defined struct). `Food`, `Position`, `Snake`, `Direction` imports remain from before.

### D8. `setup.rs` exact contents (the move, byte-faithful)

`src/game/setup.rs` receives moved VERBATIM from `state.rs` (no edits to bodies or names): the head-to-tail comment, the five constants (`INITIAL_SNAKE_HEAD`, `INITIAL_SNAKE_BODY_AHEAD`, `INITIAL_SNAKE_BODY_BEHIND`, `INITIAL_DIRECTION`, `INITIAL_FOOD_POSITION`), the `GameStateSetup` struct (fields stay `pub` exactly as today), and `initial_setup()`. Imports in `setup.rs`: `Direction`, `Food`, `Position`, `Snake` via the existing `crate::game::` paths. `state.rs` keeps constructing games ONLY through `GameState::new(setup::initial_setup())` — no surviving state.rs item references the moved constants (verified: `INITIAL_SCORE` stays in `state.rs`; the other five `INITIAL_*` coordinate/direction/food constants all move).

### D9. `game.rs` wiring — exact final contents

```rust
pub mod collision;
pub mod direction;
pub mod food;
pub mod food_placement;
pub mod position;
pub mod setup;
pub mod snake;
pub mod state;
```
(alphabetical; 8 modules match the 8 files in `src/game/` case-sensitively.)

### D10. Projected line-count table (rule audit basis)

| File | Now | Δ | After (total / non-comment) | Cap check |
|---|---|---|---|---|
| `src/game/state.rs` | 117 | −26 moved, +34 added | ~118 / ~95 | ✓ (<125 code) |
| `src/game/setup.rs` | — | new | ~40 / ~26 | ✓ |
| `src/game/collision.rs` | — | new | ~25 / ~16 | ✓ |
| `src/game/food_placement.rs` | — | new | ~35 / ~24 | ✓ |
| `src/game/food.rs` | 16 | +3 | ~19 / ~14 | ✓ |
| `src/game.rs` | 5 | +3 | 8 | ✓ (config-class) |
| `position.rs` / `direction.rs` / `snake.rs` / `main.rs` / `Cargo.toml` | — | 0 | untouched | ✓ |

Rule compliance: every new fn ≤ 2 args (`choose_food_position`: 1; `random_board_position`: 1; `collides_with_body`: 2; `Food::occupies`: self+1; transitions: 0); bodies ≤ 19 lines (`advance_one_step`), depth ≤ 2; all new items private except SIX `pub` additions required cross-module (`choose_food_position`, `is_outside_board`, `collides_with_body`, `occupies`, `start_playing`, `enter_game_over` — each with an external caller now or in Group D/Phase 1B); every new condition single-section; named constants only (`SCORE_INCREMENT`, `TAIL_SEGMENT_COUNT`, `TOTAL_BOARD_CELLS`); no commented-out code; real newlines.

---

## 2. Exact Contents per File

### 2.1 `src/game/setup.rs` — NEW (full file)
```rust
use crate::game::direction::Direction;
use crate::game::food::Food;
use crate::game::position::Position;
use crate::game::snake::Snake;

// Head-to-tail start: `HEAD` leads and moves `Right`, `BODY_AHEAD` sits adjacent
// to it, and `BODY_BEHIND` is the tail tip. The initial food shares row 12 but
// stays clear of every snake cell and inside the playable area.
const INITIAL_SNAKE_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_BODY_AHEAD: Position = Position { x: 9, y: 12 };
const INITIAL_SNAKE_BODY_BEHIND: Position = Position { x: 8, y: 12 };
const INITIAL_DIRECTION: Direction = Direction::Right;
const INITIAL_FOOD_POSITION: Position = Position { x: 20, y: 12 };

pub fn initial_setup() -> GameStateSetup {
    GameStateSetup {
        snake: Snake::new(Vec::from([INITIAL_SNAKE_HEAD, INITIAL_SNAKE_BODY_AHEAD, INITIAL_SNAKE_BODY_BEHIND])),
        food: Food::new(INITIAL_FOOD_POSITION),
        direction: INITIAL_DIRECTION,
    }
}

pub struct GameStateSetup {
    pub snake: Snake,
    pub food: Food,
    pub direction: Direction,
}
```
(Exact text as moved from `state.rs` lines 19–59, listed constants → struct order preserved; the leading blank line/comment placement matches §1 D8.)

### 2.2 `src/game/collision.rs` — NEW (full file)
```rust
//! Death predicates for one movement step: boundary exit and body overlap.
//! Both are pure and terminal-free; `state.rs` applies them to a next head.

use crate::game::position::Position;
use crate::game::state::is_inside_board;

const TAIL_SEGMENT_COUNT: usize = 1;

pub fn is_outside_board(next_head: Position) -> bool {
    !is_inside_board(next_head)
}

/// `true` when `next_head` lands on any segment except the current tail: the
/// tail vacates its cell in the same step, and the only step where it would
/// not (food growth) cannot enter it, since food never overlaps the snake.
pub fn collides_with_body(next_head: Position, segments: &[Position]) -> bool {
    head_to_body_slice(segments).iter().any(|segment| *segment == next_head)
}

fn head_to_body_slice(segments: &[Position]) -> &[Position] {
    if segments.is_empty() {
        return segments;
    }
    &segments[..segments.len() - TAIL_SEGMENT_COUNT]
}
```
(Boundaries for the loss definition are the canonical inclusive `0..=WIDTH - 1 × 0..=HEIGHT - 1` — enforced by `is_inside_board`, negated here. No wrap.)

### 2.3 `src/game/food_placement.rs` — NEW (full file)
```rust
use crate::game::position::Position;
use crate::game::state::{HEIGHT, WIDTH, MIN_AVAILABLE_COORDINATE};
use rand::Rng;
use rand::rngs::ThreadRng;

const TOTAL_BOARD_CELLS: usize = (WIDTH * HEIGHT) as usize;

/// Choose a random free cell for the food, retrying on snake-occupied picks.
/// Returns `None` when every cell is occupied: the explicit signal that no
/// respawn is possible, with no loop iteration entered at all.
pub fn choose_food_position(occupied: &[Position]) -> Option<Position> {
    if is_board_full(occupied) {
        return None;
    }
    let mut board_rng = rand::thread_rng();
    loop {
        let candidate = random_board_position(&mut board_rng);
        if !occupied.contains(&candidate) {
            return Some(candidate);
        }
    }
}

fn is_board_full(occupied: &[Position]) -> bool {
    occupied.len() >= TOTAL_BOARD_CELLS
}

fn random_board_position(board_rng: &mut ThreadRng) -> Position {
    Position {
        x: board_rng.gen_range(MIN_AVAILABLE_COORDINATE..=WIDTH - 1),
        y: board_rng.gen_range(MIN_AVAILABLE_COORDINATE..=HEIGHT - 1),
    }
}
```

### 2.4 `src/game/food.rs` — MODIFY (add ONE method to the existing `impl Food`)
```rust
pub fn occupies(&self, position: Position) -> bool {
    self.position == position
}
```
(Output becomes `Position == Self::position` — the consume test of task 10. `Food` derives `Copy`; `position` param passes by value like `position()` returns.)

### 2.5 `src/game/state.rs` — MODIFY (exact deltas)

1. **Imports:** replace the current four `use crate::game::…` lines with:
```rust
use crate::game::collision;
use crate::game::direction::Direction;
use crate::game::food::Food;
use crate::game::food_placement;
use crate::game::position::Position;
use crate::game::setup::GameStateSetup;
use crate::game::snake::Snake;
```
2. **Const block** becomes exactly:
```rust
pub const WIDTH: i32 = 40;
pub const HEIGHT: i32 = 25;

const INITIAL_SCORE: i32 = 0;
pub const MIN_AVAILABLE_COORDINATE: i32 = 0;
pub const SCORE_INCREMENT: i32 = 1;
```
(`MIN_AVAILABLE_COORDINATE` gains `pub` — the single source of the lower bound for `food_placement.rs`; board dims stay centralized. `is_within_bounds`/`is_inside_board` stay untouched, `pub` stays on `is_inside_board`.)
3. **Remove** the five `INITIAL_*` coordinate/direction/food constants, their comment, `initial_setup()`, and the `GameStateSetup` struct (all now in `setup.rs`, consumed by no state.rs item besides the `GameStateSetup` type import for `GameState::new`).
4. **Keep** `GameStatus`, `GameState` (fields private), all five getters, `change_direction`, `is_immediate_reversal` — untouched.
5. **Append to the existing `impl GameState`** (below `advance_one_step`'s new form):
```rust
fn is_playing(&self) -> bool {
    self.status == GameStatus::Playing
}

pub fn start_playing(&mut self) {
    self.status = GameStatus::Playing;
}

pub fn enter_game_over(&mut self) {
    self.status = GameStatus::GameOver;
}

fn respawn_food(&mut self) {
    let occupied: Vec<Position> = self.snake.segments().to_vec();
    if let Some(new_food_position) = food_placement::choose_food_position(&occupied) {
        self.food = Food::new(new_food_position);
    }
}
```
6. **Replace** the existing `advance_one_step` body with the exact D7 form (order: gate → calc → boundary → self → move-with-flag → consume → score → respawn).

### 2.6 `src/game.rs` — MODIFY (full file per D9).

### 2.7 Untouched files
`Cargo.toml` (rand already declared — now consumed), `src/main.rs`, `src/game/position.rs`, `src/game/direction.rs`, `src/game/snake.rs`.

---

## 3. High-Level Approach (ordered)

1. Git: confirm branch `feat/phase1a-core-game-model` (already verified by this planner; implementer re-checks with `git branch --show-current`). NO branch creation, NO push, NO merge.
2. Implement in TWO waves so every commit leaves the module tree internally consistent (a `game.rs` entry must never point at a missing file):
   - Wave 1 (pure refactor, compiles-by-reading on its own): create `setup.rs` (content per D8/§2.1), strip the moved items from `state.rs` (imports/consts updated per §2.5 items 1–4), add the `setup` mod line to `game.rs` → commit 1.
   - Wave 2 (features): create `collision.rs` (per D5/§2.2), create `food_placement.rs` (per D3/§2.3), add the `occupies` method to `food.rs` (per D4), finish `game.rs` to the full 8-mod list (per D9), apply the `state.rs` transitions + respawn + new `advance_one_step` (per D6/D7/§2.5 items 5–6) → commits 2–4.
3. Read-only self-verification with §5 checklist after each file (NO compilation — toolchain absent by design; `cargo` is never run).
4. Commits per §4.

---

## 4. Git Handling

- Branch: stay on `feat/phase1a-core-game-model`. No push, no merge, no `main` writes (workflow Step 5 owns those).
- Exactly FOUR commits, in this order, staging ONLY the listed paths:
  1. `refactor: extract initial game setup into its own module` — files: `src/game/setup.rs`, `src/game/state.rs`, `src/game.rs`.
  2. `feat: add random food placement with free-cell handling` — tasks 9 — files: `src/game/food_placement.rs`, `src/game.rs`.
  3. `feat: add collision detection for board bounds and snake body` — tasks 11–12 — files: `src/game/collision.rs`, `src/game.rs`.
  4. `feat: wire food consumption, scoring, and game status transitions` — tasks 10, 13 — files: `src/game/state.rs`, `src/game/food.rs`.
- Before each commit: run `git status`, read `.gitignore`; stage ONLY files matching no ignore pattern (`src/*.rs` never matches `target/`/`dist/`; `target/` does not exist — nothing compiles). Existing untracked `.agent/todos/20261001/20261001-todo-3.md`, `todo-4.md`, and deleted `.kilo/plans/.gitkeep` are NOT owned by this step — leave untouched.

---

## 5. Verification (no compiler — manual checks, ordered)

1. **Module wiring closed-loop:** `src/game.rs` lists exactly 8 `pub mod` lines — `collision, direction, food, food_placement, position, setup, snake, state` — matching `src/game/*.rs` case-sensitively, one-to-one.
2. **Use-path consistency (+ borrow arithmetic):** `setup.rs` imports `Direction/Food/Position/Snake` and uses all four; `collision.rs` imports `Position` + `state::is_inside_board` (both used); `food_placement.rs` imports `Position`, `state::{WIDTH, HEIGHT, MIN_AVAILABLE_COORDINATE}`, `rand::Rng`, `rand::rngs::ThreadRng` — `Rng` trait in scope for `gen_range`, `ThreadRng` used as the helper param type; `state.rs` imports `collision`, `food_placement`, `setup::GameStateSetup` and still uses `Food`, `Position`, `Snake`, `Direction`. `respawn_food` snapshots `occupied` BEFORE the `if let` that mutates `self.food` (no simultaneous borrow — E0502 guard, §1 D7). `TOTAL_BOARD_CELLS` compares `usize == usize` (cast `(WIDTH * HEIGHT) as usize` correct: 40×25=1000).
3. **Syntax read-through:** delimiter balance; derives untouched; `&self`/`&mut self` receivers exactly as §2; `self` receiver only on `opposite`, `offset`, `occupies`, `is_playing`; `loop` in `choose_food_position` has exactly one exit (`return Some(candidate)`) plus the pre-loop `return None`; no trailing `;`-style or `\n` literal issues; real newlines.
4. **TODO task 9 audit:** placement is random (thread_rng + gen_range) ✓; `candidate` always in `0..=WIDTH - 1`/`0..=HEIGHT - 1` (inside board) ✓; `occupied.contains(&candidate)` retry accounts for "randomly selected position already occupied by the Snake" ✓; `is_board_full` check BEFORE the loop → `None` without iterating = explicit no-free-cells handling, NO infinite loop ✓; board-full behavior picked: respawn skipped, existing food kept (simple per TODO) ✓; injectable `occupied: &[Position]` for Group D ✓.
5. **TODO task 10 audit:** consume condition is exactly `self.food.occupies(next_head)` = head position == food position ✓; on consume: `score += SCORE_INCREMENT` = exactly +1 ✓, `advance(next_head, false)` = length exactly +1 (head in, tail retained — no double growth) ✓, `respawn_food()` generates a fresh valid position (in-bounds via random_board_position, snake-free via the occupied slice/dedup ✓ — note the snapshot is the POST-advance snake, so the new food cannot overlap the just-grown snake) ✓; score starts 0 (`INITIAL_SCORE` untouched) ✓; no additional scoring ✓.
6. **TODO task 11 audit:** `collision::is_outside_board(next_head)` fires on ANY axis out of `0..=WIDTH - 1`/`0..=HEIGHT - 1`, delegates to the canonical bounds predicate ✓; no wrap (no modulo/clamp anywhere in the tick path) ✓; loss transitions via `enter_game_over` ✓; zero terminal/rendering coupling ✓.
7. **TODO task 12 audit:** `collision::collides_with_body(next_head, self.snake.segments())` — head equals any body segment (tail excluded, tail-vacate accounted, growth step provably safe per doc comment) ✓; runs BEFORE the move ✓; "simple way for the game state to determine Game Over" = these predicates feeding `enter_game_over` ✓.
8. **TODO task 13 audit:** representation = existing 3-variant `GameStatus` (no new state info beyond the TODO minimum) ✓; transitions: `start_playing` (WaitingToStart→Playing, called later by terminal phase), `enter_game_over` (→GameOver, called by tick on death) ✓; `is_playing` gate keeps logic-level only; statuses never set outside these two methods; NO terminal messages ✓.
9. **Brief §7 order audit:** next-head calc → boundary → self → move → consume(+score/grow/new food) — exact sequence, with the growth flag folded into the move single routine.
10. **Rule audit:** line counts vs §1 D10 table; method bodies ≤ 50 (max 19): args ≤ 2 (max 2: `collides_with_body`, `advance` untouched); depth ≤ 2 (`if !self.is_playing()` = 1 block; `if let` inside `respawn_food` = 1); private-by-default (`is_board_full`, `random_board_position`, `head_to_body_slice`, `is_playing`, `respawn_food` private; the six cross-module `pub` items listed in D10); single-section conditions everywhere (no `&&`-chains added — `is_within_bounds` retains its two CONDITION-CLAUSE conjunction from Group A, out of scope); no commented-out code; named constants only (`SCORE_INCREMENT`, `TAIL_SEGMENT_COUNT`, `TOTAL_BOARD_CELLS`); derive list untouched on existing structs.
11. **gitignore compliance:** after each commit `git status` shows clean (except pre-existing TODO-3/TODO-4 untracked + `.kilo/plans/.gitkeep` deletion — untouched); nothing ignored staged.
12. **Exclusions respected:** no `#[cfg(test)]`, no tests, no terminal strings/messages, no rendering, no input handling, no Docker files, no README/project-info edits.

---

## 6. Explicit Non-Goals for Group C (scope fence)

- NO tests (task 14 = Group D) — but code must be shape-compatible with the checklist above (deterministic predicates; injectable `occupied` slice; headless `advance_one_step`).
- NO README (task 15 = Group D).
- NO terminal `Press any key to start` / `GAME OVER` strings, NO start-screen or game-over rendering (later phase); `start_playing` exists as logic only.
- NO new loss conditions beyond boundary + self collision (board-full does NOT set GameOver — food simply does not respawn, D3).
- NO wrap-around, NO speed/acceleration logic, NO input reversal auto-gating (Group B's `change_direction` gate wiring to keys is Phase 1B).
- NO changes to `Cargo.toml`, `src/main.rs`, `src/game/position.rs`, `src/game/direction.rs`, `src/game/snake.rs`.
- NO project-structure.md / context.md edits (step 4.4 owns docs; Step 6 owns context).

---

## 7. Notes for Later Steps (handoff — do not act on them in this step)

- **4.3 (code-reviewer/code-simplifier):** pre-decided OUT of fix bounds without plan amendment: `thread_rng()` + `gen_range` over alternatives; `Option<Position>` return of `choose_food_position`; board-full = keep old food (no invented GameOver); `Food::occupies` location; unconditional tail exclusion in `collides_with_body`; `is_playing` gate; `start_playing`/`enter_game_over` names; the `occupied.to_vec()` snapshot (borrow-check fix, NOT an optimization issue); `pub` on `MIN_AVAILABLE_COORDINATE`; the three new consts; the 3-file split layout.
- **4.4 (docs-specialist):** module docs already specified in §2 (one-liners for `collision.rs` + doc comments on `collides_with_body`); update `.agent/project-structure.md`: add `src/game/collision.rs`, `src/game/food_placement.rs`, `src/game/setup.rs` to the tree + describe `state.rs` as "game state, status transitions, and the per-tick move/consume/collide driver" instead of listing bounds only.
- **4.6 (implementer):** append `[DONE]` to task headings 9, 10, 11, 12, 13 ONLY (byte-preserving; overwrite-todo-file-prevention rule), commit.
- **Group D handoff (task 14 tests):** deterministic seams exist — `choose_food_position` takes an injectable `&[Position]` (full-board → `None` case directly testable); collision predicates take a next head + a slice (pure functions); `Food::occupies` / `change_direction` bool / `Snake::advance(…, false)` all return observable values or expose `segments()`/`head()`; transitions callable directly (`start_playing`, `enter_game_over`); randomized placement tests may assert only GUARANTEES (in-bounds, non-overlap, `Some` when a free cell exists) — never exact coordinates. NOTE: a `cargo check` in this state would emit `dead_code` for `start_playing` (no caller until Phase 1B/Group D tests) — expected, not a defect.
- **Phase 1B handoff:** terminal loop drives `change_direction` + `advance_one_step` at 120 ms ticks, calls `start_playing()` after the start-key press; render reads `status()`/`score()`/`food()`/`snake()`.

---

## 8. Cross-Check Against TODO Tasks 9–13

| TODO requirement | Where satisfied in this plan |
|---|---|
| 9: food inside the board | `random_board_position` ranges `MIN_AVAILABLE_COORDINATE..=WIDTH - 1` / `..=HEIGHT - 1` |
| 9: not overlapping any snake segment | `!occupied.contains(&candidate)` retry |
| 9: random selection | `rand 0.8` `thread_rng` + `gen_range` (D2) |
| 9: occupied pick handled | retry loop (D3) |
| 9: no free cells handled explicitly, no infinite loop | `is_board_full` pre-check returns `None` before any iteration |
| 9: board-full behavior kept simple | respawn skipped; existing food position stays (D3) |
| 10: consumed when head position == food position | `Food::occupies(next_head)` |
| 10: score +1 exactly | `self.score += SCORE_INCREMENT` (const 1) |
| 10: length +1 exactly | `advance(next_head, false)` on the consuming move (tail kept) |
| 10: new valid food generated | `respawn_food()` → `choose_food_position` (post-move segments) |
| 10: score starts 0 | `INITIAL_SCORE` (Group A, untouched) |
| 10: no additional scoring system | single counter, no bonuses |
| 11: loses when head moves outside valid coords | `collision::is_outside_board` → `enter_game_over`, BEFORE the move |
| 11: NO wrap-around | no mod/clamp; off-board `i32` transient detected and ends the tick |
| 11: independent from rendering/terminal | pure predicate module |
| 12: loses when head onto own body | `collision::collides_with_body` → `enter_game_over` |
| 12: accounts for normal tail movement | tail cell excluded from the check (vacates same tick unless growth; growth case provably safe) |
| 12: simple way for game state to know Game Over | the two predicates feed `enter_game_over()` directly in `advance_one_step` |
| 13: minimum representation of 3 states | existing `GameStatus` enum, no new fields |
| 13: transitions only, logic-level | `start_playing` / `enter_game_over` + `is_playing` gate |
| 13: no terminal messages | zero strings/render code anywhere |
| Implementation Constraints (small, idiomatic, terminal-free) | §6 fence + plain functions/methods, no new types beyond this design |
| Out of Scope (no terminal UI/loop/input/Docker/tests/README) | §6 + §5.11–5.12 |
| Global plan rulings (rand-only dep, rule caps, private-by-default, no tests authored) | D2/D3/D10 + §2 + rule audit |

Plan verified against all five TODO tasks and both TODO constraint sections — correct as written.
