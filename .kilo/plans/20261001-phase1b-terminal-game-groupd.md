# Implementation Plan — Phase 1B Group D (Tasks 9 & 10): Headless Gameplay-Flow Validation + Tests

- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` — Tasks **9** ("Validate the Complete Gameplay Flow") and **10** ("Add or Update Tests Where Appropriate"), plus its **Implementation Constraints** and **Out of Scope** sections.
- **Global plan**: `.kilo/plans/20261001-phase1b-terminal-game.md` (Group D row: tasks 9+10, marked NOT front-end — no 4.1a spec exists, none needed).
- **Branch**: `feat/phase1b-terminal-game` (already created — step 2 done). Do NOT create/switch branches. HEAD at plan time: `bda329a`. Working tree clean.
- **Version**: `0.2.0` (already bumped — step 3 done). Do NOT touch `Cargo.toml` version.
- **Sub-step**: 4.1b Implementation Plan => implementer executes it in 4.2. This plan is the ONLY artifact of this step. No code is written by the planner.

---

## 0. Scope

### In scope (exactly)

1. **Task 9** — executable headless evidence of the complete flow: one new integration test file `tests/gameplay_flow.rs` driving the sanctioned hooks (`GameState` domain API + `terminal::game_loop::tick` + `terminal::renderer::Renderer` over an in-memory buffer), PLUS a static-trace walkthrough of the `src/main.rs` wiring written as the `//!` module documentation of that test file (marked for mirroring into `docs/terminal-ui.md` in step 4.4).
2. **Task 10** — one new integration test file `tests/terminal_modules.rs` covering, headlessly only:
   - arrow-key → `Direction` mapping (incl. non-arrow press, key release, and reversal handling *through* `tick`);
   - `tick` loop-step semantics (directions applied chronologically, exactly one step per call, `GameOver` return, frame rendered into the buffer);
   - renderer output snapshot checks via a `&mut Vec<u8>` buffer (border corner count, glyph presence/counts, score line);
   - static guarantee that all six Phase 1A test files are untouched.
3. **Documentation touch-points** (executed in sub-step **4.4** by docs-specialist; content fully specified here):
   - `README.md` — test-section update (Phase 1B tests authored, executed in Phase 2 Docker).
   - `docs/terminal-ui.md` — "Headless Tests (Group D)" section + Status update.
   - `.agent/project-structure.md` — test file rows for the two new files.
4. **Git**: two `feat` commits on `feat/phase1b-terminal-game` (see §6). No push.

### Out of scope (implementer MUST NOT do)

- **No changes to any existing file** in `src/`, `tests/`, or `Cargo.toml`. Group D adds tests + (in 4.4) docs only. In particular:
  - No new dependency (integration tests reach `crossterm` types directly because `crossterm = "0.29"` is already a `[dependencies]` entry — package deps are available to `tests/`).
  - No `Renderer::into_inner`/accessor additions — the adopted capture pattern is `Renderer::new(&mut Vec<u8>)` (decision D22 below).
- **No tests for** (not headless-testable; excluded by decision):
  - `run_playing_loop` (120 ms sleeps, real event polling),
  - `drain_arrow_directions` / `drain_arrow_event` (poll the real crossterm event queue),
  - `TerminalHandle::enable`/`disable`/`Drop` (raw-mode + alternate-screen touch the real console),
  - start/game-over screens and `wait_for_any_key_press` (binary-private, real terminal),
  - full keyboard-driven interactive gameplay.
- **No execution**: no local Rust toolchain, no Docker in this phase (Docker = Phase 2). Verification is strictly static (§7). Tests are *authored* now.
- No restart flow, no timing changes, no refactor of Phase 1A code.

---

## 1. Verified Facts the Tests Depend On (source of truth, read from sources)

| Fact | Location |
|---|---|
| `initial_setup()`: snake head (10,12), body (9,12), (8,12); direction `Right`; fixed food (20,12) — same row as snake | `src/game/setup.rs` |
| `GameState::new(GameStateSetup)` → status `WaitingToStart`; `change_direction` rejects only the immediate reversal; `advance_one_step()` no-ops unless `Playing`; `start_playing()` / `enter_game_over()`; accessors `snake()` / `food()` / `score()` / `status()` / `current_direction()` | `src/game/state.rs` |
| `WIDTH = 40`, `HEIGHT = 25` (cell counts, coordinates `0..=39` / `0..=24`, no wrap); top-wall collision = next head `y < 0`; bottom-wall = `y > HEIGHT - 1` | `src/game/state.rs` |
| Self-collision excludes the vacating tail (`collides_with_body` slices off the last segment) | `src/game/collision.rs` |
| `GameStateSetup { snake, food, direction }` — **all fields `pub`**; `Snake::new(Vec<Position>)` and `Food::new(Position)` are `pub` ⇒ tests can build *deterministic custom setups* | `src/game/setup.rs`, `snake.rs`, `food.rs` |
| `Direction::offset()`: Up = (0,-1), Down = (0,+1), Left = (-1,0), Right = (+1,0) | `src/game/direction.rs` |
| `tick(state, directions: &[Direction], renderer) -> io::Result<GameStatus>`: applies each direction **in slot order** (reversals silently rejected by the domain), performs **exactly one** `advance_one_step()`, renders once, returns the new status; headless (no sleep, no real terminal) — sanctioned 3-param exception (D21) | `src/terminal/game_loop.rs` |
| `Renderer::new(output: W)`; `Renderer::render(&mut self, state: &GameState) -> io::Result<()>` writes: `MoveTo(0,0)` ANSI escape, top border `+` + 40 `-` + `+`, 25 rows of `\|` + 40 cells + `\|`, bottom border, `Score: N`, each line ending `\r\n`; glyphs: head `●`, body `■`, food `◆`; private `output: W` with **no accessor** | `src/terminal/renderer.rs` |
| `&mut Vec<u8>` implements `std::io::Write` (std blanket `impl<W: Write + ?Sized> Write for &mut W`, and `Vec<u8>` implements `Write`) ⇒ `Renderer::new(&mut buffer)` renders into a capturable buffer **with zero source changes** | std |
| `queue!("MoveTo")` always writes ANSI (`\x1b[1;1H`) for `MoveTo` (default `is_ansi_code_supported` = true) ⇒ only ASCII `[`,`1`,`;`,`1`,`H` bytes added — cannot collide with glyph/byte assertions | crossterm 0.29 |
| `KeyEvent { code, modifiers, kind, state }` — all fields public; `KeyModifiers::NONE`, `KeyEventState::NONE`, `KeyEventKind::{Press, Release, Repeat}` exist in 0.29 | crossterm 0.29 |
| Phase 1A test style: `fn fresh_game() -> GameState` helpers, `#[test]` per behavior, direct `snake::game::*` imports | `tests/*.rs` |

### Geometry conclusions used by the tests (verified step-by-step, encoded below)

- **10 straight ticks** from the initial setup move the head from (10,12) to (20,12) = the fixed food ⇒ one consume: score → 1, length → 4, food respawns randomly (never again deterministic — used only via non-deterministic-safe assertions).
- **A length-4 snake can never self-collide**: the only body cells adjacent to the head are `segments[1]` (= reversal, banned) and `segments[3]` (= the vacating tail, excluded) — verified by construction. Therefore the self-collision test builds a **length-5 hook** through the public `GameStateSetup` fields (decision D23).
- **Down route avoids all food randomness**: turning Down from the initial state never crosses food at (20,12) (it sits on row 12 ahead, x>10 — moving down leaves the row immediately). Score stays 0.

---

## 2. Design Decisions Recorded (binding)

- **D22 — buffer capture pattern (instead of an `into_inner` accessor):** every renderer/`tick` assertion uses
  ```rust
  let mut buffer = Vec::new();
  {
      let mut renderer = Renderer::new(&mut buffer);
      // drive ticks / render
  }
  // buffer is plain Vec<u8> again; assert on it
  ```
  The inner block makes the temporary `&mut Vec<u8>` borrow obviously scoped for a junior implementer; NLL would also allow it without the block but the block is the mandated style.
- **D23 — self-collision via custom deterministic setup:** the flow test constructs its own length-5 hook through `GameStateSetup { snake, food, direction }` (all fields `pub` in Phase 1A) instead of relying on random respawns. Initial `initial_setup()` is still used wherever the geometry permits (start gate, eat/grow, boundary).
- **D24 — assertion style:** test functions are `#[test]` with no return type; `io::Result` is unwrapped with `.expect("…")` (fail message names the call). No `?` plumbing in test bodies.
- **D25 — test-exclusions:** `run_playing_loop`, the event drains, and the lifecycle guard are documented as manual/Docker-phase validation targets only (mirrored into `docs/terminal-ui.md` in 4.4).
- **D26 — file/fn naming:** test files `tests/gameplay_flow.rs` (task 9), `tests/terminal_modules.rs` (task 10). Test names use lowercase snake_case built from the behavior (see §3/§4).
- **D27 — counting helper:** one shared private helper per test file signature `fn count_occurrences(buffer: &[u8], text: &str) -> usize` (2 params, allowed). "Contains X" ⇒ `count_occurrences(...) > 0`. Byte-window comparison is UTF-8-safe (`text.as_bytes()` and the buffer bytes are both UTF-8).
- **D28 — static-trace walkthrough location:** the `//!` module doc of `tests/gameplay_flow.rs` (task 9's "documented walkthrough"); `docs/terminal-ui.md` mirroring is step 4.4 (docs-specialist is the only agent authorized to modify documentation per `markdown-generation-rule`).

---

## 3. Task 9 — `tests/gameplay_flow.rs` (exact spec)

### 3.1 Module doc (the static-trace walkthrough — required verbatim content)

File header `//!` documentation (allowed: high-level explanation, not commented-out code). Reproduce the flow chain and the trace table:

```text
Start (main)          -> GameState::new(initial_setup()), status WaitingToStart    -> covered: test 1
Press any key         -> state.start_playing() (domain edge; the key-wait itself
                         is binary-private and validated manually / Phase 2)       -> covered: test 1
Playing               -> per tick: drain -> change_direction (reversals rejected)
                         -> advance_one_step -> render                             -> covered: test 2 + tests/terminal_modules.rs tick tests
Collision             -> enter_game_over() via the domain predicates                -> covered: tests 3 (boundary), 4 (self)
Game over / Exit      -> GAME OVER + Score: N + wait key + TerminalHandle Drop      -> manual (docs/terminal-ui.md), Phase 2 run
```

State explicitly in the doc: the screens and key-waits cannot be tested headlessly; the tests assert the *domain and loop state transitions* that the wiring triggers in the identical order `main.rs` uses (setup → start gate → start_playing → ticks → collision → GameOver → final score preserved).

### 3.2 Imports (exact)

```rust
use std::io;

use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
```

### 3.3 Private helpers (≤2 params, inside the test file)

- `fn fresh_game() -> GameState` — `GameState::new(initial_setup())` (Phase 1A precedent).
- `fn hooked_snake_game() -> GameState` — decision D23, **exact construction**:
  ```rust
  fn hooked_snake_game() -> GameState {
      let hook_segments = vec![
          Position { x: 10, y: 10 },
          Position { x: 9, y: 10 },
          Position { x: 8, y: 10 },
          Position { x: 7, y: 10 },
          Position { x: 6, y: 10 },
      ];
      GameState::new(GameStateSetup {
          snake: Snake::new(hook_segments),
          food: Food::new(Position { x: 30, y: 20 }),
          direction: Direction::Right,
      })
  }
  ```
  Food (30,20) is never crossed by the scripted path below ⇒ fully deterministic, score stays 0.
- `fn count_occurrences(buffer: &[u8], text: &str) -> usize`:
  ```rust
  fn count_occurrences(buffer: &[u8], text: &str) -> usize {
      buffer.windows(text.len())
          .filter(|byte_window| *byte_window == text.as_bytes())
          .count()
  }
  ```
  (Byte-wise comparison of two `&[u8]` slices compiles via the `PartialEq` impl for slices.)

### 3.4 Test functions (4 tests — exact names, drives, and assertions)

**Test 1 — `flow_starts_only_after_a_key_press`** (models start → press-any-key)

1. `let mut game = fresh_game();` — assert `game.status() == GameStatus::WaitingToStart`.
2. Render buffer scope (D22): `tick(&mut game, &[], &mut renderer).expect("tick succeeds while waiting")` — assert **still** `WaitingToStart` and head unchanged at `Position { x: 10, y: 12 }` (the snake must not move on the start screen — brief §11).
3. `game.start_playing();` — assert `game.status() == GameStatus::Playing`.

**Test 2 — `flow_moves_eats_and_grows_during_ticks`** (Move / Input / Eat / Grow)

1. `let mut game = fresh_game(); game.start_playing();`
2. Drive **10 ticks** with `&[]` directions (movement is autonomous; input mapping is covered in §4): the head walks (10,12) → (20,12).
3. Assertions after the 10th tick:
   - `game.status() == GameStatus::Playing` (eating does not end the game),
   - `game.snake().head() == Position { x: 20, y: 12 }` (the consumed food cell),
   - `game.score() == 1` (Phase 1A scoring rule preserved),
   - `game.snake().length() == 4` (growth by exactly one),
   - `game.food().position() != Position { x: 20, y: 12 }` (respawned; already guaranteed not-on-snake by Phase 1A — do not re-assert domain internals here).
4. During the same buffer session (one buffer, one renderer, render happens inside `tick`): assert `count_occurrences(&buffer, "Score: 1") == 1` (only the 10th frame) and `count_occurrences(&buffer, "Score: 0") == 9` (frames 1–9) — proves the per-tick frames flowed from render through `tick`.

**Test 3 — `flow_boundary_collision_returns_game_over`**

1. `let mut game = fresh_game(); game.start_playing();`
2. Tick 1 with `&[Direction::Down]` — steer off row 12 (avoids the fixed food); assert `game.current_direction() == Direction::Down` and head became `Position { x: 10, y: 13 }`.
3. `let ticks_until_bottom_wall = (HEIGHT - 1) - game.snake().head().y;` (= 11 at head y = 13).
4. Loop `for _ in 0..=ticks_until_bottom_wall` — 12 iterations: 11 land the head on `y = HEIGHT - 1` (inside, still Playing), the 12th attempts `y = 25` ⇒ collision. Capture each call's return `let tick_status = tick(...).expect("tick succeeds");` and keep the **last** one.
5. Assertions after the loop:
   - last returned `tick_status == GameStatus::GameOver` (the collision tick itself reports the transition),
   - `game.status() == GameStatus::GameOver`,
   - `game.snake().head().y == HEIGHT - 1` — the head was **not** moved onto the wall cell after collision (Phase 1A order: collision is checked before the move),
   - `game.score() == 0` (the Down route consumed nothing),
   - buffer contains `"Score: 0"` (final frame was rendered even on the losing tick — matches `run_playing_loop`'s "renders the final frame" contract).

**Test 4 — `flow_self_collision_returns_game_over`** (D23 hook)

1. `let mut game = hooked_snake_game(); game.start_playing();` — assert `game.snake().length() == 5` (documents why the hook is needed).
2. Scripted ticks (verified trace):
   - tick `&[]` → head (11,10);
   - tick `&[Direction::Up]` → head (11,9);
   - tick `&[Direction::Left]` → head (10,9); assert `game.status() == GameStatus::Playing` right before the next tick, and `game.snake().length() == 5` still (nothing eaten — food at (30,20) untouched).
   - tick `&[Direction::Down]` → next head (10,10) hits `segments[3]` (not the tail) ⇒ **GameOver**.
3. Assertions: the fatal tick's returned status == `GameStatus::GameOver`; `game.status() == GameStatus::GameOver`; head unchanged at `Position { x: 10, y: 9 }`; `game.snake().length() == 5`; `game.score() == 0`.
   (This is the *loop-mechanics-equivalent* of pressing Down after a row-12 left-hook — full `tick` path, no reversal shortcut, no randomness.)

### 3.5 Rules compliance for this file

- No `#[cfg]` tricks, no `mod` blocks — a flat `#[test]` list like Phase 1A files.
- Test bodies stay ≤ 50 lines each (loop bodies are one `tick` call + one assignment); nesting ≤ 2 (top `for` loop only); booleans single-section; files expected ≈ 150–190 lines total (the 200-line cap strictly applies to `src/` only, but target it anyway).

---

## 4. Task 10 — `tests/terminal_modules.rs` (exact spec)

Three sections; flat files, no `mod` blocks. Imports (exact):

```rust
use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState};

use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
use snake::terminal::input::map_key_event_to_direction;
```

### 4.1 Arrow-key → Direction mapping + reversal handling (3 tests)

Helpers (private, 1 param each, exact):

```rust
fn arrow_press(key_code: KeyCode) -> KeyEvent {
    KeyEvent { code: key_code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }
}

fn arrow_release(key_code: KeyCode) -> KeyEvent {
    KeyEvent { code: key_code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, state: KeyEventState::NONE }
}
```

1. **`arrow_presses_map_to_the_four_directions`** — loop over
   `[(KeyCode::Up, Direction::Up), (KeyCode::Down, Direction::Down), (KeyCode::Left, Direction::Left), (KeyCode::Right, Direction::Right)]`;
   assert `map_key_event_to_direction(&arrow_press(key_code)) == Some(expected)` for each. (Single for-loop = nesting 1.)
2. **`non_arrow_press_maps_to_none`** — `map_key_event_to_direction(&arrow_press(KeyCode::Char('a')))` is `None`.
3. **`key_release_maps_to_none`** — `map_key_event_to_direction(&arrow_release(KeyCode::Left))` is `None`.
   (Windows parsing never yields `Repeat`; release coverage is sufficient — do not add a Repeat case.)

*Reversal handling expectations move through the loop hook, not the mapping (mapping is domain-free by design):*

4. **`tick_applies_directions_before_one_step_and_rejects_reversal`** — fresh game + `start_playing()`; `tick(&mut state, &[Direction::Up, Direction::Down], &mut renderer).expect("tick succeeds")`:
   - `Up` applied, `Down` rejected (immediate reversal of `Up` — the domain rejects it inside `apply_directions`; the `bool` is discarded exactly as the loop does),
   - exactly **one** step in the surviving direction: head `Position { x: 10, y: 11 }`,
   - `state.current_direction() == Direction::Up`,
   - returned status == `GameStatus::Playing`.

### 4.2 `tick` loop-step semantics (4 tests)

5. **`tick_advances_exactly_one_step_per_call`** — fresh + `start_playing()`; two consecutive `tick(..., &[], ...)` calls; after the first head `Position { x: 11, y: 12 }`, after the second `Position { x: 12, y: 12 }` (one cell per call — the timed waiting belongs to `run_playing_loop` and is not tested).
6. **`tick_returns_game_over_when_the_head_exits_the_boundary`** — fresh + `start_playing()`; identical driving to §3.4 test 3: tick `&[Direction::Down]` once, then `let ticks_until_bottom_wall = (HEIGHT - 1) - game.snake().head().y;` and loop `for _ in 0..=ticks_until_bottom_wall`; final captured return == `GameStatus::GameOver`; head `y == HEIGHT - 1`.
   (Same constants and loop shape as §3.4 test 3 — it is acceptable and intended that both files validate the boundary path: one asserts the *flow*, this asserts the *return contract of `tick`*. Import `HEIGHT` here too.)
7. **`tick_is_a_no_move_before_playing`** — fresh game (`WaitingToStart`, **no** `start_playing()`); `tick(&mut state, &[], &mut renderer).expect("tick succeeds")`:
   - `state.status() == GameStatus::WaitingToStart`,
   - head still `Position { x: 10, y: 12 }`,
   - **do not** assert `current_direction()` here — direction *steering* is not status-gated (only movement is; consistent with Phase 1A's `advance_does_nothing_before_playing`).
8. **`tick_returns_the_new_status_playing`** — fresh + `start_playing()`; one `tick(..., &[], ...)`; assert returned value == `GameStatus::Playing` (positive-control return contract).

### 4.3 Renderer snapshot checks via `&mut Vec<u8>` (3 tests)

All use the D22 pattern; assertions via `count_occurrences`.

9. **`render_writes_the_full_frame_into_the_buffer`** — fresh game rendered directly (`renderer.render(&game).expect("render succeeds")`):
   - `count_occurrences(&buffer, "+") == 4` (two borders × two corners; `MoveTo`'s ANSI bytes contain no `+`),
   - `count_occurrences(&buffer, "Score: 0") == 1`,
   - head glyph: `count_occurrences(&buffer, "●") == 1`, body glyph `"■" == 2` (initial length 3 − head), food glyph `"◆" == 1` (fixed initial food is free of the snake),
   - the `LineTo row` widths are implicitly covered by the corner count; **do not** assert full-line equality strings (keeps the test resilient and short).
10. **`tick_renders_one_consistent_frame_of_glyphs`** — after one `tick` (playing, `&[]`) into a fresh buffer: head glyph count == 1, body glyph count == 2, food glyph count == 1, `"Score: 0"` count == 1. (Snapshot consistency: a full frame can only contain one head.)
11. **`two_renders_reuse_the_frame_without_scrolling`** — render the **same** fresh state twice into the same buffer (two scopes or one renderer used twice):
    - `count_occurrences(&buffer, "+") == 8` (4 corners × 2 frames),
    - `count_occurrences(&buffer, "Score: 0") == 2`,
    - `count_occurrences(&buffer, "●") == 2`.
    (Escapes + rewrites per frame rather than appending a growing board — the no-scroll property observed on the byte level.)

### 4.4 Rules compliance for this file

- Same constraints as §3.5; expected ≈ 190–230 lines (target ≤ 230; the strict `src/` cap does not apply to `tests/` — cited from the max-lines-per-file rule — but keep functions ≤ 50 lines and files compact).
- No tests for `is_key_press` private helpers etc. — public API only.

---

## 5. Static Trace of `src/main.rs` Wiring (plan side — implementer reproduces as the §3.1 module doc)

| main.rs line(s) | Behavior | Headless coverage |
|---|---|---|
| 27 | `GameState::new(initial_setup())` → `WaitingToStart` | flow test 1 |
| 28 | `TerminalHandle::enable(stdout())` | manual (touches the real console) |
| 30–31 | start screen + `wait_for_any_key_press()` | manual |
| 32 | `state.start_playing()` → `Playing` | flow test 1 |
| 34–35 | `Renderer` shares `terminal.output()`; `run_playing_loop` (drain → change_direction → advance_one_step → render → sleep) | `tick` tests + flow tests 2/3/4 (`tick` is the headless stand-in for the loop's inner body; timing and the real drain are excluded) |
| 37–38 | game-over screen + exit wait | manual |
| drop | `TerminalHandle` restores the terminal | manual (Drop guard, real console) |

The junior implementer reproduces this table essence in the `//!` doc of `tests/gameplay_flow.rs` (§3.1) — no `docs/` edits at 4.2.

---

## 6. Git & Commit Sequence (implementer, step 4.2)

All commands single (per the tools rule); no push, no branch creation. Before each commit: `git status` (gitignore-compliance — `tests/` files are not ignored; confirm no `target/` artifacts exist).

1. Write `tests/gameplay_flow.rs` exactly per §3 (no other file touched) → commit:
   `test: add headless gameplay flow integration test`
2. Write `tests/terminal_modules.rs` exactly per §4 (no other file touched) → commit:
   `test: cover terminal input mapping, tick semantics, and renderer output`
3. Verify statically the six Phase 1A test files are untouched:
   `git diff --stat HEAD~2 -- tests/initial_state.rs tests/direction.rs tests/movement_and_growth.rs tests/food_consumption_scoring.rs tests/collision.rs tests/food_placement.rs src/ Cargo.toml` must produce **no output** (no Phase 1A or source file was modified).

### Commit hygiene

- No `Cargo.lock` is expected (nothing compiled); do not add one.
- The TODO file is NOT modified by this group (4.6 handles its `[DONE]` marks).

---

## 7. Static Verification Checklist (no toolchain — perform 100% before/after each commit)

For each authored file, verify:

- [ ] All imports resolve to **existing public items**: `snake::terminal::game_loop::tick`, `snake::terminal::renderer::Renderer`, `snake::terminal::input::map_key_event_to_direction`, `snake::game::setup::{initial_setup, GameStateSetup}` (three `pub` fields), `snake::game::state::{GameState, GameStatus, HEIGHT, WIDTH}` — cross-checked against `src/game/*` and `src/terminal/*` signatures read in §1.
- [ ] `crossterm` symbols: exactly `crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState}` — all exist in crossterm 0.29; `KeyEvent` is constructed with the **struct literal with all four fields** (not the deprecated `KeyEvent::new`).
- [ ] Every `expect("...")` message names the call that failed.
- [ ] Every test function body ≤ 50 lines; helpers ≤ 2 params (or 1/0); all helpers private; no `pub` items in test files.
- [ ] No function has > 2 levels of nesting; no multi-section boolean conditions.
- [ ] No commented-out code; `//!`/`//` comments only for the walkthrough and unavoidable local clarifications (self-documenting-code rule: prefer names).
- [ ] Real newlines in the written files (newline-prevention rule) — the plan's `"Score: "`, `"\r\n"` strings appear only as assertions about bytes the *renderer* emits; nothing writes literal `\n` into files.
- [ ] `git diff` against HEAD shows **only** the two new files (`git status` before each commit confirms no other path touched).
- [ ] `Cargo.toml` untouched; no new dependencies.
- [ ] Line counts: `Get-Content tests/gameplay_flow.rs | Measure-Object -Line` ≤ 200 (target); same for `tests/terminal_modules.rs` (≤ 230 acceptable, target ≤ 200).

---

## 8. Execution Steps for the Implementer (4.2 — very small, ordered)

1. Confirm working tree clean & branch `feat/phase1b-terminal-game` (`git status`, `git branch --show-current`).
2. Write `tests/gameplay_flow.rs` per §3 (module doc first, then helpers, then the four tests in order 1–4). Do not create anything else.
3. Run the §7 checklist against `tests/gameplay_flow.rs`; fix any finding (re-read the plan section).
4. Commit `test: add headless gameplay flow integration test`.
5. Write `tests/terminal_modules.rs` per §4 (helpers first, then tests 1–11 in order). Do not touch other files.
6. Run the §7 checklist against `tests/terminal_modules.rs`.
7. Verify Phase 1A files untouched (three-file git-diff command in §6.3).
8. Commit `test: cover terminal input mapping, tick semantics, and renderer output`.
9. Return a completion summary listing: files created, commits made, checklist result; explicitly state *not done*: no test execution (no toolchain), no `src/` or docs changes (they belong to 4.3 review / 4.4 docs), no TODO `[DONE]` marks (4.6).

## 9. 4.4 Documentation Touch-Points (docs-specialist — content fully specified here)

1. `README.md` — in the "Terminal UI (Phase 1B)" section append one short paragraph + adjust the `tests/` bullet in "Project Structure":
   - New test files: `tests/gameplay_flow.rs` (start gate, move/eat/grow ticks, boundary & self-collision `GameOver`) and `tests/terminal_modules.rs` (arrow-key mapping, `tick` semantics, renderer snapshot via `&mut Vec<u8>`).
   - All fifteen new tests (four flow + eleven terminal-module), like the six Phase 1A ones, are authored and executed in the Phase 2 Docker phase; interactive play (`dist/snake.exe`) stays manual.
2. `docs/terminal-ui.md`:
   - Update **Status**: Group D added fifteen headless tests in two new files; still no execution locally (Phase 2 Docker).
   - New section **"Headless Tests (Group D)"**: map each public hook (`tick`, `map_key_event_to_direction`, `Renderer` over `&mut Vec<u8>`) to its test file; list **explicit exclusions** (lifecycle, event drains, screens, timing) and point to the "How to Validate Manually" section.
3. `.agent/project-structure.md`:
   - Replace the `tests/` paragraph to say **eight** headless integration test files.
   - Add rows: `tests/gameplay_flow.rs — complete flow through tick: start gate, eat/grow, boundary & self collisions` and `tests/terminal_modules.rs — arrow-key mapping, tick semantics, renderer snapshot via a Vec<u8> buffer`.

## 10. Verification Against the Original Tasks

- **Task 9** ("verify the complete flow, preserve Phase 1A rules"): validated by four executable headless tests tracing the exact `main.rs` order + the `//!` walkthrough (§3.1); Phase 1A rules (reversal ban, score +1, growth +1, no wrap, collision-before-move) are asserted at every stage; the un-testable screen/key-wait/exit parts are explicitly routed to manual/Phase 2 evidence. ✔
- **Task 10** ("extend tests only where headless; don't automate full keyboard gameplay; Phase 1A tests keep passing"): fifteen new headless tests (4 + 11) use only pure/`io::Write`-generic hooks; the un-headless areas (lifecycle, event drains, screens, timing) are named and excluded (D25); Phase 1A files are verified untouched by git diff (§6.3). ✔
- **Constraints & Out of Scope**: no new dependencies, no configuration systems, no restart, no Docker, no `src/` changes, no test execution. ✔
