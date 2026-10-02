# Terminal UI (Phase 1B)

Detailed notes for AI agents on the terminal layer under `src/terminal/`. The
short user-facing summary lives in [`README.md`](../README.md#terminal-ui-phase-1b);
this file is the module map and integration contract.

## Status

Implemented now (Phase 1B, Groups A and B): the renderer, arrow-key input
mapping, lifecycle guard, and the game loop (`run_playing_loop`, `tick`) with
its chronological arrow-directions drain. **Not yet present** (Group C): the
start / game-over screens and the `main.rs` wiring that connects them; test
execution arrives with the Docker build phase (Phase 2). No local Rust
toolchain is configured in this environment — `cargo build` and `cargo test`
do not run here.

## File Map

| Path | Responsibility |
| --- | --- |
| `src/terminal.rs` | Module root; declares the `game_loop`, `input`, `lifecycle`, `renderer` submodules. |
| `src/terminal/game_loop.rs` | Fixed 120 ms playing loop and headless one-tick function. |
| `src/terminal/renderer.rs` | Full-frame board draw (borders, snake, food, score) to any `io::Write`. |
| `src/terminal/input.rs` | Arrow-key press to `Direction` mapping and non-blocking event drain. |
| `src/terminal/lifecycle.rs` | Raw mode / alternate screen / cursor toggle with a Drop-cleanup guard. |

The crate root `src/lib.rs` exposes `terminal` alongside `game`, so both the
binary and the integration tests reach the terminal API through `snake::terminal`.

## Design Principles

- **Rendering is read-only and stateless.** The renderer never mutates game
  state and holds no rules; it only reads `GameState` and writes characters.
- **Full-frame redraw, no scroll.** Each `render` moves the cursor to home
  (`MoveTo(0, 0)`) and rewrites the whole board, so the display stays stable and
  never scrolls.
- **Input mapping is pure.** `map_key_event_to_direction` is a standalone
  function over a `crossterm` `KeyEvent`, with no terminal side effects.
- **Reversal is a domain concern.** Rejecting an immediate reversal belongs to
  `src/game`, not the terminal layer — see the "no duplicated rules" constraint
  in the Phase 1B TODO. The input layer only reports the requested direction.
- **Cleanup is guaranteed.** `TerminalHandle` restores the terminal on both an
  explicit `disable()` and on `Drop`, including when setup partially fails.

## Public API (integration contract)

Generic over `W: std::io::Write` so the output can be a real terminal (e.g.
`std::io::stdout`) or an in-memory buffer for headless validation.

Renderer (`src/terminal/renderer.rs`):

- `Renderer::new(output: W) -> Renderer<W>`
- `Renderer::render(&mut self, state: &GameState) -> io::Result<()>`
  - Reads `game::state::{GameState, WIDTH, HEIGHT}` (board is 40 x 25) and the
    domain accessors `state.snake().head()` / `.segments()`, `state.food()`,
    `state.score()`.
  - Glyphs: head `●`, body `■`, food `◆`, borders `+ - |`, score line
    `Score: <n>`.

Input (`src/terminal/input.rs`):

- `map_key_event_to_direction(event: &KeyEvent) -> Option<Direction>`
  - `Up`/`Down`/`Left`/`Right` key **press** events map to the matching
    `game::direction::Direction`; every other key, release, or repeat returns
    `None`.
- `drain_arrow_event() -> io::Result<Option<KeyEvent>>`
  - Polls with `Duration::ZERO` (never blocks), consumes all pending events, and
    returns the last arrow-key press seen, if any.

Game loop (`src/terminal/game_loop.rs`):

- `run_playing_loop(&mut GameState, &mut Renderer<W>) -> io::Result<()>` —
  precondition `status() == Playing`; per tick: drain → apply → advance →
  render → sleep remainder; returns after rendering the final frame once status
  leaves `Playing`.
- `tick(&mut GameState, &[Direction], &mut Renderer<W>) -> io::Result<GameStatus>` —
  one headless transition, no sleep/terminal read; Group-D test hook (drives a
  `Vec<u8>` renderer).
- `drain_arrow_directions() -> io::Result<Vec<Direction>>` — all buffered arrow
  presses chronologically, never blocks.

Lifecycle (`src/terminal/lifecycle.rs`):

- `TerminalHandle::enable(output: W) -> io::Result<TerminalHandle<W>>`
  - Enables raw mode, enters the alternate screen, hides the cursor. If the
    screen/cursor writes fail, it disables raw mode before returning the error.
- `TerminalHandle::disable(&mut self) -> io::Result<()>`
  - Shows the cursor, leaves the alternate screen, disables raw mode; returns the
    first error encountered.
- `Drop for TerminalHandle` calls `disable()` and ignores errors, so a leaked or
  early-returned handle still restores the terminal.

## How the Next Group Connects It (planned, not yet implemented)

The playing loop itself is implemented (`run_playing_loop`); the surrounding
`main` wiring arrives with the next group:

1. Create a `TerminalHandle` over the chosen output and a `Renderer` writing to
   that output.
2. Each tick: `drain_arrow_directions()` → `run_playing_loop` applies
   `change_direction` (the domain rejects reversals) → `advance_one_step()` →
   `Renderer::render(&state)` → sleep the remainder of the tick.
3. On game-over: stop the loop, present the final score, wait for a key press,
   then exit. Returning/dropping the `TerminalHandle` restores the terminal.

Keep every gameplay rule (movement, collision, scoring, growth) in `src/game`.
The terminal layer stays responsible only for input, rendering, timing, and
lifecycle.

## How to Validate Manually Later

Once the loop and `main` wiring exist, run `dist/snake.exe` in a real Windows
terminal (via the Docker build phase) and check:

- Board draws once per tick and does **not** scroll; head and body glyphs are
  distinct.
- Arrow keys steer the snake; a press that would reverse into itself is ignored.
- On exit (including after game over) the cursor is visible and the terminal is
  back on its normal screen in cooked input mode.

Headless checks possible without a TTY (usable by future tests):

- **Renderer:** build a `GameState` and `render` into a `Vec<u8>` buffer, then
  assert the border/glyph/score text — the generic `io::Write` makes this
  straightforward.
- **Input:** call `map_key_event_to_direction` with constructed `KeyEvent` values
  (pure function, no terminal needed).
- **Lifecycle:** drive `TerminalHandle` over a buffer to confirm it emits the
  expected control sequences on `enable`/`disable` and that `Drop` restores.
