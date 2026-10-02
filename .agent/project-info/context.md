# Context — Rust Snake

## Current Work Focus

- Phase 2 (Windows Build Infrastructure — TODO `.agent/todos/20261001/20261001-todo-4.md`) is **implemented**: all 10 tasks `[DONE]` after Group C. `Dockerfile` (pinned `rust:1.98.1-slim-bookworm`, mingw-w64, `x86_64-pc-windows-gnu`), `compose.yaml` (`build` service → `dist/snake.exe`), `.cargo/config.toml`, and the documentation (`docs/BUILD.md`, rewritten README Build & Run, refreshed structure map and project info) have landed on branch `feat/phase2-docker-windows-build`; merge/push happens in workflow step 5. The build itself is executed by the user outside this workflow.
- Next up: user runs `docker compose run --rm build` (first real compilation + `Cargo.lock` generation), then manual Windows validation of `dist/snake.exe` per brief §18.

---

## Recent Changes

- **Phase 2 (Windows Build Infrastructure)** implemented on branch `feat/phase2-docker-windows-build`, executed in three groups tracked by TODO `.agent/todos/20261001/20261001-todo-4.md`: Group A (tasks 1–4) build infra — version bump to 0.3.0 (`4ac280a`), mingw-w64 linker config (`30d3f95`), Dockerfile (`0a04d7c`), compose build service (`1587f72`); Group B (tasks 5–7) artifact convention/single command in the compose header (`f79cafe`), no-helper-script decision recorded; Group C (tasks 8–10) documentation — `docs/BUILD.md`, README rewrite, structure map + project info updates, plus `[DONE]` marks (`93a738e`, `8de56a5`, and the Group C commits). Plan: [`.kilo/plans/20261002-phase2-groupC-documentation.md`](../../.kilo/plans/20261002-phase2-groupC-documentation.md); adherence report: [`.kilo/plans/20261002-phase2-groupC-adherence.md`](../../.kilo/plans/20261002-phase2-groupC-adherence.md).
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
- **Implemented (Phase 2):** the Docker-based Windows build workflow — pinned image, compose build service, artifact convention, documented command and docs — is complete as authored work.
- **Still absent:** no `Cargo.lock` or `dist/` output yet (both are produced by the user's first `docker compose run --rm build`); `dist/` and `target/` are git-ignored; the 59 authored test functions have never been executed — no cargo toolchain in this context.
- The domain logic in `src/game/` remains the **untouched source of truth** for game rules; the Phase 1B terminal layer only reads `GameState`, drives it through the existing `advance_one_step`/`change_direction`/`start_playing` API, and renders it — it adds no game rules.

---

## Immediate Next Steps

Order matters — this is the project's live roadmap (Phase 2, TODO `.agent/todos/20261001/20261001-todo-4.md`):

1. Run `docker compose run --rm build` to perform the first real compilation, generate `Cargo.lock`, and produce `dist/snake.exe` (see `docs/BUILD.md`).
2. Commit the generated `Cargo.lock` and verify `dist/snake.exe` exists; report any compile errors back into a fix TODO.
3. Manually validate gameplay on Windows by running `dist/snake.exe` against the Definition of Done checklist in `brief.md` §18 (start screen, 3-block initial snake, continuous movement, arrow controls, no immediate reversal, food/score/growth, boundary and self game-over, score display, key-press exit).
4. Optionally execute the authored test suite (`cargo test` in a Rust/Docker environment) in a later phase — not part of the current build command.

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
