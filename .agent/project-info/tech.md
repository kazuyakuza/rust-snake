# Tech — Rust Snake

## Stack

- Language: Rust (brief §2).
- Target platform: Windows (brief §2).
- UI: terminal/console only — no graphical window, no external game engine, no sprites/graphical assets; ASCII/Unicode characters represent game elements (brief §2).

---

## Development Setup

- Docker is used only as the compilation environment; the resulting executable must run directly on Windows outside the container (brief §3).
- The Docker image contains everything required to compile the Rust project for the Windows target (brief §3) — no local Rust toolchain is described by the brief.
- AI-agent tooling: Kilo Code and/or opencode, per [AGENTS.md](../../AGENTS.md) (both already configured in this repo).

---

## Build & Output

- Build command: `docker compose run --rm build` (brief §3 expected workflow).
- Output: Windows executable in a host-mounted output directory `dist/snake.exe`; run it directly with `snake.exe` from Windows (brief §3).
- Required project files for this to work: `Dockerfile`, `compose.yaml`, `Cargo.toml`, `src/` (brief §3 minimum list).
- Note: none of these files exist yet — implementation pending.

---

## Technical Constraints

- Fixed game speed 120 ms per movement; no acceleration required (brief §7).
- No restart system, no persistence, no configuration files (brief §12, §16).
- Terminal-only rendering budget: keep the implementation in `main.rs` unless growth justifies modules (brief §17).

---

## Tool Usage Patterns

- Git flow and commits follow `.agent/WORKFLOWS.md` and `.kilo/commands/critical-workflow.md` (feature branches `feat/*` or `fix/*`; meaningful commit messages).
- Agents must evaluate project info files at the start of every task (`instructions.md` "Initial Instruction") and keep `context.md` current.
- CI/CD and additional dev tooling: none defined by the brief — only the Docker build workflow (brief §3) is defined so far.

---

## Pending Decisions

- Exact terminal characters for snake body/head, food, and borders — decided during implementation (brief §5).
- Exact Rust data-structure layout for `Snake`/`Game` — left to the implementation (brief §14).
- Random number generation approach — a Rust learning objective in the brief; crate vs. hand-rolled choice is an implementation decision (brief §15).

---

## Important Note for AI Agents

Agents working on this project must follow the onboarding, workflows, and rules in [AGENTS.md](../../AGENTS.md), and treat [brief.md](brief.md) as the source of truth for scope.
