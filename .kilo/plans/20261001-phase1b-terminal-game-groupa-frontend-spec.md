# Front-end Technical Specification — Phase 1B Group A: Terminal Primitives

**TODO source:** `.agent/todos/20261001/20261001-todo-3.md`  
**Group scope:** Tasks 1 (Terminal Rendering), 2 (Terminal Input), and 7 (Handle Terminal Lifecycle) only.  
**Branch:** `feat/phase1b-terminal-game`  
**Global plan:** `.kilo/plans/20261001-phase1b-terminal-game.md`

---

## 1. Scope & Boundaries

This specification covers the three terminal-layer primitives assigned to Group A:

- **Task 1 — Terminal Rendering:** full-frame board renderer.
- **Task 2 — Terminal Input:** arrow-key → `Direction` mapping and non-blocking polling contract.
- **Task 7 — Terminal Lifecycle:** raw mode, alternate screen, cursor visibility, and guaranteed cleanup.

**Out of scope for this spec (handled by other groups):**

- The 120 ms game loop itself (Task 3).
- Movement/input/state wiring (Task 4).
- Start screen and game-over screen text/presentation (Tasks 5, 6).
- Rendering refinement such as color, centering, or dynamic resizing (Task 8).
- Full gameplay-flow validation and automated tests (Tasks 9, 10).

---

## 2. Target Framework & Dependencies

- **Language:** Rust, 2021 edition.
- **Existing dependency:** `rand = "0.8"` (domain layer, unchanged).
- **New dependency (single addition):** `crossterm` stable line, pinned to the version confirmed at planning time (e.g. `0.28`).
  - Rationale: provides cross-platform raw input, non-blocking `event::poll`, cursor control, and alternate-screen support without becoming a game engine or advanced terminal framework.
- **Domain API (source of truth, do not redesign):** `crate::game::state::{GameState, GameStatus, WIDTH, HEIGHT}` and `crate::game::direction::Direction`.

---

## 3. Component Structure

Add a new `src/terminal/` module tree to the library crate so pure parts remain reachable from tests.

```text
src/
├── lib.rs            add `pub mod terminal;`
├── main.rs           thin entry point (wiring handled by later groups)
└── terminal/
    ├── mod.rs        module root; declares submodules
    ├── renderer.rs   Task 1
    ├── input.rs      Task 2
    └── lifecycle.rs  Task 7
```

Each file must respect the project `max-lines-per-file` rule (≤200 lines; target ≤125 code lines).

---

## 4. Renderer Design (Task 1)

### 4.1 Visual Tokens

| Element | Glyph | Unicode | Notes |
|---|---|---|---|
| Snake head | `●` | U+25CF | Must be visually distinct from body. |
| Snake body | `■` | U+25A0 | All non-head segments. |
| Food | `◆` | U+25C6 | One per board. |
| Empty cell | ` ` | space | Playable background. |
| Top/bottom-left corner | `+` |  |  |
| Top/bottom-right corner | `+` |  |  |
| Horizontal border | `-` |  | Repeated `WIDTH` times. |
| Vertical border | `\|` |  | Left and right edges of every board row. |
| Score prefix | `Score: ` |  | Followed by the integer score. |

No colors or styling effects are required in this phase.

### 4.2 Frame Layout

The frame is a logical grid surrounded by an ASCII border, with the score line directly beneath the bottom border.

- Board logical cells: `WIDTH` columns × `HEIGHT` rows.
- Border dimensions: `WIDTH + 2` columns × `HEIGHT + 2` rows.
- Score line: one additional row below the border containing `Score: {score}`.
- Total rendered rows: `HEIGHT + 3`.
- Total rendered columns: `WIDTH + 2`.

Example for a tiny conceptual board (actual `WIDTH=40`, `HEIGHT=25`):

```text
+-----+
|     |
| ■●◆ |
|     |
+-----+
Score: 0
```

### 4.3 Coordinate Mapping

- Logical `Position { x, y }` lives in `0..WIDTH` and `0..HEIGHT`.
- Terminal column for logical `x`: `x + 1` (one cell inside the left border).
- Terminal row for logical `y`: `y + 1` (one cell inside the top border).
- When rendering a board row, iterate `x = 0..WIDTH` and emit one glyph per logical cell.

### 4.4 Cell Precedence

When deciding what glyph to emit for a logical cell, check in this order:

1. Snake head (`snake.head()` == cell) → head glyph.
2. Any other snake segment → body glyph.
3. Food position (`food.position()` == cell) → food glyph.
4. Otherwise → empty space.

Head takes precedence over body and food (body/food overlap is not expected in a valid state, but the rule must be deterministic).

### 4.5 Redraw Strategy

- Full-frame redraw on every render call.
- Avoid scrolling: move the cursor to the top-left of the terminal (`cursor::MoveTo(0, 0)`), then overwrite the entire frame in place.
- Because every row is exactly `WIDTH + 2` characters followed by a newline and the frame dimensions are fixed, overwrite produces a stable display without trailing residue.
- Do not implement any animation system, partial updates, or double-buffering.
- Queue all terminal commands with `queue!`, write the frame content as plain text, then `flush()` the output handle.

### 4.6 Renderer Contract

```rust
use std::io::{self, Write};
use crate::game::state::GameState;

pub struct Renderer<W: Write> {
    output: W,
}

impl<W: Write> Renderer<W> {
    pub fn new(output: W) -> Renderer<W>;

    /// Draw the complete frame for the current game state.
    pub fn render(&mut self, state: &GameState) -> io::Result<()>;
}
```

- Generic over `std::io::Write` so tests can use `Vec<u8>`.
- The renderer reads only through the public `GameState` accessors (`snake()`, `food()`, `score()`, `status()`).
- It does not mutate game state and contains no movement, collision, or scoring logic.
- `render` returns an `io::Error` on write failure so the caller can decide how to handle it.

### 4.7 Score Line Behavior

- Always render `Score: {score}` on the row immediately below the bottom border, left-aligned at column 0.
- Use the integer returned by `GameState::score()`.
- The score line is rendered regardless of `GameStatus`.

---

## 5. Input Design (Task 2)

### 5.1 Key Mapping

Only the four arrow keys produce a `Direction`. All other key events are ignored for direction changes.

| Crossterm `KeyCode` | `Direction` |
|---|---|
| `KeyCode::Up` | `Direction::Up` |
| `KeyCode::Down` | `Direction::Down` |
| `KeyCode::Left` | `Direction::Left` |
| `KeyCode::Right` | `Direction::Right` |

Modifiers (Shift, Ctrl, Alt) are ignored for this mapping.

### 5.2 Pure Mapping Function

Expose a pure, testable function with this contract:

```rust
use crossterm::event::KeyEvent;
use crate::game::direction::Direction;

/// Returns the game `Direction` corresponding to an arrow-key event,
/// or `None` for any other key.
pub fn map_key_event_to_direction(event: &KeyEvent) -> Option<Direction>;
```

- Must be deterministic and side-effect free.
- Must return `None` for non-arrow keys, release events, and modifier-only events.
- With `crossterm` 0.28+, only process `KeyEventKind::Press` (or ignore `KeyEventKind::Release`) to avoid duplicate direction changes.

### 5.3 Non-Blocking Polling Contract

Provide an input helper that never blocks the game tick:

```rust
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

/// Drain all keyboard events currently available without waiting.
/// Returns the last arrow-key event seen, if any.
pub fn drain_arrow_event() -> io::Result<Option<KeyEvent>>;
```

- Uses `event::poll(Duration::ZERO)` in a loop until no events remain.
- Returns the **last** arrow-key `KeyEvent` encountered during the drain.
- Non-arrow events are consumed and discarded.
- Never waits for input; if no event is available it returns immediately with `Ok(None)`.

### 5.4 Integration with Domain Direction Rules

- The caller passes the resulting `Direction` to `GameState::change_direction`.
- Immediate reversal rejection remains the responsibility of `GameState::change_direction`; the input layer does not duplicate that logic.
- If multiple arrow keys are pressed between ticks, only the most recent arrow key is applied (last-wins).

---

## 6. Lifecycle Design (Task 7)

### 6.1 Enable Sequence

When the application starts, perform these setup actions in order:

1. `terminal::enable_raw_mode()` — read key presses immediately, disable line buffering.
2. `queue!(output, EnterAlternateScreen)` — use a secondary screen buffer so the original terminal content is preserved.
3. `queue!(output, cursor::Hide)` — hide the blinking cursor during gameplay.
4. `output.flush()`.

### 6.2 Disable Sequence

On application exit (normal, error, or panic where practical), perform cleanup in the reverse order:

1. `queue!(output, cursor::Show)`.
2. `queue!(output, LeaveAlternateScreen)`.
3. `output.flush()`.
4. `terminal::disable_raw_mode()`.

### 6.3 Lifecycle Contract

```rust
use std::io::{self, Write};

pub struct TerminalHandle<W: Write> {
    output: W,
}

impl<W: Write> TerminalHandle<W> {
    /// Enable raw mode, alternate screen, and hidden cursor.
    pub fn enable(output: W) -> io::Result<TerminalHandle<W>>;

    /// Disable raw mode, leave alternate screen, and show cursor.
    pub fn disable(&mut self) -> io::Result<()>;
}

impl<W: Write> Drop for TerminalHandle<W> {
    fn drop(&mut self);
}
```

- `enable` returns a handle that owns the output stream.
- `disable` performs the explicit cleanup sequence and flushes.
- `Drop` calls `disable` as a safety net for panics or early returns.
  - `Drop` must swallow errors from cleanup; it must not panic while unwinding.
- `main` (or the caller) must still call `disable` explicitly on the normal exit path; do not rely solely on `Drop`.

### 6.4 Error-Exit Cleanup

- On any `io::Error` from rendering, input, or lifecycle, attempt cleanup before returning the error.
- If cleanup itself fails, prefer to return the original error.
- On panic, the `Drop` implementation provides best-effort restoration of raw mode, alternate screen, and cursor visibility.

### 6.5 Windows Console Considerations

- Use `crossterm` with its default feature set; it abstracts Windows console specifics.
- No direct WinAPI calls or Windows-only crates are required.
- Raw mode and alternate screen are supported by `crossterm` on Windows 10/11 console and Windows Terminal.

---

## 7. Domain API Integration Contract

The terminal layer interacts with the domain exclusively through the existing Phase 1A API:

- Read board dimensions from `state::WIDTH` and `state::HEIGHT`.
- Render from `GameState` via `snake()`, `food()`, `score()`, and `status()`.
- Apply input via `GameState::change_direction(Direction) -> bool`.
- Trigger start-of-play via `GameState::start_playing()` (caller in later group).
- Detect game-over via `GameState::status() == GameStatus::GameOver` (caller in later group).

No terminal module may:

- Reimplement movement, collision, scoring, growth, or direction-reversal rules.
- Mutate `GameState` except by calling the public methods above.
- Depend on internal domain modules other than `state` and `direction`.

---

## 8. Testability Hooks

- **Renderer:** generic over `std::io::Write`. Unit tests can pass a `Vec<u8>` and assert that the rendered output contains expected border characters, head/body/food glyphs, and the score line.
- **Input:** pure `map_key_event_to_direction` function accepts constructed `crossterm::event::KeyEvent` values, enabling tests for each arrow key and for ignored keys.
- **Lifecycle:** `TerminalHandle` exposes `enable`/`disable` so tests can verify that the guard calls the expected cleanup sequence. Tests must not require a real interactive terminal.
- No automated tests in this group may depend on actual keyboard input or a visible terminal window.

---

## 9. Responsive Behavior

- The game uses a fixed logical board of 40×25 cells.
- No dynamic terminal resizing, centering, or scaling is required.
- Minimum comfortable terminal size: at least 42 columns × 28 rows.
- If the terminal window is smaller, the display may be clipped; no special handling is required for this phase.

---

## 10. Accessibility

- Accessibility considerations for a terminal game are minimal.
- The chosen glyphs (`●`, `■`, `◆`) provide clear visual distinction between head, body, and food.
- No color-dependent information is used.

---

## 11. Performance Budget

- Each render emits O(`WIDTH` × `HEIGHT`) characters, approximately 1,050 cell glyphs plus borders and score line.
- Full redraw at 120 ms is well within terminal throughput limits.
- Avoid scrolling or emitting more bytes than the full frame per tick.
- No animation frames, partial updates, or off-screen buffering.

---

## 12. Constraints Compliance

- **No GUI/engine/framework beyond crossterm:** `crossterm` is used only for raw input, cursor control, screen buffers, and non-blocking polling. It is not a game engine.
- **Keep it simple:** no colors, menus, windows, panels, or advanced terminal UI layouts.
- **Windows console:** `crossterm` default features provide the required Windows console support.
- **Domain isolation:** terminal layer is responsible only for input, rendering, and lifecycle; gameplay rules stay in `src/game/`.

---

## 13. Acceptance Criteria

- [ ] `Renderer` is generic over `std::io::Write` and exposes `new` and `render`.
- [ ] Rendered frame contains correct `+`, `-`, `\|` borders, `WIDTH×HEIGHT` board area, and `Score: N` line.
- [ ] Snake head renders as `●`, body as `■`, food as `◆`, empty cells as space.
- [ ] Redraw strategy uses cursor-home + overwrite and does not scroll.
- [ ] `map_key_event_to_direction` returns the correct `Direction` for each arrow key and `None` for all other keys.
- [ ] Input polling is non-blocking (`event::poll(Duration::ZERO)`) and drains available events.
- [ ] Lifecycle `enable` enters raw mode, alternate screen, and hides the cursor.
- [ ] Lifecycle `disable` / `Drop` restores cursor, leaves alternate screen, and disables raw mode.
- [ ] No domain logic (movement, collision, scoring, reversal rejection) is duplicated in the terminal layer.
- [ ] All new files stay under the 200-line project limit.
