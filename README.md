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

- Board: 40 x 25 logical grid with visible boundaries and no wrap-around.
- Snake starts at length 3 (1 head + 2 body), moving continuously, initial direction right.
- Controls: arrow keys change direction; movement continues between ticks; immediate reversal into itself is rejected.
- Speed: fixed 120 ms per move (~8.3 moves/s), constant, no acceleration.
- Food: exactly one on the board; spawns randomly, never on the snake; eating it = score +1, length +1, respawn.
- Score displayed during gameplay, e.g. `Score: 7`.
- Game over on boundary hit or self collision; shows final score; exits on key press; no restart in the initial version.
- Start screen: press any key to start.

## Terminal UI (Phase 1B)

The terminal layer is now wired end to end in [`src/main.rs`](src/main.rs): it builds the initial game state, enables the terminal, shows a **start screen** (`Press any key to start`), waits for any key press, transitions the domain state to `Playing` via `start_playing`, and hands off to the fixed 120 ms playing loop (`run_playing_loop` in [`src/terminal/game_loop.rs`](src/terminal/game_loop.rs)) — each tick drains the pending arrow-key directions, applies them through the domain's `change_direction` (which still rejects immediate reversals), advances one step, renders a single frame, then sleeps the remainder of the interval, exiting once the game status leaves `Playing`. On game over, `main` shows a **game-over screen** with the final score (`GAME OVER` / `Score: N` / `Press any key to exit`), waits for a key press, and returns; the terminal handle is then dropped, restoring the real terminal. There is no restart flow. The `main` screen helpers (`show_start_screen`, `show_game_over_screen`, `wait_for_any_key_press`) are private to the binary. `tick` is a headless one-tick function so the loop's state transitions can be validated without a terminal; those tests remain authored-only in the repository (the Docker build command performs the release build, not a test run), and the full start→play→game-over→exit flow is validated manually by running the built binary in a real Windows terminal.

- **Renderer** (`src/terminal/renderer.rs`): `Renderer<W: io::Write>` draws a full frame — ASCII borders, distinct snake head/body glyphs, food, and a `Score: N` line — moving the cursor to home each tick so the board redraws in place without scrolling. It reads state only; it holds no game logic.
- **Input** (`src/terminal/input.rs`): `map_key_event_to_direction` translates an arrow-key press into a game `Direction`; `drain_arrow_directions` collects every pending arrow press in chronological order without blocking (the drain `run_playing_loop` applies each tick). Immediate-reversal rejection is **not** done here — it stays in the domain rules under `src/game`, which remain the single source of truth for movement.
- **Lifecycle** (`src/terminal/lifecycle.rs`): `TerminalHandle` enables raw mode + the alternate screen and hides the cursor, exposes the wrapped stream through `output()` so `main` can share it between the start/game-over screens and the renderer, then restores all three on `disable()` or on `Drop` (including error paths), so the terminal is never left in a hidden-cursor or raw-input state.

The renderer and handle are generic over `io::Write`, so frames can be validated headlessly against an in-memory buffer before an interactive terminal is available. Full module map and integration notes: [`docs/terminal-ui.md`](docs/terminal-ui.md).

Two headless integration test files extend the suite for Phase 1B: [`tests/gameplay_flow.rs`](tests/gameplay_flow.rs) covers the start gate, move/eat/grow ticks, and boundary & self-collision `GameOver`, and [`tests/terminal_modules.rs`](tests/terminal_modules.rs) covers arrow-key mapping, `tick` semantics, and renderer snapshots captured through a `&mut Vec<u8>` buffer. All fifteen new tests (four flow + eleven terminal-module), like the six Phase 1A ones, are authored in Phase 1B; interactive play (`dist/snake.exe`) stays manual.

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
- `tests/`: integration tests for the deterministic core logic — six Phase 1A files plus `tests/gameplay_flow.rs` and `tests/terminal_modules.rs` from Phase 1B (execution still requires the Cargo toolchain inside Docker; the documented build command performs the release build — see [Build & Run](#build--run)).
- `.cargo/config.toml`: mingw-w64 linker configuration for the `x86_64-pc-windows-gnu` target.
- [`Dockerfile`](Dockerfile) and [`compose.yaml`](compose.yaml): pinned Docker cross-compilation environment (`rust:1.98.1-slim-bookworm`) and the `build` service producing `dist/snake.exe`.
- `Cargo.lock`: dependency lockfile — generated by the Docker build and then committed.
- `dist/snake.exe`: generated Windows artifact (git-ignored; appears after the first build).

AI agent integration (`.agent/`, `.kilo/`, `.opencode/`) is documented in [`AGENTS.md`](AGENTS.md).

## AI Agents

Before making changes, read [`AGENTS.md`](AGENTS.md) and follow the Critical Workflow (`.kilo/commands/critical-workflow.md`). Persistent project context lives in [`.agent/project-info/`](.agent/project-info/) — `brief.md` is the source of truth for scope.
