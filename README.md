# Rust Snake

Rust Snake is a terminal Snake game written in Rust, built by AI agents through the Critical Workflow. The core game model and its deterministic tests are implemented; the Windows build is covered by the Docker build infrastructure — see [Build & Run](#build--run).

**Attention AI Agents:** Before making any changes, you **must** read and adhere to the guidelines outlined in [`AGENTS.md`](AGENTS.md). This file contains critical information about the project's workflow, rules, and architectural standards.

## Table of Contents

- [About this Project](#about-this-project)
- [Game Rules & Controls](#game-rules--controls)
- [Terminal UI (Phase 1B)](#terminal-ui-phase-1b)
- [Build & Run](#build--run)
- [Project Structure](#project-structure)
- [AI Agents](#ai-agents)

## About this Project

Rust Snake is a classic Snake game played in a Windows terminal: the player steers a continuously moving snake with the arrow keys, collects food, and avoids the walls and its own body.

It is a learning and experimentation project for Rust fundamentals (structs, enums, collections, ownership/borrowing, loops, input handling, timers, modules) — not a production-quality game.

The Windows executable is built with Docker — see [Build & Run](#build--run) and the [Windows Build Guide](docs/BUILD.md).

## Game Rules & Controls

- Board: 80 x 80 logical grid with visible boundaries and no wrap-around.
- Terminal size: with the 80 x 80 board the full frame spans 82 columns x 43 rows (two logical rows are packed into each terminal row using half-block glyphs; borders + score line); it fits a terminal window of at least ~84 x 45 character cells.
- Snake starts at length 3 (1 head + 2 body), moving continuously, initial direction right.
- Controls: arrow keys change direction; movement continues between ticks; the snake can never begin a move going the direction opposite to the direction it actually moved in the previous move — no matter how many direction presses are buffered between moves, a quick multi-swap input can never reverse the snake onto its own body (the head can never step onto the cell its neck occupies).
- 90-degree turns still work: one turn per move, or a double-turn drained within a single move toward free cells.
- Speed: fixed 120 ms per move (~8.3 moves/s), constant, no acceleration.
- Food: exactly one on the board; spawns randomly, never on the snake; eating it = score +1, length +1, respawn.
- Score displayed during gameplay, e.g. `Score: 7`.
- Game over on boundary hit or self collision; shows final score; exits on key press; no restart in the initial version.
- Start screen: press any key to start.

## Terminal UI (Phase 1B)

The terminal layer is now wired end to end in [`src/main.rs`](src/main.rs): it builds the initial game state, enables the terminal, shows a **start screen** (`Press any key to start`), waits for any key press, transitions the domain state to `Playing` via `start_playing`, and hands off to the fixed 120 ms playing loop (`run_playing_loop` in [`src/terminal/game_loop.rs`](src/terminal/game_loop.rs)) — each tick drains the pending arrow-key directions, applies them through the domain's `change_direction` (immediate reversals are still rejected at apply time, and `advance_one_step` additionally resolves a buffered burst that would end up opposite the last moved direction before the move begins), advances one step, renders a single frame, then sleeps the remainder of the interval, exiting once the game status leaves `Playing`. On game over, `main` shows a **game-over screen** with the final score (`GAME OVER` / `Score: N` / `Press any key to exit`), waits for a key press, and returns; the terminal handle is then dropped, restoring the real terminal. There is no restart flow. The `main` screen helpers (`show_start_screen`, `show_game_over_screen`, `wait_for_any_key_press`) are private to the binary. `tick` is a headless one-tick function so the loop's state transitions can be validated without a terminal; those tests are executed in the Alpine VM Docker (`cargo test`; see [`docs/testing.md`](docs/testing.md)) and the full suite is green — 71 test functions across 9 files, 0 failed as of 2026-10-06, and the full start→play→game-over→exit flow is validated manually by running the built binary in a real Windows terminal.

- **Renderer** (`src/terminal/renderer.rs`): `Renderer<W: io::Write>` draws a full frame — ASCII borders, color-coded half-block glyphs (head yellow, body green, food red, two logical rows packed per terminal row), and a `Score: N` line — moving the cursor to home each tick so the board redraws in place without scrolling. It reads state only; it holds no game logic.
- **Input** (`src/terminal/input.rs`): `map_key_event_to_direction` translates an arrow-key press into a game `Direction`; `drain_arrow_directions` collects every pending arrow press in chronological order without blocking (the drain `run_playing_loop` applies each tick). Reversal enforcement is **not** done here — it stays in the domain rules under `src/game` (`change_direction` apply-time rejection plus the step-time impossible-reversal resolution in `advance_one_step`), which remain the single source of truth for movement.
- **Lifecycle** (`src/terminal/lifecycle.rs`): `TerminalHandle` enables raw mode + the alternate screen and hides the cursor, exposes the wrapped stream through `output()` so `main` can share it between the start/game-over screens and the renderer, then restores all three on `disable()` or on `Drop` (including error paths), so the terminal is never left in a hidden-cursor or raw-input state.

The renderer and handle are generic over `io::Write`, so frames can be validated headlessly against an in-memory buffer before an interactive terminal is available. Full module map and integration notes: [`docs/terminal-ui.md`](docs/terminal-ui.md).

Two headless integration test files extend the suite for Phase 1B: [`tests/gameplay_flow.rs`](tests/gameplay_flow.rs) covers the start gate, move/eat/grow ticks, and boundary & self-collision `GameOver`, and [`tests/terminal_modules.rs`](tests/terminal_modules.rs) covers arrow-key mapping, `tick` semantics, and renderer snapshots captured through a `&mut Vec<u8>` buffer. All fifteen new tests (four flow + eleven terminal-module), like the six Phase 1A ones, are authored in Phase 1B; [`tests/direction_swap_reversal.rs`](tests/direction_swap_reversal.rs) (10 direction-swap reversal tests) was added in the 2026-10-06 fix workflow; interactive play (`dist/snake.exe`) stays manual.

## Build & Run

- Build the Windows executable with the single documented command (no host
  Rust toolchain needed — the pinned image builds implicitly from
  [`Dockerfile`](Dockerfile) via [`compose.yaml`](compose.yaml); define all
  build parameters there, not on the command line):

      docker compose run --rm build

- Artifact on the host: `dist/snake.exe` — copy this single file to a
  Windows machine and run it there; Windows runtime validation is a
  separate manual step, NOT part of the Docker build.
- Full prerequisites and step-by-step workflow (including `dist/`
  git-ignoring and `Cargo.lock` generation): [`docs/BUILD.md`](docs/BUILD.md).

## Project Structure

A compact Rust project; the game domain is split into small modules under `src/game/`:

- [`Cargo.toml`](Cargo.toml): the Cargo manifest (package `snake`, dependencies `rand` and `crossterm`).
- `src/lib.rs`: library entry; exposes the `game` module for tests and future phases.
- `src/main.rs`: binary entry point; wires initial setup, terminal enable, the start screen, the playing loop, the game-over screen, and exit cleanup.
- [`src/game.rs`](src/game.rs): `game` module root; declares the domain submodules.
- `src/game/position.rs`, `src/game/direction.rs`, `src/game/snake.rs`, `src/game/food.rs`: core value types (grid cell, direction, head-first snake, food).
- `src/game/setup.rs`, `src/game/state.rs`: initial setup + game state with the per-tick move/consume/collide driver.
- `src/game/collision.rs`, `src/game/food_placement.rs`: death predicates and random free-cell food placement.
- `src/terminal.rs`, `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/game_loop.rs`, `src/terminal/lifecycle.rs`: terminal layer primitives (board rendering, arrow-key input, timed playing loop, terminal lifecycle).
- `tests/`: integration tests for the deterministic core logic — nine files with 71 test functions: six Phase 1A files plus `tests/gameplay_flow.rs` and `tests/terminal_modules.rs` from Phase 1B plus `tests/direction_swap_reversal.rs` (10 direction-swap reversal tests); execution still requires the Cargo toolchain inside Docker, and the documented build command performs the release build — see [Build & Run](#build--run) and [`docs/testing.md`](docs/testing.md).
- `.cargo/config.toml`: mingw-w64 linker configuration for the `x86_64-pc-windows-gnu` target.
- [`Dockerfile`](Dockerfile) and [`compose.yaml`](compose.yaml): pinned Docker cross-compilation environment (`rust:1.98.1-slim-bookworm`) and the `build` service producing `dist/snake.exe`.
- `Cargo.lock`: dependency lockfile — generated by the Docker build and then committed.
- `dist/snake.exe`: generated Windows artifact (git-ignored; appears after the first build).

AI agent integration (`.agent/`, `.kilo/`, `.opencode/`) is documented in [`AGENTS.md`](AGENTS.md).

## AI Agents

Before making changes, read [`AGENTS.md`](AGENTS.md) and follow the Critical Workflow (`.kilo/commands/critical-workflow.md`). Persistent project context lives in [`.agent/project-info/`](.agent/project-info/) — `brief.md` is the source of truth for scope.
