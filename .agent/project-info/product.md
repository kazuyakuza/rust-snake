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

The initial version should not include:

- A graphical UI
- A game engine
- Sprites
- Textures
- Sound
- Music
- Multiplayer
- Networking
- Save games
- High-score persistence
- Multiple levels
- Increasing difficulty
- Power-ups
- Menus
- Pause functionality
- Settings
- Configuration files
- Animations beyond normal terminal rendering
- Windows-specific GUI APIs

These may be considered in future experiments, but they are outside the initial scope.

---

## Success Criteria

`brief.md` §18 "Definition of Done" is the authoritative checklist. Summary of its 17 points:

1. The project builds successfully using Docker.
2. Docker produces a Windows `.exe`.
3. The `.exe` runs directly on Windows.
4. The game starts with a `Press any key to start` screen.
5. The Snake starts with exactly 3 blocks.
6. The Snake moves continuously.
7. Arrow keys control the direction.
8. The Snake cannot immediately reverse direction.
9. A food item appears at a valid random position.
10. Eating food increases the score by 1.
11. Eating food increases the Snake length by 1.
12. The food respawns after being eaten.
13. Hitting the board boundary causes Game Over.
14. Hitting the Snake's own body causes Game Over.
15. The score is displayed during gameplay.
16. Game Over displays the final score.
17. A key press exits the application after Game Over.
