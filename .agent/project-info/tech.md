# Tech — Rust Snake

## Stack

- Language: Rust (brief §2).
- Target platform: Windows (brief §2).
- UI: terminal/console only — no graphical window, no external game engine, no sprites/graphical assets; ASCII/Unicode characters represent game elements (brief §2).

---

## Development Setup

- Docker is used only as the compilation environment; the resulting executable must run directly on Windows outside the container (brief §3).
- The Docker image contains everything required to compile the Rust project for the Windows target (brief §3) — no local Rust toolchain is described by the brief.
- Current Phase 1A workflow: all Rust code is hand-written and manually reviewed; `cargo build`/`cargo test` execution awaits the Docker phase.
- AI-agent tooling: Kilo Code and/or opencode, per [AGENTS.md](../../AGENTS.md) (both already configured in this repo).

---

## Build & Output

- Build command: `docker compose run --rm build` (brief §3 expected workflow).
- Output: Windows executable in a host-mounted output directory `dist/snake.exe`; run it directly with `snake.exe` from Windows (brief §3).
- Required project files for this to work: `Dockerfile`, `compose.yaml`, `Cargo.toml`, `src/` (brief §3 minimum list).
- Status: `Cargo.toml` and `src/` are implemented (Phase 1A). `Dockerfile`, `compose.yaml`, and `Cargo.lock` remain planned — the Docker build phase will create and compile them.
- Tests: six integration test files under `tests/` are authored but **not yet executed**; they require a Rust/Cargo toolchain provided by the Docker build environment.

---

## Technical Constraints

- Fixed game speed 120 ms per movement; no acceleration required (brief §7).
- No restart system, no persistence, no configuration files (brief §12, §16).
- Terminal-only rendering budget: the core domain is organized into `src/game/` modules (Phase 1A); the terminal layer will be introduced in later phases as needed (brief §17).

---

## Tool Usage Patterns

- Git flow and commits follow `.agent/WORKFLOWS.md` and `.kilo/commands/critical-workflow.md` (feature branches `feat/*` or `fix/*`; meaningful commit messages).
- Agents must evaluate project info files at the start of every task (`instructions.md` "Initial Instruction") and keep `context.md` current.
- CI/CD and additional dev tooling: none defined by the brief — only the Docker build workflow (brief §3) is defined so far.

---

## Pending Decisions

- Exact terminal characters for snake body/head, food, and borders — decided during implementation (brief §5). *Still open (Phase 1B).*
- ~~Exact Rust data-structure layout for `Snake`/`Game`~~ — resolved in Phase 1A: ordered head-first `Vec<Position>` for `Snake`, struct `GameState` (see `src/game/`).
- ~~Random number generation approach~~ — resolved in Phase 1A: the `rand` crate is the sole dependency (see `Cargo.toml`).

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
