# Architecture — Rust Snake

**Status:** Planned — not yet implemented. The planned files `Cargo.toml`, `Dockerfile`, `compose.yaml`, and `src/main.rs` do not exist yet.

## System Architecture

- Single terminal application; the game state is a logical grid, not pixels (brief §5).
- Current status: planned / not yet implemented — the planned files `Cargo.toml`, `Dockerfile`, `compose.yaml`, and `src/main.rs` do not exist yet.

---

## Planned Repository Layout

```text
rust-snake/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── compose.yaml
├── src/
│   └── main.rs
└── dist/
    └── snake.exe
```

- Code starts entirely in `main.rs`; module split only if justified later (brief §17).
- `dist/` is a host-mounted build output folder, ignored by git (`.gitignore`).

---

## Core Components

Suggested model — exact structure left to the implementation (brief §14):

- `Position` — `x`, `y` coordinates.
- `Direction` — enum `Up | Down | Left | Right`.
- `Snake` — `Vec<Position>`.
- `Food` — a `Position`.
- `Game` — holds `Snake`, `Food`, `Direction`, `Score`, `GameState`.
- Constraint: favor straightforward Rust over unnecessary abstraction (brief §14).

---

## Game Loop

- Fixed tick interval: 120 ms per movement (~8.3 moves per second); constant speed; no acceleration (brief §7).
- Per-tick sequence:

1. Read/process player input.
2. Calculate the new head position.
3. Check boundary collision.
4. Check self collision.
5. Move the snake.
6. Check whether the snake ate the food.
7. If food was eaten: increase score, grow snake, generate new food.
8. Render the updated game state.
9. Wait for the next tick.

- Note: the implementation may adjust this ordering to accommodate terminal input handling (brief §7).

---

## Game Layout & Input Rules

- Board: logical grid, initial `WIDTH = 40`, `HEIGHT = 25` cells; visible boundaries; no screen wrap (brief §5, §8).
- Cell states: empty, snake head, snake body, food; exact terminal characters decided during implementation (brief §5, §13).
- Snake starts with 1 head + 2 body segments = 3 blocks; initial direction `RIGHT`; food spawns at a random valid non-overlapping position (brief §4, §13).
- Controls: arrow keys only; movement is continuous; immediate reversal (e.g. `RIGHT` → `LEFT` in one move) must be rejected (brief §6).

---

## Collision & Food Rules

- Head outside the play area → game over; no wrap-around (brief §8, §9).
- Head onto own body segment → game over (brief §8).
- Always exactly one food on the board; spawns randomly, never outside the play area, never on the snake; handles the edge case of no free cells left (brief §9).

---

## Screens & States

- Start screen on launch: `Press any key to start`; gameplay begins after a key press (brief §11).
- Playing state: board rendered each tick with the score visible, e.g. `Score: 7` (brief §10).
- Game over state: gameplay stops; the screen shows `GAME OVER` with the final score; the application exits on key press; no restart system (brief §12).

---

## Design Patterns / Constraints

- Keep-it-simple principle (brief §19): simple Rust → understandable code → successful Windows build → working game.
- No menus/pause/settings/config files — see `product.md` Non-Goals.

---

## Critical Paths

- Build: `docker compose run --rm build` → host-mounted output `dist/snake.exe` (brief §3).
- Run: execute `dist/snake.exe` directly on Windows terminal, outside the container (brief §3).
- Develop: read `AGENTS.md` → `.agent/WORKFLOWS.md` → Critical Workflow (`.kilo/commands/critical-workflow.md`); project context lives in `.agent/project-info/*`.
