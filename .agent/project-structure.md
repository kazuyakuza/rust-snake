# Project Structure

# Folders in src/

- src/game/ - core game domain module tree (`position`, `direction`, `snake`, `food`, `state`)

# Rust project files

- Cargo.toml - Cargo package manifest (package `snake`, sole dependency `rand`)
- src/main.rs - crate entry point; declares the `game` module
- src/game.rs - `game` module root; declares the `game/*` submodules
- src/game/position.rs - grid cell coordinate value type (`x`, `y`)
- src/game/direction.rs - movement direction enum with opposite-direction detection
- src/game/snake.rs - snake as an ordered, head-first collection of positions
- src/game/food.rs - food newtype wrapping a `Position`
- src/game/state.rs - game state + status, board dimensions, and playable-bounds predicate

# Not yet present (later phases)

- Cargo.lock, Dockerfile, compose.yaml, dist/ - generated/defined by the later Docker build phase

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
  - project-info/ - brief.md (source of truth), product.md, context.md, architecture.md, tech.md, instructions.md
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- docs/ - Documentation files
