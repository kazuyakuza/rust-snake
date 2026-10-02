# Front-end Technical Specification — Phase 1B Group B: Game Loop, Wiring, and Rendering Refinement

**TODO source:** `.agent/todos/20261001/20261001-todo-3.md`  
**Group scope:** Tasks 3 (Implement the Game Loop), 4 (Connect Movement, Input, and Game State), and 8 (Refine Terminal Rendering Behavior) only.  
**Branch:** `feat/phase1b-terminal-game`  
**Global plan:** `.kilo/plans/20261001-phase1b-terminal-game.md`  
**Group A baseline:** `.kilo/plans/20261001-phase1b-terminal-game-groupa-frontend-spec.md` and the committed `src/terminal/` primitives (`renderer.rs`, `input.rs`, `lifecycle.rs`).

---

## 1. Scope & Boundaries

This specification covers Group B only:

- **Task 3 — Implement the Game Loop:** a fixed 120 ms playing loop that polls input, advances the domain state, renders, and waits for the next tick.
- **Task 4 — Connect Movement, Input, and Game State:** wire the existing Group A terminal primitives to the Phase 1A domain API without duplicating gameplay rules.
- **Task 8 — Refine Terminal Rendering Behavior:** confirm that the existing full-frame overwrite renderer already provides a stable, non-scrolling display; define any loop-side discipline required to keep it stable.

**Explicitly out of scope for this spec (handled by other groups):**

- Start-screen wait loop and `start_playing()` trigger (Task 5, Group C).
- Game-over screen text, final-score presentation, and exit-on-key behavior (Task 6, Group C).
- `src/main.rs` wiring that orchestrates lifecycle, start screen, playing loop, and game-over screen (Group C).
- Automated flow validation and test authoring (Tasks 9 and 10, Group D).

---

## 2. Target Framework & Dependencies

- **Language:** Rust, 2021 edition.
- **Existing dependencies:** `rand = "0.8"` (domain), `crossterm = "0.29"` (terminal primitives from Group A).
- **No new dependencies** are added in Group B.
- **Domain API (source of truth, do not redesign):** `crate::game::state::{GameState, GameStatus, WIDTH, HEIGHT}` and `crate::game::direction::Direction`.
- **Group A primitives consumed (frozen signatures, do not change):**
  - `crate::terminal::renderer::Renderer<W: Write>::render(&mut self, state: &GameState) -> io::Result<()>`.
  - `crate::terminal::input::map_key_event_to_direction(&KeyEvent) -> Option<Direction>`.
  - `crate::terminal::input::drain_arrow_event() -> io::Result<Option<KeyEvent>>`.

---

## 3. Component Structure

Group B adds one new file to the `src/terminal/` tree introduced by Group A:

```text
src/
├── lib.rs            unchanged: pub mod game; pub mod terminal;
├── main.rs           out of scope for Group B (Group C wires it)
└── terminal/
    ├── renderer.rs   Group A baseline (no behavior change)
    ├── input.rs      Group A baseline + one new helper for the loop
    ├── lifecycle.rs  Group A baseline (no behavior change)
    └── game_loop.rs  NEW — playing loop and single-tick logic
```

`src/terminal.rs` must declare the new submodule alphabetically alongside the existing ones:

```rust
pub mod game_loop;
pub mod input;
pub mod lifecycle;
pub mod renderer;
```

Each file must respect the project `max-lines-per-file` rule (≤200 lines; target ≤125 code lines).

---

## 4. Game Loop Design (Task 3)

### 4.1 Tick Timing

- Fixed tick duration: **120 ms** per movement step.
- Constant: `const TICK_DURATION: Duration = Duration::from_millis(120);`.
- The loop measures wall-clock time at the start of each tick and sleeps only the **remaining** time after input, domain advance, and render work completes.
- If the work of a tick exceeds 120 ms, the loop does **not** try to catch up multiple ticks; it simply proceeds to the next tick immediately.

```rust
fn sleep_remaining(tick_start: Instant) {
    let elapsed = tick_start.elapsed();
    if elapsed < TICK_DURATION {
        thread::sleep(TICK_DURATION - elapsed);
    }
}
```

### 4.2 Input Must Never Block Movement

- The Group A helper `drain_arrow_event()` already polls with `event::poll(Duration::ZERO)` and never blocks.
- Group B adds a new non-blocking helper `drain_arrow_directions()` that returns **all** buffered arrow-key directions in chronological order.
- Because input polling is zero-timeout, the 120 ms tick cadence is preserved regardless of whether the player is pressing keys.

### 4.3 Per-Tick Sequence

Each tick follows this exact order:

1. Record `tick_start = Instant::now()`.
2. Drain buffered arrow events: `let directions = drain_arrow_directions()?;`.
3. Apply each direction in order to the domain: for each `direction` in `directions`, call `state.change_direction(direction);`.
   - The domain rejects immediate reversals; the terminal layer ignores the returned `bool`.
4. Advance the game state: `state.advance_one_step();`.
   - This resolves movement, collision, food consumption, scoring, growth, and the `WaitingToStart`/`Playing` tick gate.
5. Render the updated state: `renderer.render(state)?;`.
6. Sleep the remainder of the tick: `sleep_remaining(tick_start);`.

### 4.4 WaitingToStart Behavior

- **This spec defines only the playing-loop contract.**
- The playing loop is entered only when `state.status() == GameStatus::Playing`.
- The caller (Group C) is responsible for the start-screen wait loop and for invoking `state.start_playing()` before calling the playing loop.
- If the playing loop is somehow invoked while `WaitingToStart`, `advance_one_step()` is a domain-level no-op, but this is **not** the intended entry point and behavior is unspecified for Group B.

### 4.5 GameOver Behavior

- The loop continues while `state.status() == GameStatus::Playing`.
- When `advance_one_step()` transitions the status to `GameOver`, the loop still renders the final frame (step 5) so the player sees the snake in its losing position, then exits before the next tick.
- The loop returns `Ok(())` on reaching `GameOver`.
- Game-over screen presentation and exit-on-key are deferred to Group C.

### 4.6 Loop Contract

```rust
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::thread;

use crate::game::direction::Direction;
use crate::game::state::{GameState, GameStatus};
use crate::terminal::input::drain_arrow_directions;
use crate::terminal::renderer::Renderer;

const TICK_DURATION: Duration = Duration::from_millis(120);

/// Run the fixed 120 ms playing loop until the game reaches GameOver.
/// 
/// Precondition: `state.status()` is `Playing`.
/// The loop polls input, applies direction changes, advances the domain,
/// renders, and sleeps the remainder of each tick.
pub fn run_playing_loop<W: Write>(
    state: &mut GameState,
    renderer: &mut Renderer<W>,
) -> io::Result<()> {
    while state.status() == GameStatus::Playing {
        let tick_start = Instant::now();
        let directions = drain_arrow_directions()?;
        apply_directions(state, &directions);
        state.advance_one_step();
        renderer.render(state)?;
        sleep_remaining(tick_start);
    }
    Ok(())
}
```

---

## 5. Single-Tick Logic and Testability (Task 3 / Group D Hook)

To make one tick testable without real time or a real terminal, extract the body of the loop into a pure, headless function:

```rust
/// Execute exactly one playing tick with the supplied input directions.
/// 
/// Applies each direction in order, advances the domain once, renders the
/// resulting state, and returns the new status.
pub fn tick<W: Write>(
    state: &mut GameState,
    directions: &[Direction],
    renderer: &mut Renderer<W>,
) -> io::Result<GameStatus> {
    apply_directions(state, directions);
    state.advance_one_step();
    renderer.render(state)?;
    Ok(state.status())
}
```

- `tick` does **not** sleep and does **not** read the real terminal.
- Group D tests can construct a `GameState`, call `tick` with a chosen slice of `Direction`s, capture output in a `Vec<u8>` renderer, and assert on the resulting status/score/rendered frame.
- `run_playing_loop` is the thin wrapper that adds timing and real input polling around `tick`.

### 5.1 Direction Application Helper

```rust
fn apply_directions(state: &mut GameState, directions: &[Direction]) {
    for &direction in directions {
        state.change_direction(direction);
    }
}
```

---

## 6. Input Helper Extension (Task 4)

Group A provides `drain_arrow_event()`, which returns only the **last** buffered arrow press. Group B needs every buffered arrow press in chronological order, so extend `src/terminal/input.rs` with:

```rust
/// Drain all currently available input events without waiting, and return the
/// arrow-key directions seen, in chronological order. Never blocks.
pub fn drain_arrow_directions() -> io::Result<Vec<Direction>> {
    let mut directions = Vec::new();
    while event::poll(Duration::ZERO)? {
        if let Some(direction) = read_arrow_direction() {
            directions.push(direction);
        }
    }
    Ok(directions)
}

fn read_arrow_direction() -> Option<Direction> {
    let event = event::read().ok()?;
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    map_key_event_to_direction(&key_event)
}
```

- Keep the existing `drain_arrow_event()` unchanged; it remains available for future use.
- Reuse `map_key_event_to_direction` so the definition of "arrow key" stays single-source.
- Ignore non-arrow events, release events, and repeat events through the existing press filter inside `map_key_event_to_direction`.

---

## 7. Wiring Contract (Task 4)

The terminal layer must interact with the domain exclusively through the public Phase 1A API. It must **not** duplicate domain rules.

### 7.1 Rules the Terminal Layer Consumes

| Domain capability | Terminal usage | Why |
|---|---|---|
| `GameState::change_direction(Direction) -> bool` | Apply each drained arrow direction. | Reversal rejection is enforced by the domain; the terminal layer does not check opposites. |
| `GameState::advance_one_step()` | Called once per tick. | Resolves movement, boundary/self collision, food consumption, scoring, growth, and food respawn. |
| `GameState::status() -> GameStatus` | Checked at the top of each loop iteration and returned by `tick`. | Detects `GameOver` to stop the loop. |
| `GameState::start_playing()` | **Not called by Group B.** | Group C invokes this from the start-screen wait loop. |
| `GameState::score() -> i32`, `snake()`, `food()` | Read only by `Renderer::render`. | Display only; no gameplay logic. |

### 7.2 Rules the Terminal Layer Must Not Duplicate

- Movement, collision detection, scoring, snake growth, food respawn, and direction-reversal rejection.
- Board-boundary logic (`WIDTH`, `HEIGHT`, `is_inside_board`).
- Food-placement randomization.

### 7.3 Responsibility Split

- **Domain (`src/game/`):** what the game state is and how it changes per tick.
- **Terminal (`src/terminal/`):** when to tick, how to collect input, how to draw the state, and how to restore the terminal.

---

## 8. Rendering Refinement (Task 8)

### 8.1 What Group A Already Provides

Group A's renderer already satisfies the stability requirements of Task 8:

- `MoveTo(0, 0)` at the start of every `render` call.
- Full-frame overwrite of `HEIGHT + 3` rows × `WIDTH + 2` columns.
- CRLF line endings (`\r\n`) so rows do not drift in raw mode.
- A single `flush()` per `render` call.
- Fixed dimensions; no scrolling; no partial updates.

### 8.2 What Group B Must Ensure

No changes to `renderer.rs` are required. Group B contributes only loop-side discipline:

1. **One render per tick:** `renderer.render(state)` is called exactly once per tick, after `advance_one_step()`.
2. **Final frame on GameOver:** the loop renders the frame that shows the snake in the losing position before exiting.
3. **No extra output between renders:** the loop must not `println!`, `eprintln!`, or emit other text that would scroll or corrupt the frame.
4. **No flush outside `Renderer::render`:** the loop relies on `render` to flush; do not add extra flushes.

### 8.3 Leftover Cells

- The snake never shrinks in this game; it only grows by one segment when eating food.
- Because every `render` overwrites the full frame, there is no residue from previous frames.
- No special "clear old tail" logic is required beyond the existing full-frame redraw.

---

## 9. Domain API Integration Contract

The terminal layer interacts with the domain exclusively through the existing Phase 1A API:

- Read board dimensions from `state::WIDTH` and `state::HEIGHT` (via the renderer).
- Render from `GameState` via `snake()`, `food()`, `score()`, and `status()`.
- Apply input via `GameState::change_direction(Direction) -> bool`.
- Advance the game via `GameState::advance_one_step()`.
- Detect game-over via `GameState::status() == GameStatus::GameOver`.

No terminal module may:

- Reimplement movement, collision, scoring, growth, or direction-reversal rules.
- Mutate `GameState` except by calling `change_direction` and `advance_one_step`.
- Depend on internal domain modules other than `state` and `direction`.

---

## 10. Testability Hooks

Group B must leave the code structured so Group D can validate it headlessly:

- **`tick(state, directions, renderer)`** exposes one complete state transition without sleeping or reading the real terminal.
- **`Renderer<W>`** is generic over `std::io::Write`; tests can pass a `Vec<u8>` and assert border/glyph/score text.
- **`map_key_event_to_direction`** remains a pure function over constructed `crossterm::event::KeyEvent` values.
- **`drain_arrow_directions()`** is the only terminal-reading input function in Group B; tests bypass it by calling `tick` directly.
- **Status transitions** are observable through `state.status()` after each `tick` call.

No automated test in Group B/D may depend on actual keyboard input, a visible terminal window, or real wall-clock delays.

---

## 11. Responsive Behavior

- The game uses a fixed logical board of 40×25 cells.
- No dynamic terminal resizing, centering, or scaling is required.
- Minimum comfortable terminal size remains 42 columns × 28 rows.
- If the terminal window is smaller, the display may be clipped; no special handling is required.

---

## 12. Accessibility

- Accessibility considerations for a terminal game are minimal.
- The chosen glyphs (`●`, `■`, `◆`) provide clear visual distinction between head, body, and food.
- No color-dependent information is used.

---

## 13. Performance Budget

- Each render emits O(`WIDTH` × `HEIGHT`) characters, approximately 1,050 cell glyphs plus borders and score line.
- Full redraw at 120 ms is well within terminal throughput limits.
- Input polling is zero-timeout, so no tick is delayed waiting for a key press.
- No animation frames, partial updates, or off-screen buffering.

---

## 14. Constraints Compliance

- **No GUI/engine/framework beyond crossterm:** `crossterm` is used only for the terminal primitives already in place. Group B adds only timing and wiring.
- **No blocking on input:** all input polling uses `Duration::ZERO`.
- **Terminal layer responsibilities only:** input collection, rendering, timing, and domain wiring. Gameplay rules stay in `src/game/`.
- **Keep it simple:** no colors, menus, pause, restart, scoring persistence, or configuration.
- **Windows console:** `crossterm` default features provide the required support.

---

## 15. Acceptance Criteria

- [ ] `src/terminal/game_loop.rs` exists and declares `TICK_DURATION = 120 ms`, `run_playing_loop`, and `tick`.
- [ ] `run_playing_loop` iterates while `state.status() == Playing`, polls input, applies directions, advances, renders, and sleeps the remainder.
- [ ] `tick` performs one headless state transition (input → advance → render) and returns the new `GameStatus`.
- [ ] `drain_arrow_directions` returns all buffered arrow-key directions in chronological order without blocking.
- [ ] Direction changes are applied only via `GameState::change_direction`; no reversal logic in the terminal layer.
- [ ] Movement, collision, scoring, growth, and food respawn are handled only by `GameState::advance_one_step`.
- [ ] The loop renders exactly once per tick and renders the final frame before exiting on `GameOver`.
- [ ] No extra terminal output (no `println!`, `eprintln!`, extra flushes, or scrolling).
- [ ] Group A files (`renderer.rs`, `input.rs`, `lifecycle.rs`) retain their frozen public contracts; only `input.rs` is extended with `drain_arrow_directions`.
- [ ] All new files stay under the 200-line project limit.

---

## 16. Notes for Implementer

- Do **not** modify `src/main.rs` in Group B; Group C owns the start/game-over orchestration.
- Do **not** modify `src/game/**` in Group B; the domain API is frozen.
- Do **not** add start-screen or game-over screen text in Group B.
- When in doubt about timing or input behavior, prefer the domain's existing tick gate and the non-blocking `Duration::ZERO` poll already established by Group A.
