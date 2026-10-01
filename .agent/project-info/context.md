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

- **Not implemented yet**: no `Cargo.toml`, no `Cargo.lock`, no `Dockerfile`, no `compose.yaml`, no Rust source (`src/` contains only `.gitkeep`); the `dist/` output directory does not exist yet.
- All build/game runtime details described in `architecture.md` and `tech.md` are the PLANNED design from `brief.md`, marked as pending until implemented.

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
