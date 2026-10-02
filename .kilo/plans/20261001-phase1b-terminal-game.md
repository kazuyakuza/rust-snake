# Global Plan — Phase 1B: Terminal Game & Gameplay Flow

- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` (Pattern C, 10 tasks)
- **Branch plan**: `feat/phase1b-terminal-game` (from current `main`, up to date with `origin/main` — no pending merge needed)
- **Version plan**: `Cargo.toml` `0.1.0` → `0.2.0` (minor: new feature phase)

---

## Global Pre-Analysis

### Current state (verified)

- Phase 1A domain model is complete and merged: `src/game/{position,direction,snake,food,setup,state,collision,food_placement}.rs`, library root `src/lib.rs` (`pub mod game;`), thin `src/main.rs` (`fn main() {}`), six headless integration tests in `tests/` (authored, never executed — no local Rust toolchain).
- Key domain API the terminal layer will consume (source of truth, `src/game/state.rs`):
  - `GameState::new(GameStateSetup)`, `.snake()`, `.food()`, `.score()`, `.status()`, `.current_direction()`
  - `GameState::change_direction(Direction) -> bool` (rejects immediate reversal)
  - `GameState::advance_one_step()` (no-op unless `Playing`; resolves move/consume/collide)
  - `GameState::start_playing()` / `.enter_game_over()`; `GameStatus::{WaitingToStart, Playing, GameOver}`
  - `WIDTH = 40`, `HEIGHT = 25` (cell counts, no wrap)
- Untracked files: `20261001-todo-3.md` (this phase) and `20261001-todo-4.md` (Phase 2 doc) — both are TODO docs, safe to commit at step 2.

### Environment constraints (affects verification)

- No local Rust/Cargo toolchain; no Docker in this phase (Docker is Phase 2, out of scope for todo-3).
- Therefore **no compile/test execution is possible in this workflow**. Verification is static (code review, plan adherence). Authored tests will be executed in Phase 2. Manual interactive validation of the game is deferred to the user (after Phase 2 produces `dist/snake.exe`).

### Technical & architecture decisions (global)

1. **New dependency: `crossterm`** (single addition to `Cargo.toml`). Rationale: the TODO requires raw terminal input + lifecycle control + non-blocking polling on Windows. `crossterm` is the minimal, de-facto standard terminal-handling crate — it is **not** a game engine and not an advanced terminal framework (e.g., it is not `ratatui`); it keeps the terminal layer thin and has native Windows console support. Exact stable version pinned at Group A planning (currently `0.28`/`0.29` line — architector confirms at 4.1a/4.1b time).
2. **New `src/terminal/` module tree**, added in the library (`src/lib.rs` → `pub mod terminal;`) so pure parts remain reachable by tests:
   - `src/terminal.rs` — module root; declares submodules.
   - `src/terminal/renderer.rs` — full-frame renderer over the logical grid; generic over `io::Write` so output can be captured in headless tests; redraws via cursor reposition + clear (no scroll).
   - `src/terminal/input.rs` — maps arrow-key events to `game::Direction` (pure, testable mapping function) + non-blocking poll/drain helper.
   - `src/terminal/lifecycle.rs` — raw mode, cursor hide/show, alternate-screen enter/leave; explicit enable/cleanup with restore-on-error where practical.
   - `src/terminal/game_loop.rs` — 120 ms fixed-tick loop: poll input, apply via `change_direction`, `advance_one_step()`, render, wait remainder; start-screen wait (`start_playing` on any key) and game-over wait (loop stops, exit on key).
   - `src/main.rs` — wires: setup → lifecycle enable → start screen → loop → cleanup.
3. **Character set (decided, resolves brief §5 open point)**: head `●` (U+25CF), body `■` (U+25A0), food `◆` (U+25C6), borders ASCII `+ - |`; score line below board: `Score: N`.
4. **Domain isolation**: terminal layer only calls the Phase 1A API above; no duplicated movement/collision/scoring/growth rules.
5. **Timing**: constant 120 ms tick; each tick uses `event::poll(remaining)` so waiting for input never blocks snake movement.
6. **Max-lines rules**: each new file stays under 200 lines (target ≤125 code lines); short functions; single-section boolean conditions.
7. **Docs target (4.4 per group)**: `README.md` (controls/gameplay/build-status), `.agent/project-structure.md` (new terminal files), `.agent/project-info/*` (final phase update at Group D completion / step 6 close-out).

### Front-end marking (for sub-steps 4.1a / 4.5a)

| Group | TODO tasks | Front-end related | Reason |
|---|---|---|---|
| A | 1, 2, 7 | YES | Terminal renderer, input, lifecycle = the UI layer |
| B | 3, 4, 8 | YES | Game loop + UI wiring + rendering refinement |
| C | 5, 6 | YES | Start screen & game-over screen presentation |
| D | 9, 10 | NO | Headless flow validation + tests |

---

## Step Plan (steps 2–6; each bullet = one `task` tool invocation)

- Step 2: Git Feature Branch Setup => implementer
- Step 3: Version Update => implementer
- Group A (Tasks 1+2+7): 4.1a Front-end Spec => frontend-specialist; 4.1b Implementation Plan => architector; 4.2 Implementation => implementer; 4.3 Review & Simplification => code-reviewer + code-simplifier (then 4.3-fix => implementer); 4.4 Documentation => docs-specialist; 4.5a Front-end Verification => frontend-specialist; 4.5b Plan Adherence => architector; 4.6 Task Completion => implementer
- Group B (Tasks 3+4+8): 4.1a → 4.1b → 4.2 → 4.3 (+fix) → 4.4 → 4.5a → 4.5b → 4.6 => same sub-agent sequence
- Group C (Tasks 5+6): 4.1a → 4.1b → 4.2 → 4.3 (+fix) → 4.4 → 4.5a → 4.5b → 4.6 => same sub-agent sequence
- Group D (Tasks 9+10): 4.1b Implementation Plan => architector; 4.2 Implementation => implementer; 4.3 Review & Simplification => code-reviewer + code-simplifier (+fix); 4.4 Documentation => docs-specialist; 4.5b Plan Adherence => architector; 4.6 Task Completion => implementer
- Step 5: TODO File Completion (rename `-DONE`, cleanup, merge to `main`, push `origin` only) => implementer
- Step 6: Finish — summary + next-TODO prompt (Planner)

### Grouping rationale

Tasks are granular slices of one terminal runtime; Phase 1A precedent (15 tasks → 4 groups) applies. Groups: A = terminal primitives, B = running loop + wiring, C = screens/flow, D = validation/tests.
