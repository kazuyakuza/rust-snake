# Implementation Plan — Phase 1B Group A: Terminal Primitives (TODO Tasks 1, 2, 7)

- **Sub-step**: 4.1b of Group A (Tasks 1 + 2 + 7) — produced by the Architector.
- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` (Pattern C; ONLY tasks 1, 2, 7 + "Implementation Constraints" + "Out of Scope" are in scope for this group).
- **Front-end input (mandatory follow)**: `.kilo/plans/20261001-phase1b-terminal-game-groupa-frontend-spec.md` (sections cited below as `SPEC §n`).
- **Global plan**: `.kilo/plans/20261001-phase1b-terminal-game.md`.
- **Branch**: `feat/phase1b-terminal-game` (already created; version already bumped to `0.2.0` — committed `e3c239b`). NO branch creation/switching here. NO version bump here. NO push.
- **Implementer level**: JUNIOR, hard-blocked from scope/architecture decisions; follow this plan exactly; STOP and ask the caller if anything is ambiguous or impossible.
- **Verification mode**: NO local Rust toolchain, NO Docker. There is **no compile/test execution** in this group. Verification = static review only (checklist in §4 Step A8). Authored tests execute in Phase 2 (Docker).

---

## 0. Scope Guardrails (binding)

### 0.1 In scope (this group, sub-step 4.2 implementation)

| TODO task (20261001-todo-3.md) | Deliverable | File(s) |
|---|---|---|
| Task 1 — Terminal Rendering | Full-frame board renderer | `src/terminal/renderer.rs` |
| Task 2 — Terminal Input | Arrow-key → `Direction` mapping + non-blocking drain | `src/terminal/input.rs` |
| Task 7 — Terminal Lifecycle | Raw mode / alternate screen / cursor guard | `src/terminal/lifecycle.rs` |
| Module wiring | Library visibility of the terminal layer | `src/lib.rs`, `src/terminal.rs` |
| Dependency + ignore | crossterm pin, `target/` ignore | `Cargo.toml`, `.gitignore` |
| Structure map | Reflect new files | `.agent/project-structure.md` |

### 0.2 Out of scope (do NOT touch, do NOT create)

- `src/main.rs` (stays `fn main() {}`), `src/terminal/game_loop.rs` (Group B), start screen / game-over screen text (Group C), tests authoring (Group D), `Dockerfile`/`compose.yaml`/`Cargo.lock`/`dist/` (Phase 2), any README commit in 4.2 (README edits belong to docs-specialist at 4.4; content specified in §7 below).
- Reimplementing any domain rule (movement, collision, scoring, growth, reversal rejection) — forbidden by SPEC §7; verified at 4.3 review.
- No crossterm feature flags beyond default; no extra crates; no config systems; no colors.

### 0.3 Files NOT to modify

`src/main.rs`, `src/game.rs`, `src/game/**` (all 8 files), `tests/**` (all 6 files), `README.md` (4.4 only), `.agent/project-info/*` (close-out only), `.kilo/plans/*` (except this plan's own file by Architector), `.gitignore` (only the single line added in A2).

---

## 1. Pre-Analysis — Decisions Encoded (no implementer judgment required)

### 1.1 Dependency choice and version pin (researched at plan time)

- **Decision: add `crossterm = "0.29"` to `[dependencies]`** (semver caret → resolves to `0.29.x`, currently `0.29.0`, published 2025-04-05, not yanked, `rust_version 1.63.0`, edition 2021 — compatible with the project's edition 2021 and the future Docker toolchain).
- Rationale: newest stable line of the de-facto standard terminal crate; `0.28` in SPEC §2 was an example ("e.g."); the global plan defers the final pin to this step (4.1b). Version semantics `"0.29"` covers 0.29.0 and any later 0.29 patch.
- All APIs required by SPEC are present and unchanged in 0.29.0 (verified against docs.rs):
  - `crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}` — Windows console supported via the crate's default `windows` feature (on by default; no direct WinAPI calls anywhere in this plan).
  - `crossterm::cursor::{Hide, Show, MoveTo}` (u16 col,row).
  - `crossterm::event::{poll, read, Event, KeyEvent, KeyEventKind, KeyCode}`. Signature pinned from docs.rs 0.29.0: **`pub fn poll(timeout: Duration) -> Result<bool>` (by value, NOT `&Duration`)** and `pub fn read() -> Result<Event>`; `Ok(true)` guarantees the next `read()` does not block.
  - `crossterm::{execute!, queue!}` macros, `crossterm::style::Print` available (Print not needed; plain `std::io::Write` via `write!` is used for frame text — SPEC §4.5 allows "write the frame content as plain text").
- Uses **default features only** (which include `windows`, `events`, `bracketed-paste`, `derive-more`). Do NOT add `event-stream`, `serde`, `filedescriptor`, or `osc52`.
- KE201 note (anchors SPEC §5.2): on Windows console, key events are `Press` or `Release` only (`Repeat` never occurs); filters must therefore key off `KeyEventKind::Press`.

### 1.2 Module layout decision

- **Decision: use `src/terminal.rs` as the module root** (NOT `src/terminal/mod.rs`). SPEC §3 shows a `mod.rs` sketch, but the assigned-scope prompt, the global plan, and repo precedent (`src/game.rs` + `src/game/*`) use the modern 2018-style layout. One decision, one implementation.
- Resulting tree (all under the `lib` crate so pure parts stay test-reachable from `tests/`):

```text
src/
├── lib.rs            MODIFY: add `pub mod terminal;`
├── main.rs           UNCHANGED (fn main() {})
├── terminal.rs       NEW: module root; declares submodules
└── terminal/
    ├── renderer.rs   NEW: Task 1
    ├── input.rs      NEW: Task 2
    └── lifecycle.rs  NEW: Task 7
```

- Line budgets (rule ≤200/file, ideal ≤125 code lines): `terminal.rs` ≤ 10, `renderer.rs` ≤ 110, `input.rs` ≤ 50, `lifecycle.rs` ≤ 75. If an implementation draft exceeds a budget, the implementer STOPs and reports — do not split files without approval.

### 1.3 Behavioral decisions (picking one of each, per junior restriction)

| # | Question | Decision |
|---|---|---|
| D1 | Newlines inside raw mode | Rows end with `\r\n` (CRLF). In raw mode a bare `\n` moves down without returning to column 0. Constant `LINE_BREAK: &str = "\r\n"`. |
| D2 | Draw mechanism | `queue!(self.output, cursor::MoveTo(0, 0))` once, then plain `write!` per row (io::Write), then one `flush()`. No `Clear`, no double-buffering, no partial updates (SPEC §4.5). |
| D3 | Cell precedence | head > body > food > empty via three single-condition guards in `cell_glyph` (SPEC §4.4 order). |
| D4 | Release/Repeat events | Filtered out once at input boundary with `is_key_press` helper (SPEC §5.2); applies to both mapping and drain. |
| D5 | Multiple arrow keys per tick | Drain returns the LAST arrow Press seen (last-wins, SPEC §5.4); caller (Group B) applies it via `GameState::change_direction`. |
| D6 | `TerminalHandle::enable` shape | Associated function returning `io::Result<TerminalHandle<W>>` exactly as SPEC §6.3. |
| D7 | Enable failure rollback | If raw mode succeeds but `queue!`/`flush` fails, best-effort `let _ = terminal::disable_raw_mode();` before returning the error (SPEC §6.4). |
| D8 | `disable` error precedence | Always attempt both restore-commands and `disable_raw_mode`; report the restore error first if any (helper `report_first_error`), else the raw-mode result. `Drop` swallows errors (`let _ = self.disable();`). |
| D9 | Renderer status independence | Score line always rendered; renderer never branches on `GameStatus` (SPEC §4.7); status screens come in Groups B/C. |
| D10 | Glyph storage | Private `const` `char`s in `renderer.rs` (NOT a public API; SPEC calls them implementation detail). |
| D11 | `Position` construction in renderer | Build `Position { x, y }` inline per cell (fields are `pub`; `Position` is `Copy`); no renderer-side bounds math — uses `WIDTH`/`HEIGHT` consts from `game::state`. |
| D12 | Borrow flow | Helpers that only read state take `&GameState` (+ `self`); methods that write output take `&mut self`; no mixing in one call chain that would fight the borrow checker. |

### 1.4 Domain API contract (verified in-tree, DO NOT redesign)

Consumed from `src/game/state.rs` (already implemented): `WIDTH: i32 = 40`, `HEIGHT: i32 = 25` (cell counts), `GameState::{snake() -> &Snake, food() -> &Food, score() -> i32, status() -> GameStatus, change_direction(Direction) -> bool}`, and from `src/game/food.rs`: `Food::position() -> Position` / `Food::occupies(Position) -> bool`; from `src/game/snake.rs`: `Snake::head() -> Position` (Copy), `Snake::segments() -> &[Position]`; `src/game/position.rs`: `Position { pub x: i32, pub y: i32 }` (`PartialEq`, `Copy`). Input consumes `crate::game::direction::Direction::{Up, Down, Left, Right}`. `change_direction` already rejects immediate reversals — the input layer MUST NOT duplicate that (SPEC §5.4).

---

## 2. Target State — File-by-File Structure

Every item below is a hard requirement; signatures are frozen (they match SPEC §4.6, §5.2, §5.3, §6.3); the implementer writes the bodies following the snippets in §3.

### 2.1 `Cargo.toml` (modify)

After the change:

```toml
[package]
name = "snake"
version = "0.2.0"
edition = "2021"

[dependencies]
rand = "0.8"
crossterm = "0.29"
```

### 2.2 `.gitignore` (modify)

Append a new section after `# Build artifacts (generic)` block lines:

```gitignore

# Rust build artifacts
target/
```

(Do not reuse the generic `# Build artifacts (generic)` heading; add the two-line block as its own section. `dist/` is already ignored.)

### 2.3 `src/lib.rs` (modify)

Final content (module doc line updated to stay accurate):

```rust
//! Library crate root: exposes the `game` and `terminal` modules to both the binary and the integration tests under `tests/`.

pub mod game;
pub mod terminal;
```

### 2.4 `src/terminal.rs` (new — module root)

```rust
//! Terminal UI layer: rendering, arrow-key input, and terminal lifecycle.
//! Gameplay rules remain in `game`; this layer only renders state, maps keys,
//! and toggles raw mode / alternate screen / cursor visibility.

pub mod input;
pub mod lifecycle;
pub mod renderer;
```

(Submodules listed alphabetically, matching `src/game.rs` style.)

### 2.5 `src/terminal/renderer.rs` (new — TODO Task 1)

| Item | Specification |
|---|---|
| Imports | `use std::io::{self, Write};` `use crossterm::{cursor::MoveTo, queue};` `use crate::game::position::Position;` `use crate::game::state::{GameState, HEIGHT, WIDTH};` |
| Constants (private) | `const HEAD_GLYPH: char = '●';` (U+25CF) · `const BODY_GLYPH: char = '■';` (U+25A0) · `const FOOD_GLYPH: char = '◆';` (U+25C6) · `const EMPTY_GLYPH: char = ' ';` · `const CORNER_GLYPH: char = '+';` · `const HORIZONTAL_GLYPH: char = '-';` · `const VERTICAL_GLYPH: char = '|';` · `const SCORE_PREFIX: &str = "Score: ";` · `const LINE_BREAK: &str = "\r\n";` |
| Type | `pub struct Renderer<W: Write> { output: W }` — field private (prefer-private rule) |
| Public API | `pub fn new(output: W) -> Renderer<W>` · `pub fn render(&mut self, state: &GameState) -> io::Result<()>` |
| Private helpers | `fn write_top_border(&mut self) -> io::Result<()>` · `fn write_bottom_border(&mut self) -> io::Result<()>` · `fn write_board_rows(&mut self, state: &GameState) -> io::Result<()>` · `fn write_score_line(&mut self, state: &GameState) -> io::Result<()>` · `fn write_line(&mut self, content: &str) -> io::Result<()>` · `fn render_board_row(&self, state: &GameState, row: i32) -> String` · `fn cell_glyph(&self, state: &GameState, cell: Position) -> char` |
| Frame size | border rows `HEIGHT + 2`, board rows `HEIGHT`, score row 1 → total `HEIGHT + 3` strings, each `WIDTH + 2` chars + CRLF (SPEC §4.2, §4.3) |

### 2.6 `src/terminal/input.rs` (new — TODO Task 2)

| Item | Specification |
|---|---|
| Imports | `use std::io::{self};` `use std::time::Duration;` `use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};` `use crate::game::direction::Direction;` |
| Pure mapping (public) | `pub fn map_key_event_to_direction(event: &KeyEvent) -> Option<Direction>` — `None` unless `is_key_press(event)`; then match `event.code`: `Up→Direction::Up`, `Down→Direction::Down`, `Left→Direction::Left`, `Right→Direction::Right`, `_→None` (SPEC §5.1, §5.2) |
| Press filter (private) | `fn is_key_press(event: &KeyEvent) -> bool { event.kind == KeyEventKind::Press }` (keeps single-section booleans, encodes decision D4) |
| Non-blocking drain (public) | `pub fn drain_arrow_event() -> io::Result<Option<KeyEvent>>` — loop `while event::poll(Duration::ZERO)?` reading events (DECISION: `Duration::ZERO` is passed BY VALUE, 0.29 signature), tracking the last arrow-key Press (SPEC §5.3) |
| Event classifier (private) | `fn pressed_arrow_key(event: Event) -> Option<KeyEvent>` — returns the key event only when it is a `Press` of an arrow key (uses `is_key_press` + `map_key_event_to_direction` to stay single-source-of-truth on "what is an arrow key") |
| No struct | Free functions only; no state to own. Reversal rejection lives in `GameState::change_direction` (caller is Group B) |

### 2.7 `src/terminal/lifecycle.rs` (new — TODO Task 7)

| Item | Specification |
|---|---|
| Imports | `use std::io::{self, Write};` `use crossterm::{cursor, queue, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};` |
| Type | `pub struct TerminalHandle<W: Write> { output: W }` — field private |
| Enable (public, associated) | `pub fn enable(mut output: W) -> io::Result<TerminalHandle<W>>` — sequence per SPEC §6.1: `terminal::enable_raw_mode()?` → `queue!(output, EnterAlternateScreen, cursor::Hide)` → `output.flush()?` → `Ok(TerminalHandle { output })`; if queue! fails: `let _ = terminal::disable_raw_mode();` then return the error (DECISION D7 rollback) |
| Disable (public method) | `pub fn disable(&mut self) -> io::Result<()>` — reverse order per SPEC §6.2 with error-precedence DECISION D8: run `write_restore_commands()`, then `terminal::disable_raw_mode()`, then `report_first_error` |
| Private helpers | `fn write_restore_commands(&mut self) -> io::Result<()>` (`queue!(self.output, cursor::Show, LeaveAlternateScreen)?` then `self.output.flush()`) · `fn report_first_error(restore_result: io::Result<()>, raw_mode_result: io::Result<()>) -> io::Result<()>` (returns restore error if present, else raw-mode result) |
| Drop guard | `impl<W: Write> Drop for TerminalHandle<W> { fn drop(&mut self) { let _ = self.disable(); } }` — swallows all errors; must never panic while unwinding (SPEC §6.3) |

---

## 3. Code Sketches for Tricky Parts (encodes all decisions; the implementer may adjust local variable names only)

### 3.1 `renderer.rs` — frame draw loop (`render` + row builders)

```rust
    /// Draw the complete frame for `state` at the home position, without scrolling.
    pub fn render(&mut self, state: &GameState) -> io::Result<()> {
        queue!(self.output, MoveTo(0, 0))?;
        self.write_top_border()?;
        self.write_board_rows(state)?;
        self.write_bottom_border()?;
        self.write_score_line(state)?;
        self.output.flush()
    }

    fn write_board_rows(&mut self, state: &GameState) -> io::Result<()> {
        for row in 0..HEIGHT {
            let row_text = self.render_board_row(state, row);
            self.write_line(&row_text)?;
        }
        Ok(())
    }

    fn render_board_row(&self, state: &GameState, row: i32) -> String {
        let mut row_text = String::new();
        row_text.push(VERTICAL_GLYPH);
        for column in 0..WIDTH {
            row_text.push(self.cell_glyph(state, Position { x: column, y: row }));
        }
        row_text.push(VERTICAL_GLYPH);
        row_text
    }

    fn cell_glyph(&self, state: &GameState, cell: Position) -> char {
        if self.is_snake_head(state, cell) {
            return HEAD_GLYPH;
        }
        if self.is_snake_body(state, cell) {
            return BODY_GLYPH;
        }
        if self.is_food(state, cell) {
            return FOOD_GLYPH;
        }
        EMPTY_GLYPH
    }

    fn is_snake_head(&self, state: &GameState, cell: Position) -> bool {
        state.snake().head() == cell
    }

    fn is_snake_body(&self, state: &GameState, cell: Position) -> bool {
        state.snake().segments().contains(&cell)
    }

    fn is_food(&self, state: &GameState, cell: Position) -> bool {
        state.food().occupies(cell)
    }
```

- Border writer shared shape (both borders):

```rust
    fn write_top_border(&mut self) -> io::Result<()> {
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }
```

- Free function (no params):

```rust
fn rendered_border_row() -> String {
    let mut border_row = String::new();
    border_row.push(CORNER_GLYPH);
    for _ in 0..WIDTH {
        border_row.push(HORIZONTAL_GLYPH);
    }
    border_row.push(CORNER_GLYPH);
    border_row
}
```

- `write_line` (appends CRLF, constant bytes per frame):

```rust
    fn write_line(&mut self, content: &str) -> io::Result<()> {
        write!(self.output, "{content}{LINE_BREAK}")
    }
```

- `write_score_line`:

```rust
    fn write_score_line(&mut self, state: &GameState) -> io::Result<()> {
        let score_text = format!("{SCORE_PREFIX}{}", state.score());
        self.write_line(&score_text)
    }
```

Rule checks (must hold in the final code): every `if` is a single call (`is_snake_head(...)` etc.); no nested loop-in-if; bodies ≤ 50 lines; ≤ 2 non-self params per function.

### 3.2 `input.rs` — pure mapping + non-blocking drain

```rust
/// Returns the game direction of an arrow-key press, or `None` for any
/// other key, release, or repeat event.
pub fn map_key_event_to_direction(event: &KeyEvent) -> Option<Direction> {
    if !is_key_press(event) {
        return None;
    }
    match event.code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

fn is_key_press(event: &KeyEvent) -> bool {
    event.kind == KeyEventKind::Press
}

/// Drain all currently available input events without waiting, and return the
/// last arrow-key press seen, if any. Never blocks: when no event is pending
/// the zero-duration poll ends the loop immediately.
pub fn drain_arrow_event() -> io::Result<Option<KeyEvent>> {
    let mut last_arrow_press = None;
    while event::poll(Duration::ZERO)? {
        if let Some(arrow_press) = pressed_arrow_key(event::read()?) {
            last_arrow_press = Some(arrow_press);
        }
    }
    Ok(last_arrow_press)
}

fn pressed_arrow_key(event: Event) -> Option<KeyEvent> {
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    if is_key_press(&key_event) && is_arrow_key(&key_event) {
        return Some(key_event);
    }
    None
}

fn is_arrow_key(event: &KeyEvent) -> bool {
    map_key_event_to_direction(event).is_some()
}
```

- FIx from review note: the compound condition in `pressed_arrow_key` breaks the single-section boolean rule — the FINAL code must instead be:

```rust
fn pressed_arrow_key(event: Event) -> Option<KeyEvent> {
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    if !is_arrow_key_press(&key_event) {
        return None;
    }
    Some(key_event)
}

fn is_arrow_key_press(event: &KeyEvent) -> bool {
    is_key_press(event) && is_arrow_key(event)
}
```
(composing helpers in a named predicate keeps the `if` single-section). Use THIS version, not the first one.
- `Duration::ZERO` is passed by value: `event::poll(Duration::ZERO)?` — confirmed signature `pub fn poll(timeout: Duration) -> Result<bool>` (docs.rs 0.29.0).
- `game_loop.rs` (Group B) will reuse `drain_arrow_event` with its own timed poll — no coupling added here.
- Note for Group D (do not implement here): this function reads the real terminal, so it is NOT headless-testable; the mapping function IS (`KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)` builds a `Press` event).

### 3.3 `lifecycle.rs` — enable sequence, disable sequence, Drop guard

```rust
/// Guard that owns the terminal output stream and restores the terminal state:
/// raw mode off, main screen buffer back, cursor visible.
pub struct TerminalHandle<W: Write> {
    output: W,
}

impl<W: Write> TerminalHandle<W> {
    /// Enable raw mode, switch to the alternate screen, and hide the cursor.
    pub fn enable(mut output: W) -> io::Result<TerminalHandle<W>> {
        terminal::enable_raw_mode()?;
        if let Err(setup_error) = write_setup_commands(&mut output) {
            let _ = terminal::disable_raw_mode();
            return Err(setup_error);
        }
        Ok(TerminalHandle { output })
    }

    /// Restore the terminal: cursor visible, main screen buffer, raw mode off.
    pub fn disable(&mut self) -> io::Result<()> {
        let restore_result = self.write_restore_commands();
        let raw_mode_result = terminal::disable_raw_mode();
        report_first_error(restore_result, raw_mode_result)
    }

    fn write_restore_commands(&mut self) -> io::Result<()> {
        queue!(self.output, cursor::Show, LeaveAlternateScreen)?;
        self.output.flush()
    }
}

impl<W: Write> Drop for TerminalHandle<W> {
    fn drop(&mut self) {
        let _ = self.disable();
    }
}

fn write_setup_commands(output: &mut impl Write) -> io::Result<()> {
    queue!(output, EnterAlternateScreen, cursor::Hide)?;
    output.flush()
}

fn report_first_error(restore_result: io::Result<()>, raw_mode_result: io::Result<()>) -> io::Result<()> {
    match restore_result {
        Err(error) => Err(error),
        Ok(()) => raw_mode_result,
    }
}
```

- The private `write_setup_commands(&mut impl Write)` keeps `enable` short and lets the rollback error path stay one level deep.
- `report_first_error` prefers the restore-command error; `disable_raw_mode()` is ALWAYS attempted (decisions D7/D8). No `?` after `enable_raw_mode` success other than the single early return shown; no branching on two conditions anywhere in the file.

---

## 4. Step-by-Step Implementation (execute in order; every step ends in a verification)

Legend: each numbered step is atomic and verifiable. Line counts quoted are budget ceilings; the implementer does not get to exceed them.

### Step A0 — Preconditions check (no file changes)

1. Run `git status` and `git branch --show-current`.
2. Verify: branch is `feat/phase1b-terminal-game`; `Cargo.toml` reports version `0.2.0`; untracked files are exactly `.kilo/plans/20261001-phase1b-terminal-game.md` and `.kilo/plans/20261001-phase1b-terminal-game-groupa-frontend-spec.md` (plus this plan file).
3. STOP and ask the caller if any condition fails.
4. NO commit in this step.

### Step A1 — Commit the workflow plan documents

1. Stage only the plan documents: `git add .kilo/plans/20261001-phase1b-terminal-game.md .kilo/plans/20261001-phase1b-terminal-game-groupa-frontend-spec.md .kilo/plans/20261001-phase1b-terminal-game-groupa.md`.
2. Commit: `git commit -m "docs: add phase 1B global plan and group A terminal spec and implementation plan"`.
3. Verify `git status` shows a clean tree before continuing.

### Step A2 — Dependency + gitignore (TODO Task 7 prerequisite; environment hygiene)

1. Edit `Cargo.toml`: in `[dependencies]`, after the `rand = "0.8"` line, add `crossterm = "0.29"`. Exact file content per §2.1. No other sections, no feature list.
2. Edit `.gitignore`: append the `# Rust build artifacts` block with `target/` per §2.2.
3. Static check: `git diff --stat` is limited to `Cargo.toml` and `.gitignore`.
4. Stage + commit: `git commit -am "chore: add crossterm dependency and ignore cargo target dir"`.

### Step A3 — Module wiring

1. Edit `src/lib.rs` to the exact content of §2.3 (add `pub mod terminal;`, update the module doc line).
2. Create `src/terminal.rs` with the exact content of §2.4.
3. Static check: `git diff --stat` shows exactly `src/lib.rs` modified and `src/terminal.rs` added; `src/terminal/` does not exist yet (compile would fail until Step A4 — acceptable; do NOT create placeholder files, do NOT run any build command).
4. Stage + commit: `git add src/lib.rs src/terminal.rs` then `git commit -m "feat: expose terminal module in library root"`.

(DO NOT commit A3 and A4 together — separate review units.)

### Step A4 — Renderer (TODO Task 1)

1. Create `src/terminal/renderer.rs` implementing §2.5 exactly, with the code shape of §3.1.
2. Self-check against SPEC §4 items BEFORE staging: glyphs exact (`'●'`, `'■'`, `'◆'`, `' '`, `'+'`, `'-'`, `'|'` — verify by copying from this plan, files are UTF-8); frame = `HEIGHT + 3` lines × `WIDTH + 2` columns; coordinate offset via inline `Position { x: column, y: row }`; precedence head→body→food→empty; score line left-aligned right below the bottom border; `MoveTo(0, 0)` then overwrite then single `flush()`; CRLF line endings; no blank first line (do NOT insert a newline BEFORE the top border on later redraws).
3. Static rule check: imports list shorter than the §2.5 imports table (nothing extra — e.g., no `Clear`, no `GameStatus` import); no `pub` besides `Renderer`, `new`, `render`; file ≤ 200 lines (expected ~100).
4. Stage + commit: `git add src/terminal/renderer.rs` then `git commit -m "feat: add terminal board renderer"`.

### Step A5 — Input (TODO Task 2)

1. Create `src/terminal/input.rs` implementing §2.6 exactly, with the code shape of §3.2 (**use the corrected `pressed_arrow_key` version** noted there).
2. Self-check against SPEC §5: mapping is total over `KeyCode` via `_ => None`; only `KeyEventKind::Press` passes; modifiers ignored (the mapping never reads `event.modifiers`); drain never blocks (`poll(Duration::ZERO)`), drains ALL pending events, returns the last arrow press; no reversal logic anywhere in the file; imports exactly per §2.6.
3. Static rule check: two public functions only; both private helpers ≤ 2 params; `drain_arrow_event` body ≤ 8 lines; nesting depth of `drain_arrow_event` = 2 (`while` + `if let`) — max allowed; file ≤ 60 lines.
4. Stage + commit: `git add src/terminal/input.rs` then `git commit -m "feat: add arrow key input mapping and non blocking drain"`.

### Step A6 — Lifecycle (TODO Task 7)

1. Create `src/terminal/lifecycle.rs` implementing §2.7 exactly, with the code shape of §3.3.
2. Self-check against SPEC §6: enable order = raw mode → EnterAlternateScreen → cursor::Hide → flush; disable order = cursor::Show → LeaveAlternateScreen → flush → raw mode off; Drop calls `disable` and swallows errors; no panic path in Drop; no `execute!` used (only `queue!` + flush, per SPEC §6.1–6.2 which queue then flush).
3. Static rule check: `TerminalHandle` field private; public API is exactly `enable` and `disable`; `Drop::drop` body exactly one remove-semicolon line `let _ = self.disable();`; file ≤ 80 lines.
4. Stage + commit: `git add src/terminal/lifecycle.rs` then `git commit -m "feat: add terminal lifecycle guard"`.

### Step A7 — Project structure map (structure maintenance workflow)

1. Edit `.agent/project-structure.md` preserving its existing format and sections:
   - Under `# Folders in src/`, after the `src/game/` line, add: `- src/terminal/ - terminal UI module tree (renderer, input, lifecycle)`
   - Under `# Rust project files`, update the `Cargo.toml` comment to `- Cargo.toml - Cargo package manifest (package snake, dependencies rand and crossterm)` and update the `src/lib.rs` line to `- src/lib.rs - library crate root; exposes the game and terminal modules to the binary and the integration tests`.
   - In `# Rust project files` after the `src/game/food_placement.rs` line, add:
     - `- src/terminal.rs - terminal module root; declares the terminal/* submodules`
     - `- src/terminal/renderer.rs - full-frame board renderer over an io::Write output (borders, snake, food, score line)`
     - `- src/terminal/input.rs - arrow key press to game Direction mapping plus non blocking event drain`
     - `- src/terminal/lifecycle.rs - raw mode, alternate screen and cursor visibility with Drop guard cleanup`
   - Leave every other line untouched (the integration-tests section, folders, other folders).
2. Static check: no other file changed (`git status`).
3. Stage + commit: `git add .agent/project-structure.md` then `git commit -m "docs: add terminal module to project structure"`.

### Step A7b — README status note (analysis-only, one sentence; full README rewrite happens at 4.4)

1. Edit `README.md` "Project Structure" bullet list: after the `src/game/collision.rs`, `src/game/food_placement.rs` bullet, add one bullet:
   `- \`src/terminal.rs\`, \`src/terminal/renderer.rs\`, \`src/terminal/input.rs\`, \`src/terminal/lifecycle.rs\`: terminal layer primitives (board rendering, arrow-key input, terminal lifecycle) — interactive wiring arrives in the next group.`
2. Nothing else in README (no section rewrites — 4.4 owns those).
3. Stage + commit: `git add README.md` then `git commit -m "docs: mention terminal primitives in readme structure"`.

### Step A8 — Static verification (no compiler available; run in order, fix anything failing)

1. `git log --oneline -10` — verify the 8 commits A1–A7b exist, oldest to newest matching §6 list order messages.
2. `git diff --stat HEAD~8 HEAD` — verify ONLY these files changed: `Cargo.toml`, `.gitignore`, `src/lib.rs`, `src/terminal.rs`, `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `.agent/project-structure.md`, `README.md` (+ the plan `.md` files from A1).
3. Line-count audit (rule ≤200/file): count lines of each new file; STOP if `renderer.rs` > 200, `input.rs` > 60, `lifecycle.rs` > 80, `terminal.rs` > 12.
4. Naming audit: every public item name appears in this plan or in the existing codebase style (`Renderer`, `TerminalHandle`, `map_key_event_to_direction`, `drain_arrow_event`, `is_key_press`, `is_arrow_key`, `is_arrow_key_press`, `pressed_arrow_key`, `write_restore_commands`, `report_first_error`, `rendered_border_row`).
5. Forbidden-content audit (grep each pattern across `src/terminal/`): `collides|collision|score +=|SCORE_INCREMENT|opposite()\(|start_playing|advance_one_step|use crate::game::collision` → all zero hits (domain isolation, SPEC §7); `println!|eprintln!|dbg!` → zero hits (no debug leftovers); `execute!` → zero hits in new files (only `queue!` per SPEC §4.5/§6.2); `unsafe` → zero hits.
6. Cross-file module graph audit: `src/lib.rs` names `terminal`; `src/terminal.rs` names `input`, `lifecycle`, `renderer`; each file imports only paths present in §2 imports tables; no file imports `crate::terminal::*` back into itself.
7. Rule-compliance matrix (record PASS/FAIL per file): max 200 lines/file · ≤125 code lines ideal · function bodies ≤50 lines · ≤2 non-self params · ≤2 nesting depth · single-section boolean conditions · private members by default · self-documenting names · no commented-out code · minimal comments (module `//!` + `///` on public items only).
8. Glyph audit: copy-paste `HEAD_GLYPH`/`BODY_GLYPH`/`FOOD_GLYPH` values out of the file and diff against `●` `■` `◆`; check for accidental full-width characters (`｜`, `＋`, `－` are FORBIDDEN — only ASCII `|`, `+`, `-`).
9. Contract audit vs SPEC §13 acceptance list: tick each checkbox mentally and record the result in the step summary returned to the caller (all nine boxes must be satisfiable statically).
10. Any FAIL: fix the smallest possible edit, re-run A8 fully, amend nothing (new commit `fix: align terminal primitives with group A plan`); never expand scope during fixes.

---

## 5. Mapping to TODO Tasks (traceability)

| Plan artifact | TODO task | TODO requirement(s) covered | SPEC section |
|---|---|---|---|
| `src/terminal/renderer.rs` | Task 1 — Implement Terminal Rendering | boundaries / snake / food / score display; `WIDTH=40`, `HEIGHT=25` logical dims; simple characters; head distinguishable from body; redraw (no animation); rendering separate from domain | SPEC §4 (all), §9, §11 |
| `src/terminal/input.rs` | Task 2 — Implement Terminal Input | four arrow keys → `Direction`; integrates with `change_direction` (reversal NOT duplicated); input handling separate from core logic | SPEC §5 (all) |
| `src/terminal/lifecycle.rs` | Task 7 — Handle Terminal Lifecycle | terminal configured for input/rendering; restored on exit (cursor visible, no raw mode, no alternate screen); cleanup on error paths (best-effort, Drop guard); no large abstraction layer | SPEC §6 (all) |
| Steps A2–A3 | Implementation Constraints | single new dependency; simple, no engine/framework beyond crossterm | SPEC §2, §12 |
| All steps | Out of Scope | no Docker/Windows build work started in this group | global plan step list |

Explicitly NOT covered here (deferred): Task 3 (game loop 120 ms), Task 4 (movement/input/state wiring), Task 5 (start screen), Task 6 (game over flow), Task 8 (render refinement), Task 9 (flow validation), Task 10 (tests) — Groups B/C/D.

---

## 6. Commit Summary (expected end state of 4.2)

```text
* docs: mention terminal primitives in readme structure
* docs: add terminal module to project structure
* feat: add terminal lifecycle guard
* feat: add arrow key input mapping and non blocking drain
* feat: add terminal board renderer
* feat: expose terminal module in library root
* chore: add crossterm dependency and ignore cargo target dir
* docs: add phase 1B global plan and group A terminal spec and implementation plan
```
(old to new; commit A1 lists all three plan documents — global plan, frontend spec, and this implementation plan). No pushes; no merges.

---

## 7. Documentation Guidance for Sub-step 4.4 (docs-specialist; content frozen here)

These edits are performed at 4.4 by docs-specialist, using exactly this content guidance (do not execute at 4.2 beyond Step A7b):

1. **README `## About this Project`**: replace the sentence "…are implemented; a Docker-based Windows build…" with a version noting that the core game model AND the terminal primitives (renderer, arrow-key input, lifecycle guard) are implemented, while the interactive game loop and start/game-over screens are still upcoming (Groups B–C) and the Docker build remains Phase 2.
2. **README `## Game Rules & Controls`**: append one bullet after the "Board:" bullet — `Visuals: snake head ●, body ■, food ◆, empty space; ASCII border of +, -, |; score line below the board.` and one bullet after "Controls:" — `Input: arrow keys only via non-blocking terminal polling; only key presses act (releases/repeats ignored); with several arrow keys per tick the most recent one wins.`
3. **README new `## Terminal UI` section** (inserted between `Game Rules & Controls` and `Build & Run`; add to Table of Contents):
   - Renderer paragraph: full-frame redraw at 42×28 columns/rows including the ASCII border plus `Score: N` line; redraw strategy = cursor home + overwrite (no scrolling); fixed 40×25 logical board from the domain.
   - Input paragraph: crossterm key events drained without blocking the game tick; the pure mapping is `snake::terminal::input::map_key_event_to_direction`; self-reversal is rejected by the domain (`GameState::change_direction`).
   - Lifecycle paragraph: raw mode + alternate screen + hidden cursor are enabled at startup and always restored on exit, including panic best-effort via the `Drop` guard (`TerminalHandle`).
   - Validation note: interactive validation is manual and deferred until Phase 2 produces `dist/snake.exe` (no local toolchain; `cargo test` also waits for Docker).
4. **README `## Project Structure`**: add the four bullet lines for `src/terminal.rs` and the three submodules (mirroring Step A7b wording once Group B/C names exist, do not mention game_loop.rs yet).
5. **`.agent/project-structure.md`**: already updated in Step A7; at 4.4 do NOT re-edit it.

---

## 8. Front-end Spec Acceptance Cross-Reference (for 4.5a reviewer)

| SPEC §13 criterion | Where satisfied |
|---|---|
| `Renderer` generic over `io::Write`, `new` + `render` | §2.5, §3.1 |
| Correct borders, `WIDTH×HEIGHT` area, `Score: N` line | §2.5 table, §3.1 |
| `●`/`■`/`◆`/space glyphs | §2.5 constants |
| Cursor-home + overwrite redraw, no scroll | §2.5 `render` (decision D2) |
| `map_key_event_to_direction` correct for arrows, `None` otherwise | §2.6, §3.2 |
| Non-blocking `poll(Duration::ZERO)` drain | §2.6, §3.2 |
| Lifecycle enable = raw mode + alt screen + hidden cursor | §2.7, §3.3 |
| Disable/Drop restores cursor + main screen + raw mode | §2.7, §3.3 |
| No duplicated domain logic | §0.2, §4 A8 audit 5 |
| ≤200-line files | §2 line budgets, §4 A8 audit 3 |

---

## 9. Notes for Reviewers (sub-steps 4.3 / 4.5a / 4.5b)

- Review units are commits A3–A6 (one per module) — diff each commit in isolation; A7/A7b are docs-only.
- The renderer must compile as a library module without warnings about unused imports; if `w` uses `GameStatus`, flag (renderer must not branch on status).
- `drain_arrow_event` semantics on error: any `io::Error` from `poll`/`read` propagates immediately via `?` — the last-seen arrow press is then lost, which is acceptable (SPEC does not require retry).
- `TerminalHandle::drop` during panic unwind is best-effort by design (SPEC §6.4); reviewers must NOT demand `panic::set_hook`/`catch_unwind` features (out of scope, keep-it-simple).
- If `crossterm = "0.29"` resolves to a version with a different API surface at future build time, treat THAT as a build-time blocker, not a plan deviation (record in the adherence report).
