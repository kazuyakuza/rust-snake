# Tech — Rust Snake

## Stack

- Language: Rust (brief §2). Edition 2021, package `snake`, version `0.2.0`.
- Target platform: Windows (brief §2).
- UI: terminal/console only — no graphical window, no external game engine, no sprites/graphical assets; ASCII/Unicode characters represent game elements (brief §2).
- Dependencies (`Cargo.toml`):
  - `rand 0.8` — random food placement (added Phase 1A).
  - `crossterm 0.29` — the terminal-handling dependency (added Phase 1B, commit `2c4f528`): raw mode, alternate-screen switching, cursor show/hide/move, full-frame clear, and non-blocking key/arrow event polling. Used only from `src/terminal/` and `src/main.rs`.

---

## Development Setup

- Docker is used only as the compilation environment; the resulting executable must run directly on Windows outside the container (brief §3).
- The Docker image will contain everything required to compile the Rust project for the Windows target (brief §3) — no local Rust toolchain is described by the brief.
- Current Phase 1A/1B workflow: all Rust code (domain, terminal layer, and tests) is hand-written and manually reviewed; `cargo build`/`cargo test` execution awaits the Phase 2 Docker stage — **nothing has been compiled yet**.
- AI-agent tooling: Kilo Code and/or opencode, per [AGENTS.md](../../AGENTS.md) (both already configured in this repo).

---

## Build & Output

- Build command (planned): `docker compose run --rm build` (brief §3 expected workflow).
- Output: Windows executable in a host-mounted output directory `dist/snake.exe`; run it directly with `snake.exe` from a Windows terminal (brief §3).
- Required project files for this to work: `Dockerfile`, `compose.yaml`, `Cargo.toml`, `src/` (brief §3 minimum list).
- Status: `Cargo.toml` (v0.2.0) and `src/` — both the `game` domain and the new `terminal` layer — are implemented (Phases 1A + 1B). `Dockerfile`, `compose.yaml`, and `Cargo.lock` remain for Phase 2; the Docker build will create `Cargo.lock` and compile the crate.
- Tests: eight integration files under `tests/` (six Phase 1A + `gameplay_flow.rs` and `terminal_modules.rs` from Phase 1B), 59 `#[test]` functions in total, are authored but **not yet executed**; they require a Rust/Cargo toolchain provided by the Phase 2 Docker build environment.

---

## Technical Constraints

- Fixed game speed 120 ms per movement; no acceleration required (brief §7); implemented as `TICK_DURATION` in `src/terminal/game_loop.rs`.
- No restart system, no persistence, no configuration files (brief §12, §16).
- Terminal-only rendering budget: the core domain is organized into `src/game/` modules (Phase 1A); the terminal layer was introduced in `src/terminal/` in Phase 1B and keeps the domain terminal-free (brief §17).
- Terminal lifecycle is guarded (raw mode + alternate screen + hidden cursor) and restored on exit/drop, so an interrupted run does not leave the host terminal in a broken state.

---

## Tool Usage Patterns

- Git flow and commits follow `.agent/WORKFLOWS.md` and `.kilo/commands/critical-workflow.md` (feature branches `feat/*` or `fix/*`; meaningful commit messages).
- Agents must evaluate project info files at the start of every task (`instructions.md` "Initial Instruction") and keep `context.md` current.
- CI/CD and additional dev tooling: none defined by the brief — only the Docker build workflow (brief §3) is defined so far.

---

## Pending Decisions

- ~~Exact terminal characters for snake body/head, food, and borders~~ — **resolved in Phase 1B** (see `src/terminal/renderer.rs`): head `●` (U+25CF), body `■` (U+25A0), food `◆` (U+25C6), empty ` ` (U+0020); ASCII borders `+` corners, `-` horizontal, `|` vertical; score line `Score: N`. Raw mode, alternate screen, cursor hiding, full-frame clear, and arrow-key input are handled through `crossterm` (brief §5, §13).
- ~~Exact Rust data-structure layout for `Snake`/`Game`~~ — resolved in Phase 1A: ordered head-first `Vec<Position>` for `Snake`, struct `GameState` (see `src/game/`).
- ~~Random number generation approach~~ — resolved in Phase 1A: the `rand` crate (see `Cargo.toml`).
- **Open:** compilation, test execution, and the Windows `.exe` build — deferred to Phase 2 (`20261001-todo-4.md`); no code has been built or run yet.

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
