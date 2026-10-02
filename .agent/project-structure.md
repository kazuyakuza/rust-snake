# Project Structure

# Folders in src/

- src/game/ - core game domain module tree (`position`, `direction`, `snake`, `food`, `setup`, `state`, `collision`, `food_placement`)
- src/terminal/ - terminal UI module tree (game_loop, input, lifecycle, renderer)

# Rust project files

- Cargo.toml - Cargo package manifest (package snake, dependencies rand and crossterm)
- src/lib.rs - library crate root; exposes the game and terminal modules to the binary and the integration tests
- src/main.rs - binary entry point; wires initial setup, terminal enable, the start screen, the playing loop, the game-over screen, and exit cleanup (screens and key-wait as private in-file helpers)
- src/game.rs - `game` module root; declares the `game/*` submodules
- src/game/position.rs - grid cell coordinate value type (`x`, `y`)
- src/game/direction.rs - movement direction enum with opposite-direction detection
- src/game/snake.rs - snake as an ordered, head-first collection of positions
- src/game/food.rs - food newtype wrapping a `Position`
- src/game/setup.rs - initial game setup: starting snake/food/direction constants and the `GameStateSetup` seed
- src/game/state.rs - game state, board dimensions and playable bounds, status transitions, and the per-tick move/consume/collide driver
- src/game/collision.rs - death predicates for one step: boundary exit and tail-aware body overlap
- src/game/food_placement.rs - random free-cell food placement with explicit board-full handling
- src/terminal.rs - terminal module root; declares the terminal/* submodules
- src/terminal/game_loop.rs - fixed 120 ms playing loop: drains arrow directions, advances the domain, renders, and sleeps the tick remainder
- src/terminal/renderer.rs - full-frame board renderer over an io::Write output (borders, snake, food, score line)
- src/terminal/input.rs - arrow key press to game Direction mapping plus non blocking drains of the last event and of all buffered directions
- src/terminal/lifecycle.rs - raw mode, alternate screen and cursor visibility with Drop guard cleanup plus an output accessor

# Integration tests (tests/)

Eight headless integration test files for the deterministic core logic and the terminal flow; run via `cargo test` (execution arrives with the Docker build phase). Each file imports the modules through the library crate (`snake::game::*`, `snake::terminal::*`).

- tests/initial_state.rs - initial snake (length/head/segments), score, direction, status, and in-bounds/valid-food guarantees
- tests/direction.rs - accepted direction changes and rejection of immediate reversals
- tests/movement_and_growth.rs - per-step head/body movement, length preservation, the pre-play status gate, and `Snake::advance` growth/normal-step semantics
- tests/food_consumption_scoring.rs - food consumption at arrival, score increment, growth-by-one, and post-consumption respawn validity
- tests/collision.rs - boundary exit (unit + wall-death without wrap-around) and tail-aware self-collision predicates
- tests/food_placement.rs - food-placement guarantees: in-bounds, never on an occupied cell, last-free-cell `Some`, full-board `None`
- tests/gameplay_flow.rs - complete flow through tick: start gate, eat/grow, boundary & self collisions
- tests/terminal_modules.rs - arrow-key mapping, tick semantics, renderer snapshot via a Vec<u8> buffer

# Not yet present (later phases)

- Cargo.lock, Dockerfile, compose.yaml, dist/ - generated/defined by the later Docker build phase

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
  - project-info/ - brief.md (source of truth), product.md, context.md, architecture.md, tech.md, instructions.md
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- docs/ - Documentation files
