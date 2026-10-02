# Group D Code Review — Fix Plan

**Review scope:** `tests/gameplay_flow.rs` (Task 9), `tests/terminal_modules.rs` (Task 10), and the pending documentation edits in `README.md`, `docs/terminal-ui.md`, and `.agent/project-structure.md`.

**Verdict:** The implementation matches the contract (`.kilo/plans/20261001-phase1b-terminal-game-groupd.md`) and the domain/terminal APIs. All 4 + 11 tests are present with the exact names, assertions, and deterministic geometry called out in the plan. Phase 1A files remain untouched. Documentation touch-points accurately reflect the new tests and the unchanged manual/Phase 2 validation areas.

The only required fixes are **unused-import hygiene**. The plan listed them verbatim, but they will produce compiler warnings as soon as Phase 2 runs `cargo test`, and they add no value. Removing them is a minor local change that does not alter behavior or public contracts.

---

## Required Fixes

### `tests/gameplay_flow.rs`

- **Remove** the unused `std::io` import.

```rust
// Delete line 28:
use std::io;
```

All other imports are actively used (`Direction`, `Position`, `Food`, `Snake`, `GameStateSetup`, `initial_setup`, `GameStatus`, `GameState`, `HEIGHT`, `tick`, `Renderer`).

### `tests/terminal_modules.rs`

- **Remove** the unused `std::io` import.
- **Remove** the unused domain imports `Food`, `GameStateSetup`, and `Snake`.
- Keep `initial_setup` (used by `fresh_game`).

Replace lines 1 and 6–9:

```rust
use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState};

use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState, HEIGHT};
```

with:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState};

use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
use snake::terminal::input::map_key_event_to_direction;
```

(The `tick`, `Renderer`, and `map_key_event_to_direction` imports remain unchanged.)

---

## Why These Are the Only Fixes

- **Assertions are correct** against the actual domain/terminal APIs:
  - `tick` applies directions, advances exactly one step, renders, and returns `state.status()`.
  - `Renderer` emits the expected border, glyph, and score bytes.
  - `map_key_event_to_direction` returns `Some` only for arrow-key presses.
- **Geometry is deterministic:** the eat/grow path (10 ticks on row 12), the down-then-straight boundary path, and the length-5 hook self-collision path all check out against `state.rs`, `collision.rs`, and `setup.rs`.
- **No flaky randomness assumptions:** food respawn is only asserted as "not the old cell"; the boundary route never crosses food.
- **Rules compliance:**
  - File lengths: ~156 and ~193 lines (within the plan's targets).
  - All test/helper functions are well under 50 lines.
  - No function exceeds 2 parameters except the plan-sanctioned `tick` and `count_occurrences`.
  - No multi-section boolean conditions.
  - No commented-out code.
- **No scope overstep:** only the two new test files were added; `src/`, `Cargo.toml`, and the six Phase 1A test files are untouched.
- **Documentation pending content is accurate** and matches plan §9.

---

## Verification After Applying Fixes

1. Confirm `cargo test --no-run` (when available in Phase 2) compiles without unused-import warnings.
2. Re-check that `git diff --stat HEAD~2 -- src/ tests/ Cargo.toml` still shows only the two new test files.
3. Re-count file lines: `tests/gameplay_flow.rs` and `tests/terminal_modules.rs` should remain under the plan targets.
