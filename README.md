# Rust Snake

Rust Snake is a terminal Snake game written in Rust, built by AI agents through the Critical Workflow. The core game model and its deterministic tests are implemented; a Docker-based Windows build that produces `dist/snake.exe` is planned for the next phase.

**Attention AI Agents:** Before making any changes, you **must** read and adhere to the guidelines outlined in [`AGENTS.md`](AGENTS.md). This file contains critical information about the project's workflow, rules, and architectural standards.

## Table of Contents

- [About this Project](#about-this-project)
- [Game Rules & Controls](#game-rules--controls)
- [Build & Run](#build--run)
- [Project Structure](#project-structure)
- [AI Agents](#ai-agents)

## About this Project

Rust Snake is a classic Snake game played in a Windows terminal: the player steers a continuously moving snake with the arrow keys, collects food, and avoids the walls and its own body.

It is a learning and experimentation project for Rust fundamentals (structs, enums, collections, ownership/borrowing, loops, input handling, timers, modules) — not a production-quality game.

A Docker-based Windows build that produces `dist/snake.exe` is planned for a later phase (see [Build & Run](#build--run)).

## Game Rules & Controls

- Board: 40 x 25 logical grid with visible boundaries and no wrap-around.
- Snake starts at length 3 (1 head + 2 body), moving continuously, initial direction right.
- Controls: arrow keys change direction; movement continues between ticks; immediate reversal into itself is rejected.
- Speed: fixed 120 ms per move (~8.3 moves/s), constant, no acceleration.
- Food: exactly one on the board; spawns randomly, never on the snake; eating it = score +1, length +1, respawn.
- Score displayed during gameplay, e.g. `Score: 7`.
- Game over on boundary hit or self collision; shows final score; exits on key press; no restart in the initial version.
- Start screen: press any key to start.

## Build & Run

- Build (expected workflow): `docker compose run --rm build`.
- Output: Windows executable `dist/snake.exe` in a host-mounted output directory.
- Run: execute `dist/snake.exe` directly from Windows — the container is only the compile environment.
- Status: the Cargo project, the core game model, and its core logic tests are implemented; the Docker build environment is planned for the next phase (`Dockerfile` + `compose.yaml` do not exist yet).
- `dist/` is git-ignored (`.gitignore`), so binaries never get committed.

## Project Structure

A compact Rust project; the game domain is split into small modules under `src/game/`:

- [`Cargo.toml`](Cargo.toml): the Cargo manifest (package `snake`, dependency `rand`).
- `src/lib.rs`: library entry; exposes the `game` module for tests and future phases.
- `src/main.rs`: binary entry point (the interactive game loop lives in a later phase).
- [`src/game.rs`](src/game.rs): `game` module root; declares the domain submodules.
- `src/game/position.rs`, `src/game/direction.rs`, `src/game/snake.rs`, `src/game/food.rs`: core value types (grid cell, direction, head-first snake, food).
- `src/game/setup.rs`, `src/game/state.rs`: initial setup + game state with the per-tick move/consume/collide driver.
- `src/game/collision.rs`, `src/game/food_placement.rs`: death predicates and random free-cell food placement.
- `src/terminal.rs`, `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`: terminal layer primitives (board rendering, arrow-key input, terminal lifecycle) — interactive wiring arrives in the next group.
- `tests/`: integration tests for the deterministic core logic (`cargo test` runs them; execution arrives with the Docker build phase).
- Planned later: `Dockerfile`, `compose.yaml`, `Cargo.lock`, `dist/snake.exe`.

AI agent integration (`.agent/`, `.kilo/`, `.opencode/`) is documented in [`AGENTS.md`](AGENTS.md).

## AI Agents

Before making changes, read [`AGENTS.md`](AGENTS.md) and follow the Critical Workflow (`.kilo/commands/critical-workflow.md`). Persistent project context lives in [`.agent/project-info/`](.agent/project-info/) — `brief.md` is the source of truth for scope.
