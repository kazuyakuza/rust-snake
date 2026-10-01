# Rust Snake — Project Brief

## 1. Project Overview

Build a small, self-contained **Snake game in Rust** as a learning and experimentation project.

The primary goal is **not** to create a production-quality game, but to use a simple game as a practical exercise for learning Rust, including structs, enums, collections, ownership/borrowing, loops, input handling, timers, modules, and Windows compilation.

The game should initially run entirely in a **Windows terminal/console**. No graphical game engine or GUI framework is required.

---

## 2. Technology

### Programming Language

* Rust

### Target Platform

* Windows

### User Interface

* Terminal/console-based
* No graphical window
* No external game engine
* No sprites or graphical assets

The game may use ASCII/Unicode characters to represent the Snake, food, borders, and UI elements.

---

## 3. Build Environment

The project must include a Docker-based build environment.

Docker should be used **only as the compilation environment**. The resulting executable must be usable directly on Windows outside the container.

Expected workflow:

```text
docker compose run --rm build
```

The build should produce a Windows executable in a host-mounted output directory, for example:

```text
dist/snake.exe
```

The executable should then be runnable directly from Windows:

```text
snake.exe
```

The project should therefore contain, at minimum:

```text
Dockerfile
compose.yaml
Cargo.toml
src/
```

The Docker image should contain everything required to compile the Rust project for the Windows target.

---

## 4. Game Concept

The game is a classic Snake implementation.

The player controls a snake moving continuously around a bounded rectangular game area.

The snake starts with:

* 1 head
* 2 body/tail segments

Total initial length:

```text
3 blocks
```

The player must collect food.

Every time the snake eats food:

* The score increases by `1`.
* The snake grows by exactly `1` block.
* A new food position is generated.

---

## 5. Game Area

The initial game board should use a logical grid rather than actual screen pixels.

Recommended initial dimensions:

```text
WIDTH  = 40 cells
HEIGHT = 25 cells
```

The board should have visible boundaries.

Conceptually:

```text
+----------------------------------------+
|                                        |
|                 ■■●                    |
|                                        |
|                            ●           |
|                                        |
|                                        |
+----------------------------------------+
```

Each cell represents one logical position that can contain:

* empty space
* snake head
* snake body
* food

The exact terminal character used for each element can be decided during implementation.

---

## 6. Player Controls

The player controls the Snake using the arrow keys:

```text
↑
↓
←
→
```

The Snake moves continuously in its current direction.

The player changes the current direction using the arrow keys.

The Snake should not be allowed to immediately reverse direction into itself.

For example:

```text
Moving RIGHT
```

should not allow:

```text
RIGHT → LEFT
```

in a single move.

---

## 7. Game Loop

The game should operate using a simple fixed update interval.

Initial recommended movement speed:

```text
120 ms per movement
```

Approximately:

```text
8.3 moves per second
```

No acceleration is required.

The speed should remain constant throughout the game.

Each game tick should approximately follow this sequence:

```text
1. Read/process player input.
2. Calculate the new head position.
3. Check boundary collision.
4. Check self collision.
5. Move the Snake.
6. Check whether the Snake ate the food.
7. If food was eaten:
      increase score
      grow Snake
      generate new food
8. Render the updated game state.
9. Wait for the next tick.
```

The implementation can adjust this ordering as necessary to accommodate terminal input handling.

---

## 8. Collision Rules

The player loses immediately when either of the following occurs.

### Boundary Collision

If the Snake's head moves outside the playable game area:

```text
GAME OVER
```

The Snake cannot wrap around the screen.

For example:

```text
RIGHT → right wall → GAME OVER
```

### Self Collision

If the Snake's head moves into one of its own body segments:

```text
GAME OVER
```

The Snake cannot pass through itself.

---

## 9. Food

There should always be exactly one food item on the board while the game is running.

Food should:

* Spawn at a random position.
* Never spawn outside the playable area.
* Never spawn on top of the Snake.

When the Snake's head reaches the food:

```text
Score += 1
Snake length += 1
```

Then a new food position is generated.

If necessary, the implementation should handle the edge case where there are no free cells left on the board.

---

## 10. Score

The game has a single counter:

```text
Score
```

Initial value:

```text
0
```

Every food item consumed increments the score by exactly:

```text
+1
```

The score should be visible during gameplay, preferably above or below the board.

Example:

```text
Score: 7
```

No additional statistics are required.

---

## 11. Game Start

When the executable launches, it should not immediately begin moving the Snake.

Instead, display a simple start screen:

```text
Press any key to start
```

The game begins after the player presses a key.

The initial game state should then be created/reset and gameplay begins.

---

## 12. Game Over

When a collision occurs, gameplay stops.

Display a simple game-over message, for example:

```text
GAME OVER

Score: 12

Press any key to exit
```

The application should terminate after the player presses a key.

No restart system is required for the initial version.

---

## 13. Initial Game State

At game start:

```text
Score = 0
Snake length = 3
```

Example:

```text
■■●
```

where:

* `●` = head
* `■` = body

The initial direction can be:

```text
RIGHT
```

The food should be placed at a random valid position that does not overlap the Snake.

---

## 14. Suggested Rust Data Model

The implementation should remain simple.

A possible conceptual model is:

```text
Snake
    Vec<Position>

Position
    x
    y

Direction
    Up
    Down
    Left
    Right

Food
    Position

Game
    Snake
    Food
    Direction
    Score
    GameState
```

The exact structure is left to the implementation.

The project should favor straightforward Rust code over unnecessary abstraction.

---

## 15. Rust Learning Objectives

The project is intentionally small so that Rust itself remains the main subject.

The implementation should provide an opportunity to practice:

* Structs
* Enums
* `Vec<T>`
* Functions
* Modules
* Pattern matching
* `Option` / `Result` where appropriate
* Ownership
* Borrowing
* Mutable references
* Iteration
* Loops
* Random number generation
* Terminal input
* Timers
* Error handling
* Project organization with Cargo
* Windows compilation
* Cross-compilation using Docker

The project should avoid unnecessary architectural complexity.

---

## 16. Non-Goals

The initial version should **not** include:

* A graphical UI
* A game engine
* Sprites
* Textures
* Sound
* Music
* Multiplayer
* Networking
* Save games
* High-score persistence
* Multiple levels
* Increasing difficulty
* Power-ups
* Menus
* Pause functionality
* Settings
* Configuration files
* Animations beyond normal terminal rendering
* Windows-specific GUI APIs

These can be considered in future experiments, but they are outside the scope of this test.

---

## 17. Proposed Project Structure

A simple initial structure is preferred:

```text
rust-snake/
│
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── compose.yaml
│
├── src/
│   └── main.rs
│
└── dist/
    └── snake.exe
```

The code can initially live entirely in `main.rs`.

If the implementation becomes large enough to justify it, functionality can later be split into modules.

---

## 18. Definition of Done

The project is considered complete when all of the following are true:

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

---

## 19. Guiding Principle

**Keep it simple.**

This project is primarily a Rust learning exercise.

The implementation should prioritize:

```text
Simple Rust
    ↓
Understandable code
    ↓
Successful Windows build
    ↓
Working Snake game
```

rather than introducing sophisticated game architecture.

The first milestone is simply:

> **Compile a Rust program in Docker, produce a Windows `.exe`, run it locally, and play Snake in the terminal.**

<!-- DO NOT DELETE NEXT SECTION -->

## Important Note for AI Agents

All agents working on this project MUST adhere to the workflows and rules outlined in [AI Agent Onboarding document](../../AGENTS.md).

Before starting any task:

1. **Review `AGENTS.md`**: is the primary source of instructions for agents.
2. **Follow Workflows**: follow the procedures defined in `.agent/WORKFLOWS.md`, especially the `.kilo/commands/critical-workflow.md`.

<!-- END DO NOT DELETE -->
