# Project Structure

# Folders in src/

- src/game/ - core game domain module tree (`position`, `direction`, `snake`, `food`, `setup`, `state`, `collision`, `food_placement`)

# Rust project files

- Cargo.toml - Cargo package manifest (package `snake`, sole dependency `rand`)
- src/main.rs - crate entry point; declares the `game` module
- src/game.rs - `game` module root; declares the `game/*` submodules
- src/game/position.rs - grid cell coordinate value type (`x`, `y`)
- src/game/direction.rs - movement direction enum with opposite-direction detection
- src/game/snake.rs - snake as an ordered, head-first collection of positions
- src/game/food.rs - food newtype wrapping a `Position`
- src/game/setup.rs - initial game setup: starting snake/food/direction constants and the `GameStateSetup` seed
- src/game/state.rs - game state, board dimensions and playable bounds, status transitions, and the per-tick move/consume/collide driver
- src/game/collision.rs - death predicates for one step: boundary exit and tail-aware body overlap
- src/game/food_placement.rs - random free-cell food placement with explicit board-full handling

# Not yet present (later phases)

- Cargo.lock, Dockerfile, compose.yaml, dist/ - generated/defined by the later Docker build phase

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
  - project-info/ - brief.md (source of truth), product.md, context.md, architecture.md, tech.md, instructions.md
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- docs/ - Documentation files
