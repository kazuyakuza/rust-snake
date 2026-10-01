# Context — Rust Snake

## Current Work Focus

- TODO `.agent/todos/20261001/20261001-todo-1-DONE.md` (initialize project info + adapt README) is complete: work was executed on `feat/initialize-project-info`, merged to `main`, and pushed to `origin`.
- Next up: TODO `.agent/todos/20261001/20261001-todo-2.md` — Phase 1A: Project Foundation & Core Game Model (Rust project structure, game types, core logic, tests; no terminal UI/Docker yet).

---

## Recent Changes

- Project brief customized for the Rust Snake game; template leftovers (`CHANGELOG.md`, `TBD`) removed — commit `732419b`.
- Project info initialized this cycle: created `product.md`, `context.md`, `architecture.md`, `tech.md`; removed the `.initialized` marker; linked the files from `AGENTS.md`.
- `README.md` adapted from the template to describe the Rust Snake project.
- Documentation coherence pass: aligned project-info cross-links and AI-agent onboarding notes, refreshed the README table of contents, and updated the `.agent/project-structure.md` map.
- Review cycle applied: simplification plan (9 dedup steps) executed in commit `fd194ee`; adherence report `ADHERENT` (`.kilo/plans/20261001-initialize-project-info-adherence.md`).
- Task completion: both TODO lines marked `[DONE]` (`1f32ebd`); TODO renamed to `20261001-todo-1-DONE.md`; workflow plans and the next-phase TODO (`20261001-todo-2.md`) committed (`134a38e`).
- Branch state: `feat/initialize-project-info` merged fast-forward to `main` and deleted; `main` pushed to `origin` (HEAD `134a38e`, tree clean).

---

## Implementation Status

- **Implemented (Phase 1A Group A — TODO tasks 1–4):** the Cargo foundation (`Cargo.toml`, package `snake`, sole dependency `rand`) and the core game domain types (`src/main.rs`, `src/game.rs`, `src/game/{position,direction,snake,food,state}.rs`), including board dimensions (`WIDTH = 40`, `HEIGHT = 25`) and the canonical playable-bounds predicate centralized in `src/game/state.rs`.
- **Implemented (Phase 1A Group B — TODO tasks 5–8):** the initial game state (`initial_setup()` in `src/game/state.rs`, built from named `INITIAL_*` coordinate/direction/food constants — a three-segment head-first snake facing `Right`, score `0` via Group A's `GameState::new`, and a fixed initial food clear of the snake; no `rand` yet), direction handling (`GameState::change_direction`, which rejects an immediate reversal through `Direction::opposite` and returns a `bool`), and the snake movement/growth mechanics (`Direction::offset` grid math, a `std::ops::Add` impl on `Position`, `Snake::advance` with a `should_remove_tail` flag, and `GameState::advance_one_step`). The growth flag is not yet wired to food consumption.
- **Pending (later Phase 1A groups):** food placement (Group C — TODO task 9), food consumption/scoring (task 10), boundary and self collision (tasks 11–12), and game-state transitions (task 13); core-logic tests (Group D — TODO task 14). The `GameStatus` variants exist but are not yet transitioned, and no game loop drives the logic yet.
- **Still absent:** no `Cargo.lock`, `Dockerfile`, `compose.yaml`, or `dist/` output. Nothing has been compiled — the Rust/Cargo toolchain is intentionally not installed here, so all code is hand-written and manually verified (no cargo execution in this workflow).
- Terminal rendering, arrow-key input, the interactive game loop, and Docker builds remain out of scope for Phase 1A (see the TODO's Out-of-Scope list); details in `architecture.md`/`tech.md` for these remain the planned design.

---

## Immediate Next Steps

Order matters — this is the project's live roadmap:

1. Implement the Docker build environment: `Dockerfile` + `compose.yaml` so that `docker compose run --rm build` produces `dist/snake.exe` (brief §3).
2. Implement the game in Rust: `Cargo.toml` and `src/main.rs` (code may live entirely in `main.rs` initially; split into modules only if justified later — brief §17).
3. Verify the game against the Definition of Done checklist in `brief.md` §18.
4. After implementation, update project info per `instructions.md` ("Project Info Update") — especially `context.md`.

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
