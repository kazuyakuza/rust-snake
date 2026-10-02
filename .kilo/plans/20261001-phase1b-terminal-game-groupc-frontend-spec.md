# Front-end Technical Specification — Phase 1B Group C

## Start Screen, Game-Over Flow, and `main.rs` Wiring

- **Sub-step**: 4.1a of Group C (TODO tasks 5, 6 + the `src/main.rs` wiring that completes the gameplay flow).
- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` (ONLY tasks 5, 6, the `Implementation Constraints`, the `Out of Scope` sections, and the `main.rs` orchestration that connects Groups A/B/C).
- **Global plan**: `.kilo/plans/20261001-phase1b-terminal-game.md`.
- **Frozen upstream decisions**: Group A (`src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`) and Group B (`src/terminal/game_loop.rs`, `src/terminal/input.rs` extension). The domain API in `src/game/state.rs` and `src/game/setup.rs` is the source of truth.
- **Branch**: `feat/phase1b-terminal-game`.
- **Implementer level**: JUNIOR — no architectural latitude; follow this spec exactly.

---

## 1. Scope

This spec covers the **terminal-facing user interaction** that turns the existing terminal primitives and playing loop into a complete, playable flow:

1. The start screen: a simple wait state before `GameState` begins playing.
2. The game-over screen: a simple wait state before the application exits.
3. The `src/main.rs` orchestration that wires setup → terminal enable → start screen → playing loop → game-over screen → cleanup/exit.

It explicitly does **not** cover:

- Game-loop timing, input draining, or rendering (Group B, frozen).
- Terminal primitives (Group A, frozen).
- Domain rules (Phase 1A, frozen).
- Tests/validation (Group D, out of scope for this step).
- Docker/build/packaging (Phase 2, out of scope).
- Restart, pause, menus, colors, difficulty, persistence, sound, config.

---

## 2. Framework & Dependencies

- **Language**: Rust (edition 2021).
- **Terminal crate**: `crossterm = "0.29"` (already added by Group A; no new dependency).
- **Output model**: all screen output writes to `std::io::stdout()` via the `TerminalHandle` created in `main.rs`.
- **Line-ending rule**: in raw mode every written line ends with `\r\n` (CRLF), identical to Group A's `LINE_BREAK` constant. Do not use bare `\n`.

---

## 3. Component Boundaries

No new module files are introduced. The implementation lives in:

| File | Responsibility |
|---|---|
| `src/main.rs` | Application orchestrator + two small private screen helpers + one small private key-wait helper. |
| `src/terminal/lifecycle.rs` | Extended with one public accessor so `main.rs` can write screen text through the same `TerminalHandle` output that the renderer uses. |

All other `src/terminal/**` files remain frozen from Groups A/B.

---

## 4. Required `TerminalHandle` Extension

Add exactly one method to `src/terminal/lifecycle.rs`:

```rust
impl<W: Write> TerminalHandle<W> {
    /// Borrow the wrapped output stream for screen writing.
    pub fn output(&mut self) -> &mut W {
        &mut self.output
    }
}
```

This is the only Group A file modified in Group C. It does not change enable/disable/Drop behavior.

---

## 5. Start-Screen Behavior

### 5.1 Presentation

When the application starts, after `TerminalHandle::enable` succeeds, display a single left-aligned line at the top-left of the alternate screen:

```text
Press any key to start
```

Exact requirements:

- Clear the alternate screen first: `queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;`.
- Write the literal string `Press any key to start` followed by `\r\n`.
- Flush the output.
- No score, no board, no snake movement.

### 5.2 "Any key" definition

"Any key" means **any crossterm key event whose `kind` is `KeyEventKind::Press`**.

- Accept arrow keys, letter keys, digits, Enter, Escape, Space, etc.
- Ignore `KeyEventKind::Release` and `KeyEventKind::Repeat`.
- Ignore non-key events such as `Event::Resize` or mouse events.
- The start key is consumed by the wait helper and is **not** replayed as a direction change.

### 5.3 State transition

After the wait helper returns, `main.rs` must call:

```rust
state.start_playing();
```

This is the **only** production call site for `GameState::start_playing()` in the application. The snake must remain motionless before this call.

---

## 6. Playing Loop Entry

Immediately after `state.start_playing()`:

1. Create a `Renderer` borrowing the same `TerminalHandle` output:
   ```rust
   let mut renderer = Renderer::new(terminal.output());
   ```
2. Call the existing Group B function:
   ```rust
   run_playing_loop(&mut state, &mut renderer)?;
   ```

Precondition: `state.status()` is `Playing` when `run_playing_loop` is called.

---

## 7. Game-Over Flow Behavior

### 7.1 Loop exit

`run_playing_loop` returns when `state.status()` becomes `GameOver`. The final losing board has already been rendered by the loop's last tick.

### 7.2 Presentation

Display a dedicated, text-only game-over screen. Clear the alternate screen and write exactly these five lines at column 0:

```text
GAME OVER

Score: 12

Press any key to exit
```

Where `12` is replaced by `state.score()` at game-over time. Exact requirements:

1. `queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;`
2. Write `GAME OVER\r\n`.
3. Write a blank line: `\r\n`.
4. Write `Score: {score}\r\n` using the final score.
5. Write a blank line: `\r\n`.
6. Write `Press any key to exit\r\n`.
7. Flush.

No board, no snake, no border is redrawn on this screen.

### 7.3 Exit wait

After the game-over screen is flushed, block until the user presses any key, using the same "any key" definition as the start screen (any `KeyEventKind::Press`).

Once a key is pressed, return from `main` with `Ok(())`. Do **not** offer restart, continue, or menu options.

---

## 8. `main.rs` Wiring Order

`src/main.rs` must follow this exact lifecycle:

```text
1. Build initial state from setup::initial_setup().
2. Enable terminal (raw mode + alternate screen + hidden cursor) -> TerminalHandle.
3. Show start screen.
4. Wait for any key press.
5. Call state.start_playing().
6. Create Renderer from TerminalHandle output.
7. Run playing loop.
8. Show game-over screen with final score.
9. Wait for any key press.
10. Return Ok(()); TerminalHandle Drop restores the terminal.
```

### 8.1 Suggested `main` shape

```rust
fn main() -> io::Result<()> {
    let mut state = GameState::new(initial_setup());
    let mut terminal = TerminalHandle::enable(stdout())?;

    show_start_screen(terminal.output())?;
    wait_for_any_key_press()?;
    state.start_playing();

    let mut renderer = Renderer::new(terminal.output());
    run_playing_loop(&mut state, &mut renderer)?;

    show_game_over_screen(terminal.output(), state.score())?;
    wait_for_any_key_press()?;

    Ok(())
}
```

Local helper signatures in `main.rs`:

```rust
fn show_start_screen(output: &mut impl Write) -> io::Result<()> { ... }
fn show_game_over_screen(output: &mut impl Write, score: i32) -> io::Result<()> { ... }
fn wait_for_any_key_press() -> io::Result<()> { ... }
```

### 8.2 Important borrow ordering

- `terminal` must be declared before `renderer` so `renderer` can borrow `terminal.output()`.
- `terminal` remains in scope until the end of `main`, ensuring `Drop` runs and restores the terminal even if an error is returned.
- The game-over screen writes through `terminal.output()` after `run_playing_loop` returns and `renderer` is no longer borrowed.

---

## 9. Wait-for-Key Helper

Implement `wait_for_any_key_press` as a private function in `src/main.rs`:

```rust
fn wait_for_any_key_press() -> io::Result<()> {
    loop {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                return Ok(());
            }
        }
    }
}
```

Behavior:

- Blocks on `event::read()` until a key press occurs.
- Discards releases, repeats, resize events, and mouse events.
- Returns `Ok(())` on the first press; propagates any `io::Error` from `event::read()`.

---

## 10. Error Handling & Cleanup

- `main` returns `io::Result<()>`.
- All terminal I/O errors propagate via `?`.
- **Cleanup guarantee**: `TerminalHandle`'s `Drop` implementation disables raw mode, shows the cursor, and leaves the alternate screen. Because `terminal` is in scope for the entire `main` body, cleanup runs on both normal exit and error/panic unwind.
- No custom error messages, retries, or error screens are required. Rust's default `Result` error printing (after `Drop` restores the terminal) is sufficient for this learning exercise.

---

## 11. Constraints Compliance

The implementation must respect the TODO's `Implementation Constraints` and `Out of Scope` sections:

| Constraint | Compliance |
|---|---|
| No restart system | Game-over screen exits after one key press; no replay path. |
| No menus | Only two simple text screens exist. |
| No pause | Not introduced. |
| No persistent high scores | Final score is displayed only; never saved. |
| No complex terminal UI layouts | Single left-aligned text block per screen. |
| Domain isolation | `main.rs` and screen helpers only call the existing domain API (`initial_setup`, `GameState::new`, `start_playing`, `score`, `status` via the loop). No movement, collision, scoring, or growth logic is duplicated. |

---

## 12. Acceptance Criteria

A successful implementation must satisfy all of the following:

1. `src/main.rs` is no longer empty and contains the orchestration described in §8.
2. The start screen displays exactly `Press any key to start` and the snake does not move before a key is pressed.
3. The first key press transitions the game to `Playing` by calling `GameState::start_playing()`.
4. The playing loop runs with the existing 120 ms tick and existing input/render logic.
5. On game over, the loop stops and a dedicated screen shows `GAME OVER`, a blank line, `Score: N` with the final score, a blank line, and `Press any key to exit`.
6. A key press after the game-over screen exits the application cleanly.
7. The terminal is restored (cursor visible, main screen buffer, raw mode off) on both normal exit and error paths.
8. No new dependencies are added.
9. No gameplay rules are duplicated in `main.rs` or screen helpers.
10. All source files remain ≤ 200 lines (target ≤ 125 code lines), function bodies remain ≤ 50 lines, and the project's coding rules (single-section conditions, private-by-default, self-documenting names, no commented-out code) are respected.

---

## 13. Traceability

| Spec section | TODO task | Requirement |
|---|---|---|
| §5 | Task 5 — Implement the Start Screen | Simple start message; snake not moving; any key starts. |
| §7 | Task 6 — Implement Game Over Flow | Stop loop; show `GAME OVER`, final score, exit prompt; exit on key. |
| §8, §9, §10 | Main wiring | Complete flow from setup through cleanup; clean terminal restoration. |
| §11 | Implementation Constraints | No restart, menus, pause, persistence, duplicated domain logic. |

---

## 14. Notes for Implementers

- Do **not** modify `src/terminal/renderer.rs`, `src/terminal/input.rs` (except as noted in §4), `src/terminal/game_loop.rs`, `src/game/**`, `src/lib.rs`, `Cargo.toml`, or `tests/**`.
- Keep screen helpers private and in `src/main.rs`.
- Use only `queue!` + `flush` for crossterm commands; do not use `execute!`.
- Do not add colors, borders, or animations to the start/game-over screens.
- The exact vertical positioning is not centered; left-aligned at row 0 is the required behavior.
