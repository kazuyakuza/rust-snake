# Implementation Plan — 20261005 Half-Block Rendering Fix (Task 1, step 4.1b)

**TODO:** `.agent/todos/20261005/20261005-todo-2.md` (single task)
**Front-end spec (FINAL — every decision in it is binding):** `.kilo/plans/20261005-half-block-frontend-spec.md`
**Global plan:** `.kilo/plans/20261005-half-block-rendering-fix.md`
**Branch:** `fix/half-block-rendering` (already created in step 2 — do NOT create/switch branches)
**Version:** already `0.3.2` (step 3 done — do NOT touch `Cargo.toml`)
**Git push:** restricted to step 5 — no pushing in this task cycle.

---

## 0. High-Level Approach

Replace the double-width row renderer with a half-block packed renderer:

- One terminal row now displays **two logical rows** (upper half = `2 * packed_row`, lower half = `2 * packed_row + 1`) using `▀` / `▄` / `█` glyphs and crossterm `SetColors` colors (head Yellow, body Green, food Red, empty default).
- Board width returns to **1 terminal column per logical cell** → border rows `+` + `-`×80 + `+` (82 chars), single `|` verticals.
- Frame: 1 border + 40 packed rows + 1 border + 1 score line = **43 rows × 82 cols**.
- Emission: `queue!(out, SetColors(Colors::new(fg, bg)), Print(glyph))` per board cell; `ResetColor` once per packed row (after last cell, before trailing `|`), once before each border row, once before the score line, and once at the very end before `flush`.
- Tests: re-specify the 3 renderer snapshot tests, add 2 new ones (vertical packing, empty-cell default color). Headless `Vec<u8>` only.
- Docs (4.4): README size note, `docs/terminal-ui.md`, `architecture.md`, `tech.md` glyph/color updates.

**Factual verification already done (planner-level, do not re-litigate):** crossterm 0.29 `SetColors` with both colors set emits ONE combined SGR — `write!(f, csi!("{};{}m"), Colored::ForegroundColor(fg), Colored::BackgroundColor(bg))` (verified in crossterm `src/style.rs` at tag 0.29.0). Therefore the spec §7 byte expectations (`\x1B[38;5;11;49m`, `\x1B[39;49m`, `\x1B[38;5;11;48;5;10m`, `\x1B[0m`) are byte-accurate. Implement the tests exactly as specified.

---

## 1. Preconditions (verify before any edit)

1. Branch is `fix/half-block-rendering` and `git log --oneline -1` shows `6b8daf2 chore: bump version to 0.3.2`. `Cargo.toml` shows `version = "0.3.2"`.
2. `git status` shows exactly one untracked file: `.kilo/plans/20261005-half-block-frontend-spec.md` (plus this plan file once saved). These are **artifacts committed in 4.6** — never stage them in the code commit.
3. Files to touch in 4.2 (code+tests commit): ONLY `src/terminal/renderer.rs` and `tests/terminal_modules.rs`.
4. Files to touch in 4.4 (docs commit): ONLY `README.md`, `docs/terminal-ui.md`, `.agent/project-info/architecture.md`, `.agent/project-info/tech.md`.
5. **`tests/gameplay_flow.rs` needs NO edits — verified:** its only buffer assertions are `count_occurrences(&buffer, "Score: 1") == 1` / `"Score: 0" == 9` (lines 98–99) and `count_occurrences(&buffer, "Score: 0") > 0` (line 127). The score line text is unchanged by this fix; the file contains no border/glyph assertions.
6. The other 5 tick tests in `tests/terminal_modules.rs` (lines 57–139) assert domain state only — no edits.
7. Board dims stay in `src/game/state.rs` (`WIDTH = 80`, `HEIGHT = 80`) — untouched.

---

## 2. Code Changes — `src/terminal/renderer.rs` (exact)

Current file is 108 lines. Target ≈ 162 lines (≤ 200 limit). Every function body below is the exact required implementation. Assembly order = file order given here.

### 2.1 Module doc (replace lines 1–2)

```rust
//! Full-frame board renderer: ASCII borders, half-block-packed board rows,
//! and the score line, redrawn in place from an immutable `GameState`.
//!
//! The board packs two logical rows into each terminal row: terminal row
//! `packed_row` shows logical rows `2 * packed_row` (upper half) and
//! `2 * packed_row + 1` (lower half) through half-block glyphs, so the
//! 80-row board renders as `HEIGHT / 2` packed rows. `HEIGHT` is even, so
//! every logical row lands in exactly one half of one packed row.
```

(The last sentence is the documented even-`HEIGHT` assumption required by spec §4.4 — no panic path.)

### 2.2 Imports (replace lines 4–9)

```rust
use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Colors, Print, ResetColor, SetColors},
};

use crate::game::position::Position;
use crate::game::state::{GameState, HEIGHT, WIDTH};
```

### 2.3 Constants (replace lines 11–19)

**Delete** the four span constants `EMPTY_SPAN`, `BODY_SPAN`, `HEAD_SPAN`, `FOOD_SPAN`. Keep, unchanged:

```rust
const CORNER_GLYPH: char = '+';
const HORIZONTAL_GLYPH: char = '-';
const VERTICAL_GLYPH: char = '|';
const SCORE_PREFIX: &str = "Score: ";
const LINE_BREAK: &str = "\r\n";
```

### 2.4 Struct + `render` (keep `pub` surface: `new` + `render` only)

`pub struct Renderer<W: Write> { output: W }` and `pub fn new` are unchanged.

`pub fn render` — exact new body:

```rust
/// Draw the complete frame for `state` at the home position, without scrolling.
pub fn render(&mut self, state: &GameState) -> io::Result<()> {
    queue!(self.output, MoveTo(0, 0))?;
    self.write_border_row()?;
    self.write_packed_rows(state)?;
    self.write_border_row()?;
    self.write_score_line(state)?;
    queue!(self.output, ResetColor)?;
    self.output.flush()
}
```

(The final `ResetColor` guarantees the following game-over screen text is never tinted by the last board cell's colors.)

### 2.5 Replace `write_board_rows` with packed-row writers

**Delete** `write_board_rows`, `render_board_row`, `cell_span`, `is_snake_head`, `is_snake_body`, `is_food`. **Add** (in this order, inside `impl Renderer<W>`):

```rust
fn write_border_row(&mut self) -> io::Result<()> {
    queue!(self.output, ResetColor)?;
    let border_text = rendered_border_row();
    self.write_line(&border_text)
}

fn write_packed_rows(&mut self, state: &GameState) -> io::Result<()> {
    for packed_row in 0..(HEIGHT / 2) {
        self.write_packed_row(state, packed_row)?;
    }
    Ok(())
}

fn write_packed_row(&mut self, state: &GameState, packed_row: i32) -> io::Result<()> {
    let row_cells = PackedRowCells::for_packed_row(packed_row);
    queue!(self.output, Print(VERTICAL_GLYPH))?;
    for column in 0..WIDTH {
        self.write_packed_cell(state, row_cells.cell_at(column))?;
    }
    queue!(self.output, ResetColor)?;
    queue!(self.output, Print(VERTICAL_GLYPH))?;
    queue!(self.output, Print(LINE_BREAK))?;
    Ok(())
}

fn write_packed_cell(&mut self, state: &GameState, cell: PackedCell) -> io::Result<()> {
    let (glyph, foreground, background) =
        half_block_glyph(cell_kind_at(state, cell.top), cell_kind_at(state, cell.bottom));
    queue!(self.output, SetColors(Colors::new(foreground, background)))?;
    queue!(self.output, Print(glyph))?;
    Ok(())
}

fn write_score_line(&mut self, state: &GameState) -> io::Result<()> {
    queue!(self.output, ResetColor)?;
    let score_text = format!("{SCORE_PREFIX}{}", state.score());
    self.write_line(&score_text)
}

fn write_line(&mut self, content: &str) -> io::Result<()> {
    write!(self.output, "{content}{LINE_BREAK}")
}
```

ResetColor placement audit vs spec §3 (must hold exactly): one per packed row after the last cell and before the trailing `|` (in `write_packed_row`); one before the top border row and one before the bottom border row (the single `write_border_row` body, called twice); one before the score line; one at the very end of `render` before `flush`.

### 2.6 Free functions (after the `impl` block, in this order)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Empty,
    Head,
    Body,
    Food,
}

fn cell_kind_at(state: &GameState, position: Position) -> CellKind {
    if state.snake().head() == position {
        return CellKind::Head;
    }
    if state.snake().segments().contains(&position) {
        return CellKind::Body;
    }
    if state.food().occupies(position) {
        return CellKind::Food;
    }
    CellKind::Empty
}

fn cell_color(kind: CellKind) -> Color {
    match kind {
        CellKind::Head => Color::Yellow,
        CellKind::Body => Color::Green,
        CellKind::Food => Color::Red,
        CellKind::Empty => Color::Reset,
    }
}

fn half_block_glyph(top: CellKind, bottom: CellKind) -> (char, Color, Color) {
    match (top, bottom) {
        (CellKind::Empty, CellKind::Empty) => (' ', Color::Reset, Color::Reset),
        (top_kind, CellKind::Empty) => ('▀', cell_color(top_kind), Color::Reset),
        (CellKind::Empty, bottom_kind) => ('▄', cell_color(bottom_kind), Color::Reset),
        (kind_a, kind_b) if kind_a == kind_b => ('█', cell_color(kind_a), Color::Reset),
        (top_kind, bottom_kind) => ('█', cell_color(top_kind), cell_color(bottom_kind)),
    }
}

struct PackedRowCells {
    top_y: i32,
    bottom_y: i32,
}

impl PackedRowCells {
    fn for_packed_row(packed_row: i32) -> PackedRowCells {
        PackedRowCells { top_y: 2 * packed_row, bottom_y: 2 * packed_row + 1 }
    }

    fn cell_at(&self, column: i32) -> PackedCell {
        PackedCell {
            top: Position { x: column, y: self.top_y },
            bottom: Position { x: column, y: self.bottom_y },
        }
    }
}

struct PackedCell {
    top: Position,
    bottom: Position,
}
```

Classification precedence is exactly **head > body > food > empty** per half-cell (spec §4.1). `PackedRowCells`/`PackedCell` exist to keep every function at ≤ 2 non-self parameters (max-arguments rule).

### 2.7 `rendered_border_row` — one-character change

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

The only change vs the current body: `0..(WIDTH * 2)` → `0..WIDTH` (border row = `+` + `-`×80 + `+` = 82 chars).

### 2.8 Rule compliance checklist for this file

- `pub` surface = `Renderer` struct, `new`, `render` only; `CellKind`, helpers, `PackedRowCells`, `PackedCell` all private.
- Longest method body: `write_packed_row` ≈ 9 lines (≤ 50 ✓); max nesting depth 2 (the `for` loop body) ✓; max 2 non-self params ✓.
- File total ≈ 162 lines (≤ 200 ✓). No commented-out code; no new dependencies.

---

## 3. Test Changes — `tests/terminal_modules.rs` (exact)

### 3.1 Imports (replace line 5)

Old: `use snake::game::setup::initial_setup;`
New:

```rust
use snake::game::food::Food;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
```

### 3.2 New helper (insert directly after `fresh_game`, before `arrow_press`)

```rust
fn vertical_snake_game() -> GameState {
    GameState::new(GameStateSetup {
        snake: Snake::new(Vec::from([
            Position { x: 10, y: 12 },
            Position { x: 10, y: 13 },
        ])),
        food: Food::new(Position { x: 0, y: 0 }),
        direction: Direction::Right,
    })
}
```

### 3.3 `render_writes_the_full_frame_into_the_buffer` — replace the whole assertion block (old lines 151–160) with:

```rust
    assert_eq!(count_occurrences(&buffer, "+"), 4);      // 4 corners
    assert_eq!(count_occurrences(&buffer, "-"), 160);    // 80 per border row * 2 borders
    assert_eq!(count_occurrences(&buffer, "|"), 80);     // 2 per packed row * 40 packed rows
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
    assert_eq!(count_occurrences(&buffer, "\r\n"), 43);  // 1 top + 40 packed + 1 bottom + 1 score

    // Initial state: head, 2 body segments, and food all sit in logical row 12,
    // the top half of packed row 6; every bottom half is empty.
    assert_eq!(count_occurrences(&buffer, "▀"), 4);
    assert_eq!(count_occurrences(&buffer, "▄"), 0);
    assert_eq!(count_occurrences(&buffer, "█"), 0);

    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 1); // yellow head
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 2); // green body
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 1);  // red food
```

### 3.4 `tick_renders_one_consistent_frame_of_glyphs` — replace the whole assertion block (old lines 174–177) with:

```rust
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 1);
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 2);
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 1);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
    assert_eq!(count_occurrences(&buffer, "\r\n"), 43);
```

(After one tick right: head `(11, 12)`, body `(10, 12)` + `(9, 12)`, food `(20, 12)` — all still top-half of packed row 6. The trailing `\r\n` assertion is the one the spec marks acceptable — include it.)

### 3.5 `two_renders_reuse_the_frame_without_scrolling` — replace the whole assertion block (old lines 191–194) with:

```rust
    assert_eq!(count_occurrences(&buffer, "+"), 8);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 2);
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 4);
    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 2);
    assert_eq!(count_occurrences(&buffer, "\r\n"), 86);  // 43 * 2 frames
```

### 3.6 Two NEW tests — append at the end of the file, in this order:

```rust
#[test]
fn vertical_neighbors_render_as_a_full_block() {
    let state = vertical_snake_game();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        renderer.render(&state).expect("render succeeds");
    }

    assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;48;5;10m█"), 1);
}

#[test]
fn empty_cells_render_with_default_colors() {
    let state = fresh_game();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        renderer.render(&state).expect("render succeeds");
    }

    // 80 columns * 40 packed rows = 3200 board cells; 4 are occupied in the
    // initial state, and each empty cell emits the default-color SGR + a space.
    assert_eq!(count_occurrences(&buffer, "\x1B[39;49m "), 3196);
}
```

Rationale anchors: in `vertical_snake_game` the head `(10, 12)` sits directly above body `(10, 13)` → packed row 6, column 10 → top=Head, bottom=Body → `█` with Yellow fg / Green bg (the spec §7.4 sequence). The spec recommends §7.5 — it is included (decision: not optional here).

Counting correctness (verified): byte-window matching is safe because `▀`=E2 96 80, `▄`=E2 96 84, `█`=E2 96 88 differ in the 3rd byte, no ANSI sequence contains `+`, `-`, `|`, or `\r\n`, and only empty cells emit `\x1B[39;49m`.

---

## 4. Docs Edits (step 4.4 allocation — exact old → new)

### 4.1 `README.md`

**Line 27** (size note):

- OLD: `- Terminal size: with the 80 x 80 board the full frame spans about 162 columns x 83 rows (each board cell renders two columns wide; borders + score line); use a terminal window of at least ~164 x 84 character cells, and enlarge or resize the window (e.g. in Windows Terminal or the classic console) if the frame looks cut off.`
- NEW: `- Terminal size: with the 80 x 80 board the full frame spans 82 columns x 43 rows (two logical rows are packed into each terminal row using half-block glyphs; borders + score line); it fits a terminal window of at least ~84 x 45 character cells.`

**Line 40** (accuracy fix — head/body are no longer distinct glyphs):

- OLD fragment: `draws a full frame — ASCII borders, distinct snake head/body glyphs, food, and a \`Score: N\` line`
- NEW fragment: `draws a full frame — ASCII borders, color-coded half-block glyphs (head yellow, body green, food red, two logical rows packed per terminal row), and a \`Score: N\` line`

### 4.2 `docs/terminal-ui.md`

**Lines 61–63** (Public API, Renderer glyph bullet):

- OLD: `- Glyphs: each cell is a two-column span — head \`●●\`, body \`██\`, food \`◆◆\`,\n    empty \`  \`; borders stay single-character \`+ - |\` (border row is \`-\` × (2×WIDTH));\n    score line \`Score: <n>\`.`
- NEW: `- Glyphs: each terminal row packs two logical rows via half-block glyphs —\n    upper half \`▀\` (U+2580), lower half \`▄\` (U+2584), full block \`█\` (U+2588)\n    when both halves are occupied, space when empty; colors via crossterm\n    \`SetColors\`: head yellow, body green, food red, empty default; borders stay\n    single-character \`+ - |\` (border row is \`-\` × WIDTH); score line \`Score: <n>\`;\n    frame is 82 columns × 43 rows.`

**Lines 144–146** (Headless Tests bullet):

- OLD fragment: `frame\n  snapshots via byte counting (border corners, glyph counts, score line):`
- NEW fragment: `frame\n  snapshots via byte counting (border corners, glyph counts, color sequences,\n  score line):`

**Lines 162–163** (manual validation bullet):

- OLD: `- Board draws once per tick and does **not** scroll; head and body glyphs are\n  distinct.`
- NEW: `- Board draws once per tick and does **not** scroll; head, body, and food are\n  distinguishable by color (yellow / green / red).`

### 4.3 `.agent/project-info/architecture.md`

**Line 75** (Renderer component description):

- OLD fragment: `draws border rows (\`+\`/\`-\`/\`|\`), per-cell two-column glyph spans (head \`●●\` U+25CF×2, body \`██\` U+2588×2, food \`◆◆\` U+25C6×2, empty two spaces), and a \`Score: N\` line, queued at the home position and flushed.`
- NEW fragment: `draws border rows (\`+\`/\`-\`/\`|\`), packs two logical rows per terminal row via half-block glyphs (\`▀\` U+2580 upper half, \`▄\` U+2584 lower half, \`█\` U+2588 both halves; colors: head yellow, body green, food red, empty default), and a \`Score: N\` line, queued at the home position and flushed.`

**Line 101** (board bullet — append the frame size):

- OLD fragment: `visible ASCII boundaries (\`+\` corners, \`-\` horizontal, \`|\` verticals); no screen wrap (brief §8).`
- NEW fragment: `visible ASCII boundaries (\`+\` corners, \`-\` horizontal, \`|\` verticals); the frame renders as 82 columns × 43 rows (two logical rows packed per terminal row); no screen wrap (brief §8).`

**Line 102** (cell-glyph bullet):

- OLD: `- Cell glyphs (double-width since 2026-10-05, Task 4): each logical cell renders as a two-column span — snake head \`●●\` (U+25CF ×2), snake body \`██\` (U+2588 ×2), food \`◆◆\` (U+25C6 ×2), empty space two spaces; borders stay single-character (\`+\` corners, \`-\` ×(2×WIDTH) horizontal rows, \`|\` verticals); score line rendered as \`Score: N\` (brief §5, §10, §13).`
- NEW: `- Cell glyphs (half-block packed since 2026-10-05): each terminal row packs two logical rows — upper half \`▀\` (U+2580), lower half \`▄\` (U+2584), full block \`█\` (U+2588) when both halves are occupied, space when empty; colors via crossterm \`SetColors\`: head yellow, body green, food red, empty default; borders stay single-character (\`+\` corners, \`-\` × WIDTH horizontal rows, \`|\` verticals); score line rendered as \`Score: N\` (brief §5, §10, §13).`

### 4.4 `.agent/project-info/tech.md`

**Line 52** (resolved-decision glyph list):

- OLD fragment: `**resolved in Phase 1B, updated 2026-10-05 (Task 4 double-width spans)** (see \`src/terminal/renderer.rs\`): head \`●●\` (U+25CF ×2), body \`██\` (U+2588 ×2), food \`◆◆\` (U+25C6 ×2), empty two spaces (U+0020 ×2); ASCII borders \`+\` corners, \`-\` ×(2×WIDTH), \`|\` verticals; score line \`Score: N\`.`
- NEW fragment: `**resolved in Phase 1B, updated 2026-10-05 (half-block packed rows)** (see \`src/terminal/renderer.rs\`): two logical rows packed per terminal row — upper half \`▀\` (U+2580), lower half \`▄\` (U+2584), full block \`█\` (U+2588) when both halves are occupied, space when empty; colors via crossterm \`SetColors\`: head yellow, body green, food red, empty default; ASCII borders \`+\` corners, \`-\` × WIDTH, \`|\` verticals; score line \`Score: N\`; frame 82 columns × 43 rows.`

(tech.md line 56 is a historical research note — leave untouched.)

---

## 5. Git Plan

1. **Commit 1 (step 4.2, code + tests):**
   - Read `.gitignore` and run `git status` first (gitignore-compliance rule). `dist/` and `target/` are ignored — never stage them.
   - Stage explicit paths ONLY (the untracked spec/plan files belong to 4.6): `git add src/terminal/renderer.rs tests/terminal_modules.rs`
   - Commit: `fix: pack board rows with half-block glyphs to fit short consoles`
2. **Commit 2 (step 4.4, docs):**
   - `git add README.md docs/terminal-ui.md .agent/project-info/architecture.md .agent/project-info/tech.md`
   - Commit: `docs: update rendering docs for half-block packed board`
3. **Commit 3 (step 4.6, artifacts + completion — implementer):**
   - Append `[DONE]` to the task line in `.agent/todos/20261005/20261005-todo-2.md` (preserve all other content — overwrite-TODO-prevention rule).
   - Add one context bullet to `.agent/project-info/context.md` under "Recent Changes" describing this fix (half-block packed renderer, 82×43 frame, version already 0.3.2, tests re-specified + 2 new).
   - Stage: the TODO file, `context.md`, `.kilo/plans/20261005-half-block-frontend-spec.md`, `.kilo/plans/20261005-half-block-rendering.md`, and any 4.3/4.5 report files produced for this task.
   - Commit: `docs: mark half-block rendering task done`
   - Do NOT push (push is step 5).

---

## 6. Verification (non-blocking, record exit codes)

1. **VM Docker release build** (optional, non-blocking): via alpine-vm MCP — first `vm_status`; if running, `vm_run_command` with `sh -c "cd /rust-snake && docker compose run --rm build"`. Record the exit code. If the VM is not running, record "skipped (VM unavailable)" and continue — do not block the workflow.
2. **Compile-only test check (no test execution):** via alpine-vm MCP, `vm_run_command` with `sh -c "cd /rust-snake && docker compose run --rm build cargo test --no-run --target x86_64-pc-windows-gnu"` (the trailing args override the compose service command; it compiles test binaries inside the container and runs nothing). Record the exit code.
3. Never stage `dist/` or `target/`; the compile-only command writes nothing to `dist/`.

---

## 7. Out of Scope (restate — hard blocked)

- `src/game/**` (including `state.rs` board dims), `src/terminal/game_loop.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `src/main.rs`.
- `Cargo.toml` dependencies (no new crates; crossterm 0.29 stays).
- `brief.md`; board dimensions (`WIDTH`/`HEIGHT` stay 80); game timing, input, lifecycle.
- TODO `[DONE]` marks (reserved for 4.6); git push (reserved for step 5); branch create/switch (done in step 2).

---

## 8. Acceptance Checklist (map to spec §9)

| Criterion | Covered by |
|---|---|
| a. 43-row frame visible without scrolling | Frame layout §2.4–2.5 (43 rows × 82 cols); manual validation pending (user) |
| b. Contiguous body; vertical neighbors = `█` | Test `vertical_neighbors_render_as_a_full_block` (§3.6); manual on-screen check pending |
| c. Head/body/food distinguishable by color | SGR assertions in §3.3–3.5; manual color check pending |
| d. One logical vertical step ≈ half a terminal row | Architectural consequence of the packing; manual validation pending |
| e. No legacy two-column spans (`██`, `●●`, `◆◆`) | Old span constants deleted (§2.3); old `●`/`◆` assertions replaced (§3.3–3.5) |

## 9. Plan Self-Check vs TODO + Spec

- TODO line: pack two logical rows per terminal row with `▀`/`▄`/`█` + colors; frame 82×43; board stays 80×80 logical; visual squareness + speed parity preserved → encoded in §2 (packing, colors, border math) with no board-dim changes.
- Spec §3 emission strategy → §2.4–2.5 exactly; §4 classification/mapping → §2.6 verbatim; §5 borders/score → §2.5/§2.7; §6 frame layout → §2.4; §7 test contract → §3 verbatim (byte table verified against crossterm 0.29 source); §10 constraints → §2.8 checklist. No open alternatives remain.
