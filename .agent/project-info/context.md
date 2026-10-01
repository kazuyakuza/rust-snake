# Context — Rust Snake

## Current Work Focus

- Phase 1A (TODO `.agent/todos/20261001/20261001-todo-2.md`) is **complete**: all 15 tasks implemented and marked `[DONE]`. The core game domain model, movement/collision/food logic, game-status transitions, and a six-file headless integration test suite are in place on branch `feat/phase1a-core-game-model` (through commit `f3b9a77`).
- Next up: Phase 1B — terminal rendering, arrow-key input, timed game loop, and start/game-over interaction (TODO candidate `.agent/todos/20261001/20261001-todo-3.md`, currently untracked). Docker build environment and `cargo test` execution are prerequisites to wire in before gameplay integration.

---

## Recent Changes

- **Phase 1A (Project Foundation & Core Game Model)** completed on branch `feat/phase1a-core-game-model` (HEAD `f3b9a77`), executed in four groups tracked by TODO `.agent/todos/20261001/20261001-todo-2.md`:
  - Group A (tasks 1–4): Cargo foundation (`Cargo.toml`, `rand` dep) and core domain types + board dimensions.
  - Group B (tasks 5–8): initial state, direction handling (reversal rejection), and movement/growth mechanics.
  - Group C (tasks 9–13): random food placement, consumption/scoring, boundary/self collision, and status transitions.
  - Group D (tasks 14–15): library/binary split, six headless integration tests, and README rewrite.
  - Closing docs: this project-info update reflects the Phase 1A completion state.
- Project brief customized for the Rust Snake game; template leftovers (`CHANGELOG.md`, `TBD`) removed — commit `732419b`.
- Project info initialized this cycle: created `product.md`, `context.md`, `architecture.md`, `tech.md`; removed the `.initialized` marker; linked the files from `AGENTS.md`.
- `README.md` adapted from the template to describe the Rust Snake project.
- Documentation coherence pass: aligned project-info cross-links and AI-agent onboarding notes, refreshed the README table of contents, and updated the `.agent/project-structure.md` map.
- Review cycle applied: simplification plan (9 dedup steps) executed in commit `fd194ee`; adherence report `ADHERENT` (`.kilo/plans/20261001-initialize-project-info-adherence.md`).
- Task completion: both TODO lines marked `[DONE]` (`1f32ebd`); TODO renamed to `20261001-todo-1-DONE.md`; workflow plans and the next-phase TODO (`20261001-todo-2.md`) committed (`134a38e`).
- Branch state: Phase 1A work lives on `feat/phase1a-core-game-model` (HEAD `f3b9a77`); not yet merged to `main`. The earlier `feat/initialize-project-info` branch was already merged to `main` and deleted.

---

## Implementation Status

- **Implemented (Phase 1A Group A — TODO tasks 1–4):** the Cargo foundation (`Cargo.toml`, package `snake`, sole dependency `rand`) and the core game domain types (`src/main.rs`, `src/game.rs`, `src/game/{position,direction,snake,food,state}.rs`), including board dimensions (`WIDTH = 40`, `HEIGHT = 25`) and the canonical playable-bounds predicate centralized in `src/game/state.rs`.
- **Implemented (Phase 1A Group B — TODO tasks 5–8):** the initial game state (`initial_setup()` in `src/game/state.rs` (relocated to `src/game/setup.rs` in Group C), built from named `INITIAL_*` coordinate/direction/food constants — a three-segment head-first snake facing `Right`, score `0` via Group A's `GameState::new`, and a fixed initial food clear of the snake; no `rand` yet), direction handling (`GameState::change_direction`, which rejects an immediate reversal through `Direction::opposite` and returns a `bool`), and the snake movement/growth mechanics (`Direction::offset` grid math, a `std::ops::Add` impl on `Position`, `Snake::advance` with a `should_remove_tail` flag, and `GameState::advance_one_step`). The growth flag is wired to food consumption in Group C below.
- **Implemented (Phase 1A Group C — TODO tasks 9–13):** random food placement (`src/game/food_placement.rs` — the first consumer of the `rand` dependency; it retries on occupied cells and returns `None` on a full board, in which case the food is simply not repositioned rather than inventing a new loss condition); food consumption and scoring wired into `GameState::advance_one_step` (the `Food::occupies` head-equals-food test, `score += SCORE_INCREMENT`, growth via the tail-retention flag, and a respawn computed from the post-move snake); boundary and self collision as pure, terminal-free predicates in `src/game/collision.rs` (`is_outside_board` delegates to the canonical bounds check, `collides_with_body` excludes the tail that vacates its cell); and game-status transitions on the existing `GameStatus` enum (`start_playing`/`enter_game_over` plus the private `is_playing` tick gate). The Group C module split also moved the initial-setup block out of `state.rs` into the new `src/game/setup.rs`.
- **Implemented (Phase 1A Group D — TODO tasks 14–15):** a library/binary split — `src/lib.rs` now owns the `pub mod game;` module tree while `src/main.rs` is a thin `fn main() {}` entry — so the domain is reachable as `snake::game::*` from both the binary and the new `tests/` integration suite; six headless test files (`initial_state`, `direction`, `movement_and_growth`, `food_consumption_scoring`, `collision`, `food_placement`) covering task 14's rules (initial length/score/direction/status, direction acceptance and reversal rejection, movement/growth with the pre-play status gate, food consumption/scoring/respawn, boundary and tail-aware self collision, and food-placement guarantees); and the `README.md` rewrite (task 15), which drops base-project notes and describes only the current project. The tests are **authored only, not yet executed** — there is no local Rust toolchain, so `cargo test` will run them in the Docker phase.
- **Pending (later phases):** the game logic advances headlessly through `advance_one_step`, but `start_playing` still has no in-tree production caller (the terminal start trigger arrives in Phase 1B), and there is no timed game loop, terminal rendering, or keyboard input yet; running `cargo test` and building the executable both wait on the Docker phase.
- **Still absent:** no `Cargo.lock`, `Dockerfile`, `compose.yaml`, or `dist/` output. Nothing has been compiled — the Rust/Cargo toolchain is intentionally not installed here, so all code is hand-written and manually verified (no cargo execution in this workflow).
- Terminal rendering, arrow-key input, the interactive game loop, and Docker builds remain out of scope for Phase 1A (see the TODO's Out-of-Scope list); details in `architecture.md`/`tech.md` for these remain the planned design.

---

## Immediate Next Steps

Order matters — this is the project's live roadmap:

1. Establish the Docker build environment: `Dockerfile` + `compose.yaml` so `docker compose run --rm build` compiles the crate and emits `dist/snake.exe` (brief §3). This also enables `cargo test` execution for the six authored integration tests.
2. Run the test suite in Docker and fix any failures surfaced by the first real compilation.
3. Begin Phase 1B (terminal/gameplay — TODO candidate `20261001-todo-3.md`): terminal rendering, arrow-key input, the timed 120 ms game loop, and start/game-over interaction wired to the existing `GameStatus` transitions (brief §6–§13).
4. Verify the game against the Definition of Done checklist in `brief.md` §18.
5. After each phase, update project info per `instructions.md` ("Project Info Update") — especially `context.md`.

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
