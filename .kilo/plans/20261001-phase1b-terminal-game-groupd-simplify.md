# Simplification Plan — Phase 1B Group D Step 4.3 (code-simplifier review)

- **Scope**: ONLY the two Group D test files authored in step 4.2 — `tests/gameplay_flow.rs` (156 lines) and `tests/terminal_modules.rs` (193 lines). Nothing else.
- **Contract**: `.kilo/plans/20261001-phase1b-terminal-game-groupd.md` — its behavioral assertions, test names, helper signatures, and the D22 buffer-block style are FROZEN. Nothing in this plan touches them.
- **Constraint**: all edits below are behavior-neutral (test pass/fail outcomes unchanged). No `src/` changes, no Phase 1A test changes, no plan edits, no TODO file changes, no commits.
- **Implementer note**: junior developer, 50% restriction — execute each step exactly as written; no judgment calls needed or allowed. Steps are atomic.
- **Verification mode**: static only (no local Rust toolchain, no Docker, no cargo). See §4.

## Review verdict

Simplification required, but SMALL. No new TODO file needed. The scope is:

1. **Required (S1)**: remove 6 genuinely unused imports (3 in `gameplay_flow.rs`? — no: 1 there, 5 there are all used; 5 unused in `terminal_modules.rs` minus 1 shared). Exact counts: 1 unused in `gameplay_flow.rs`, 5 unused in `terminal_modules.rs`. Rust emits `unused_imports` warnings for these once the Phase 2 toolchain compiles the test targets; removal is pure cleanup.
2. **Optional (S2)**: extract a 0-param `started_game()` helper in each file to absorb the `fresh_game(); start_playing();` duplication (2 occurrences in `gameplay_flow.rs`, 5 in `terminal_modules.rs`). Low value, adds a helper beyond the plan's frozen helper list — recommend SKIP unless the caller wants maximum deduplication. Detailed below; default = not applied.

Reviewed and REJECTED as non-simplifications (rationale):
- D22 `let mut buffer = Vec::new(); { let mut renderer = Renderer::new(&mut buffer); ... }` blocks: repetition is plan-mandated (D22 "block is the mandated style"), each block drives different calls, and any helper would need a closure/state parameter combination exceeding the 2-param rule or hiding borrow-scope subtleties from the junior implementer. Keep verbatim.
- `arrow_press`/`arrow_release` one-line struct literals: formatting concern (rustfmt in Phase 2 CI), not readability/behavior. Keep.
- Assertion tightening on `flow_boundary_collision_returns_game_over`'s `assert!(count_occurrences(&buffer, "Score: 0") > 0)` → could be `== 13` (13 frames all show score 0), but the plan freezes "buffer contains Score: 0" and exact frame counts couple the test to renderer frame emission. Keep.
- Test/helper naming: all names are descriptive, plan-frozen (D26). Keep.

---

## S1 — Remove unused imports (REQUIRED)

Usage was cross-checked against every identifier in each file body (imports excluded).

### S1a `tests/gameplay_flow.rs` — 1 removal

`std::io` is never referenced in the body (`.expect("...")` handles the `io::Result` without naming the path). `Food`, `Snake`, `GameStateSetup` ARE used by `hooked_snake_game` (lines 43-56) — keep them.

**OLD** (lines 28-37):
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

**NEW**:
```rust
use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
```

(The blank line that separated `use std::io;` from the `snake::` group is removed with it; the file now starts its import block directly with `use snake::game::direction::Direction;`. Blank lines between the module doc, helpers, and tests stay as-is.)

### S1b `tests/terminal_modules.rs` — 5 removals (3 whole lines, 2 shrunk lines)

Body scan: this file only ever builds games via `fresh_game()` = `initial_setup()`; it never names `io`, `Food`, `Snake`, or `GameStateSetup`.

**OLD** (lines 1-13):
```rust
use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState};

use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
use snake::terminal::input::map_key_event_to_direction;
```

**NEW**:
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

Removal detail (map of the 5):
- `use std::io;` line deleted (plus its trailing blank line — the file now starts with the `crossterm` import).
- `use snake::game::food::Food;` line deleted.
- `use snake::game::snake::Snake;` line deleted.
- `use snake::game::setup::{GameStateSetup, initial_setup};` → `use snake::game::setup::initial_setup;` (single-item path, braces dropped).

### Post-S1 line-count effect

- `tests/gameplay_flow.rs`: 156 → 154 lines.
- `tests/terminal_modules.rs`: 193 → 189 lines.

---

## S2 — Optional `started_game()` helper (default: SKIP)

Only if the caller explicitly requests maximum deduplication. Two identical edits, one per file:

1. In each file, immediately after the existing `fn fresh_game()` helper, insert:
   ```rust
   fn started_game() -> GameState {
       let mut game = fresh_game();
       game.start_playing();
       game
   }
   ```
   (0 params; private; same file; mirrors the Phase 1A `fresh_game()` precedent.)
2. Replace every occurrence of the two-line pair
   ```rust
   let mut game = fresh_game();
   game.start_playing();
   ```
   (variable named `state` in `terminal_modules.rs`: `let mut state = fresh_game();` + `state.start_playing();`)
   with the single line `let mut game = started_game();` / `let mut state = started_game();`.

Occurrences: 2 in `gameplay_flow.rs` (tests 2, 3), 5 in `terminal_modules.rs` (tests `tick_applies...`, `tick_advances...`, `tick_returns_game_over...`, `tick_returns_the_new_status_playing`, `tick_renders_one_consistent_frame_of_glyphs`). `flow_self_collision_returns_game_over` keeps `hooked_snake_game()` + `start_playing()` — do NOT touch it (different constructor).

Reasons the default is SKIP: adds a helper outside the plan's frozen helper lists (§3.3, §4.1); saves 5 total lines across the two files; the two-line setup is already self-documenting. Low risk either way — behavior-neutral.

---

## 3. Execution steps for the implementer (atomic, ordered)

1. Apply S1a to `tests/gameplay_flow.rs` (single contiguous replacement above the first helper).
2. Apply S1b to `tests/terminal_modules.rs` (single contiguous replacement above `fn fresh_game`).
3. Do NOT apply S2 unless the caller's instruction to this sub-agent explicitly said to include it (this plan's default = skip).
4. Do not touch anything else; do not commit (commits belong to the caller's workflow, not this step).

## 4. Static verification checklist (no compiler available — perform 100%)

- [ ] `tests/gameplay_flow.rs` no longer contains the token `use std::io;`; the string `io::` appears nowhere in the file body (grep both).
- [ ] `tests/terminal_modules.rs` contains no `use std::io;`, no `Food`, no `Snake`, no `GameStateSetup` anywhere (identifier-level grep: `Food`, `Snake`, `GameStateSetup` → zero matches in that file); `initial_setup` appears exactly twice (import + `fresh_game` body).
- [ ] Every remaining import in both files is referenced by the body: `Direction`, `Position`, `GameStatus`, `GameState`, `HEIGHT` (gameplay_flow only — terminal_modules also uses HEIGHT), `tick`, `Renderer`, and in `terminal_modules.rs` also the five `crossterm::event` items and `map_key_event_to_direction`. Verify by locating ≥1 body usage per import.
- [ ] Zero other changes: `git diff` shows only import-block regions in the two files (no hunks inside any `#[test]` body, helper body, or module doc); `src/`, the six Phase 1A test files, `Cargo.toml`, docs, and TODO files untouched.
- [ ] Test semantics untouched: all 15 test functions unchanged in name, drive sequence, and assertions; line counts 154 / 189 (if S2 applied: gameplay_flow −2 lines, terminal_modules −5 lines, each +5-line helper).
- [ ] Real newlines only; no literal `\n` sequences introduced (newline-prevention rule).

## 5. Handoff note to the caller

- Behavioral contract of Group D is fully preserved; these are warning-elimination edits (rustc `unused_imports` would fire on the current files during the Phase 2 Docker compile).
- If the two 4.2 commits are already made, S1 lands as a follow-up `test:`/`chore:` commit at the caller's discretion (out of this step).
- Ambiguity flagged, unresolved by assumption (per task instructions): whether S2 is desired — default SKIP; caller decides.
