# Context — Rust Snake

## Current Work Focus

- Phase 1B (Terminal Gameplay & Interaction — TODO `.agent/todos/20261001/20261001-todo-3-DONE.md`) is **complete**: all 10 tasks implemented and marked `[DONE]`. The terminal layer — full-frame renderer, arrow-key input, the fixed 120 ms game loop, and the start/game-over screens — is now wired into `src/main.rs`, so the executable drives the whole flow (start prompt → playing loop → game-over → exit). Work landed on branch `feat/phase1b-terminal-game`, merged to `main` at `3d2d995` and pushed to `origin`; the feature branch was deleted after the merge. Version bumped to `0.2.0`; `crossterm 0.29` added as the terminal-handling dependency alongside `rand`.
- Next up: Phase 2 (TODO `.agent/todos/20261001/20261001-todo-4.md`) — the Docker build environment (`Dockerfile` + `compose.yaml` → `dist/snake.exe`), the first real compilation, and execution of the authored test suite. **No Rust/Cargo toolchain has run any code yet** — every source and test file is hand-written and manually verified only.

---

## Recent Changes

- **Phase 1B (Terminal Gameplay & Interaction)** completed on branch `feat/phase1b-terminal-game` (merged to `main` at `3d2d995`), executed in four groups tracked by TODO `.agent/todos/20261001/20261001-todo-3-DONE.md`:
  - Group A (tasks 1–4): terminal primitives — the full-frame board renderer (`src/terminal/renderer.rs`), arrow-key input mapping plus non-blocking event drain (`src/terminal/input.rs`), and the raw-mode / alternate-screen / cursor lifecycle guard (`src/terminal/lifecycle.rs`). Supporting commits: version bump `e3c239b` (→ `0.2.0`), `crossterm` dependency + `target/` gitignore `2c4f528`, library-root exposure `565f331`.
  - Group B: the fixed 120 ms playing loop and a headless `tick` driver (`src/terminal/game_loop.rs`), wired into the loop and refined (`c17f029` fixed-tick loop, `269950a` chronological arrow drain, `fbf70ad` module declaration).
  - Group C: start and game-over screens and the `main.rs` wiring of the complete flow (`3d72837` main wiring, `fc18c11` `TerminalHandle::output()` accessor).
  - Group D: 15 new headless tests (`tests/gameplay_flow.rs` 4 functions + `tests/terminal_modules.rs` 11 functions) and supporting docs (`03fa32c`, `7f6ee03`, `a58a714`).
  - Review / adherence outcomes: all four groups assessed `ADHERENT`; simplification was applied during Groups A and D (duplicate border-writer merge in `4737e1c`, unused-import cleanup in `a58a714`).
  - One cancellation incident: a Group C docs task (plan item 4.4) was interrupted mid-run, then re-executed cleanly afterward — no changes were lost and no duplicate edits remain.
- **Phase 1A (Project Foundation & Core Game Model)** completed on branch `feat/phase1a-core-game-model` (HEAD `f3b9a77`), executed in four groups tracked by TODO `.agent/todos/20261001/20261001-todo-2.md`:
  - Group A (tasks 1–4): Cargo foundation (`Cargo.toml`, `rand` dep) and core domain types + board dimensions.
  - Group B (tasks 5–8): initial state, direction handling (reversal rejection), and movement/growth mechanics.
  - Group C (tasks 9–13): random food placement, consumption/scoring, boundary/self collision, and status transitions.
  - Group D (tasks 14–15): library/binary split, six headless test files, and README rewrite.
- Project brief customized for the Rust Snake game; template leftovers (`CHANGELOG.md`, `TBD`) removed — commit `732419b`.
- Project info initialized in the Phase 1A cycle: created `product.md`, `context.md`, `architecture.md`, `tech.md`; removed the `.initialized` marker; linked the files from `AGENTS.md`.
- `README.md` adapted from the template to describe the Rust Snake project, then extended across Phase 1B to document the terminal layer, game loop, and start/game-over flow.
- Documentation coherence pass: aligned project-info cross-links and AI-agent onboarding notes, refreshed the README table of contents, and updated the `.agent/project-structure.md` map.
- Review cycle applied in the initial-project-info phase: simplification plan (9 dedup steps) executed in commit `fd194ee`; adherence report `ADHERENT` (`.kilo/plans/20261001-initialize-project-info-adherence.md`).
- Branch state: both phases are now merged to `main`. Phase 1A (`f3b9a77`) and Phase 1B (`3d2d995`) sit on `main`, pushed to `origin`; the feature branches `feat/phase1a-core-game-model`, `feat/phase1b-terminal-game`, and `feat/initialize-project-info` were all merged and deleted. `main` HEAD is `3d2d995`.

---

## Implementation Status

- **Implemented (Phase 1A Group A — TODO tasks 1–4):** the Cargo foundation (`Cargo.toml`, package `snake`, `rand` dep, now version `0.2.0`) and the core game domain types (`src/game.rs`, `src/game/{position,direction,snake,food,state}.rs`), including board dimensions (`WIDTH = 40`, `HEIGHT = 25`) and the canonical playable-bounds predicate centralized in `src/game/state.rs`.
- **Implemented (Phase 1A Group B — TODO tasks 5–8):** the initial game state (`initial_setup()`, now `src/game/setup.rs`, built from named `INITIAL_*` coordinate/direction/food constants — a three-segment head-first snake facing `Right`, score `0`), direction handling (`GameState::change_direction`, which rejects an immediate reversal through `Direction::opposite` and returns a `bool`), and the snake movement/growth mechanics (`Direction::offset` grid math, a `std::ops::Add` impl on `Position`, `Snake::advance` with a `should_remove_tail` flag, and `GameState::advance_one_step`).
- **Implemented (Phase 1A Group C — TODO tasks 9–13):** random food placement (`src/game/food_placement.rs` — retries on occupied cells, returns `None` on a full board); food consumption and scoring wired into `GameState::advance_one_step` (`Food::occupies` test, `score += SCORE_INCREMENT`, growth via the tail-retention flag, respawn from the post-move snake); boundary and self collision as pure, terminal-free predicates in `src/game/collision.rs` (`is_outside_board`, tail-aware `collides_with_body`); and game-status transitions on the `GameStatus` enum (`start_playing`/`enter_game_over` plus the private `is_playing` tick gate).
- **Implemented (Phase 1A Group D — TODO tasks 14–15):** a library/binary split — `src/lib.rs` owns the `pub mod game;` (and now `pub mod terminal;`) module trees so the domain is reachable as `snake::game::*`/`snake::terminal::*` from the binary and the `tests/` suite; six Phase 1A headless test files (`initial_state`, `direction`, `movement_and_growth`, `food_consumption_scoring`, `collision`, `food_placement`); and the `README.md` rewrite.
- **Implemented (Phase 1B Group A — terminal primitives):** `src/terminal/` module tree. `renderer.rs` — `Renderer<W: Write>` draws a complete frame from an immutable `GameState` at the home position: ASCII border rows (`+` corners, `-` horizontal, `|` vertical), per-cell glyphs (head `●` U+25CF, body `■` U+25A0, food `◆` U+25C6, empty space), and a `Score: N` line, all queued via `crossterm` and flushed. `input.rs` — pure `map_key_event_to_direction` plus the non-blocking `drain_arrow_directions` / `drain_arrow_event` (zero-duration poll; reversal rejection stays in `GameState`). `lifecycle.rs` — `TerminalHandle<W>` RAII guard enabling raw mode + alternate screen + hidden cursor and restoring them on `disable()` and on `Drop`.
- **Implemented (Phase 1B Group B — game loop):** `game_loop.rs` runs a fixed `TICK_DURATION = 120 ms` loop while `state.status() == Playing` — each tick drains buffered arrow directions chronologically, applies them via `change_direction`, calls `advance_one_step`, renders once, then sleeps the remainder of the tick; a headless `tick` helper performs the same step without sleeping or touching the real terminal (the test seam). The final losing frame is rendered before the loop returns.
- **Implemented (Phase 1B Group C — screens + wiring):** `main.rs` is the full-flow driver — build state via `initial_setup()`, enable `TerminalHandle` over `stdout()`, show the start screen (`Press any key to start`), block on any key press, `start_playing()`, run `run_playing_loop`, show the game-over screen (`GAME OVER` / blank / `Score: N` / blank / `Press any key to exit`), block on any key press, then exit (terminal restored by `TerminalHandle`'s `Drop`). The `TerminalHandle::output()` accessor lets the screens and the renderer share one owned stdout.
- **Implemented (Phase 1B Group D — tests):** 15 new headless test functions — `tests/gameplay_flow.rs` (4 — the complete start → play → game-over → exit flow driven over an in-memory buffer) and `tests/terminal_modules.rs` (11 — input mapping, drain semantics, `tick` semantics, and renderer output). Like all Phase 1A/1B tests they are **authored only, never executed** — no compilation has run in this workflow.
- **Pending (Phase 2):** first real `cargo build` / `cargo test` and Docker packaging. Phase 1A's open gap is now closed — `start_playing` has a production caller (the key-press trigger in `main.rs`).
- **Still absent:** no `Cargo.lock`, `Dockerfile`, `compose.yaml`, or `dist/` output. Nothing has been compiled — the Rust/Cargo toolchain is intentionally not installed here, so all code is hand-written and manually verified (no cargo execution in this workflow).
- The domain logic in `src/game/` remains the **untouched source of truth** for game rules; the Phase 1B terminal layer only reads `GameState`, drives it through the existing `advance_one_step`/`change_direction`/`start_playing` API, and renders it — it adds no game rules.

---

## Immediate Next Steps

Order matters — this is the project's live roadmap (Phase 2, TODO `.agent/todos/20261001/20261001-todo-4.md`):

1. Establish the Docker build environment: `Dockerfile` + `compose.yaml` so `docker compose run --rm build` compiles the crate and emits `dist/snake.exe` (brief §3); this also generates the first `Cargo.lock`.
2. Run the full authored test suite in Docker — Phase 1A's six test files plus Phase 1B's two new files (`tests/gameplay_flow.rs` and `tests/terminal_modules.rs`), i.e. 59 `#[test]` functions across 8 integration files — and fix any failures surfaced by the first real compilation.
3. Manually validate gameplay on Windows by running `dist/snake.exe` against the Definition of Done checklist in `brief.md` §18 (start screen, 3-block initial snake, continuous movement, arrow controls, no immediate reversal, food/score/growth, boundary and self game-over, score display, key-press exit).
4. After the phase, update project info per `instructions.md` ("Project Info Update") — especially `context.md`.

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
