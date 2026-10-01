# Product — Rust Snake

## Problem Definition

- The project is a small, self-contained Snake game in Rust used as a practical learning exercise.
- The goal is learning Rust — structs, enums, collections, ownership/borrowing, loops, input handling, timers, modules, Windows compilation — not producing a production-quality game.
- Problem it solves for the user: a playable, understandable terminal game that exercises core Rust concepts and Docker-driven Windows cross-compilation.

---

## Product Goals

- Playable terminal/console Snake that runs directly on Windows.
- Straightforward, understandable Rust code over sophisticated game architecture.
- Successful build chain: Docker compiles → `dist/snake.exe` runs on Windows.
- Guiding principle chain:

```text
Simple Rust → Understandable code → Successful Windows build → Working Snake game
```

- First milestone: "Compile a Rust program in Docker, produce a Windows `.exe`, run it locally, and play Snake in the terminal."

---

## User Experience

- Terminal/console only; no graphical window, no engine, no sprites/assets; ASCII/Unicode glyphs may represent snake, food, borders, UI.
- Launch shows a start screen: `Press any key to start`; game starts on key press.
- Player controls direction with arrow keys; snake moves continuously; immediate reversal into itself is forbidden.
- Exactly one food item on the board always; eating it: score +1, snake length +1, food respawns at a random valid position.
- Score is visible during gameplay, above or below the board, e.g. `Score: 7`.
- Boundary or self collision → immediate `GAME OVER` screen with final score; application exits on key press; no restart system.

---

## Non-Goals

Not in scope for the initial version — authoritative list: `brief.md` §16 (19 items, grouped by category below, nothing dropped):

- **Graphics & media**: graphical UI, game engine, sprites, textures, sound, music, animations beyond normal terminal rendering, Windows-specific GUI APIs.
- **Multiplayer & persistence**: multiplayer, networking, save games, high-score persistence.
- **Game depth**: multiple levels, increasing difficulty, power-ups.
- **UX surface**: menus, pause functionality, settings, configuration files.

These may be considered in future experiments, but they are outside the initial scope.

---

## Success Criteria

`brief.md` §18 "Definition of Done" is the authoritative checklist (17 items, grouped below, nothing dropped):

- **Build chain** (items 1–3): the project builds successfully using Docker; Docker produces a Windows `.exe`; the `.exe` runs directly on Windows.
- **Start & initial state** (items 4–6): `Press any key to start` screen; the snake starts with exactly 3 blocks; the snake moves continuously.
- **Controls** (items 7–8): arrow keys control the direction; the snake cannot immediately reverse direction.
- **Food & score** (items 9–12, 15): food appears at a valid random position; eating food increases score by 1 and length by 1; the food respawns; the score is displayed during gameplay.
- **Game over** (items 13–14, 16–17): hitting the boundary and hitting the snake's own body each cause Game Over; Game Over displays the final score; a key press exits the application.
