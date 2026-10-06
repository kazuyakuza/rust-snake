# Front-end Technical Specification — Half-Block Packed Renderer

**Task:** Fix the 80×80 board overflow on short Windows consoles by packing two logical rows into one terminal row using half-block glyphs and crossterm colors.

**Plan file:** `.kilo/plans/20261005-half-block-frontend-spec.md`  
**Global plan:** `.kilo/plans/20261005-half-block-rendering-fix.md`  
**Target file:** `src/terminal/renderer.rs`  
**Target tests:** `tests/terminal_modules.rs` renderer snapshot tests  
**Branch:** `fix/half-block-rendering`

---

## 1. Scope

This spec covers ONLY the terminal rendering layer for the packed half-block board.

**In scope:**

- `src/terminal/renderer.rs` — rewrite of row-drawing internals.
- `tests/terminal_modules.rs` — the three renderer snapshot tests (and any new snapshot tests needed to cover vertical packing).

**Out of scope (do NOT touch):**

- `src/game/**`, `src/terminal/game_loop.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `src/main.rs`.
- `Cargo.toml` dependencies.
- `brief.md`, board dimensions (`WIDTH`/`HEIGHT` stay `80`), game timing, input, lifecycle.

---

## 2. Target Framework

- Rust edition 2021, package `snake`.
- `crossterm 0.29` (already a dependency). No new crates.
- crossterm types used:
  - `crossterm::style::{Color, Colors, Print, ResetColor, SetColors}`
  - `crossterm::cursor::MoveTo`
  - `crossterm::queue!`
- Color palette:
  - Head: `Color::Yellow`
  - Body: `Color::Green`
  - Food: `Color::Red`
  - Empty / border / score: default (`Color::Reset`).

---

## 3. Emission Strategy (Decided)

**Use `queue!(out, SetColors(Colors::new(fg, bg)), Print(glyph))` for every board cell.**

Rationale:

- Junior-understandable: each cell is one `SetColors` + one `Print`.
- No inline SGR string building — crossterm handles the ANSI bytes, and tests still see deterministic ANSI sequences in a `Vec<u8>`.
- Minimal per-tick overhead is acceptable; correctness over micro-optimization.
- The `queue!` + `flush` pattern already exists in the renderer.

### Reset-to-default handling (decided)

1. **Every board cell emits a `SetColors` command.**
   - Occupied cells: `SetColors(Colors::new(occupied_color, Color::Reset))` or with a real background when two different halves stack.
   - Empty cells: `SetColors(Colors::new(Color::Reset, Color::Reset))`.
2. **Emit `ResetColor` once per packed row** after the last cell and **before** the trailing `|`.
3. **Emit `ResetColor` once before the top border row, once before the bottom border row, once before the score line, and once at the very end of the frame** (terminal hygiene before `flush`).

This guarantees border characters and the score line are never tinted by a neighboring colored cell.

---

## 4. Cell Classification & Half-Block Mapping

### 4.1 Cell kind

Introduce a private enum in `src/terminal/renderer.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Empty,
    Head,
    Body,
    Food,
}
```

Classification function (private, reuses existing `Snake`/`Food` checks):

```rust
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
```

Precedence is exactly **head > body > food > empty** per half-cell.

### 4.2 Color helper

```rust
fn cell_color(kind: CellKind) -> Color {
    match kind {
        CellKind::Head => Color::Yellow,
        CellKind::Body => Color::Green,
        CellKind::Food => Color::Red,
        CellKind::Empty => Color::Reset,
    }
}
```

### 4.3 Half-block mapping

For a packed row `packed_y` (0..40), the top logical row is `2 * packed_y` and the bottom logical row is `2 * packed_y + 1`.

For each column `x` in `0..WIDTH`:

```rust
let top = cell_kind_at(state, Position { x, y: top_y });
let bottom = cell_kind_at(state, Position { x, y: bottom_y });
let (glyph, fg, bg) = half_block_glyph(top, bottom);
```

Mapping table (final, no alternatives):

| Top kind | Bottom kind | Glyph | Foreground | Background | Reason |
|----------|-------------|-------|------------|------------|--------|
| Empty    | Empty       | ` ` (space) | `Reset` | `Reset` | empty cell |
| `T`      | Empty       | `▀` U+2580  | `cell_color(T)` | `Reset` | top half colored |
| Empty    | `B`         | `▄` U+2584  | `cell_color(B)` | `Reset` | bottom half colored |
| `K`      | `K`         | `█` U+2588  | `cell_color(K)` | `Reset` | both same; glyph fills cell |
| `T`      | `B` (T≠B)   | `█` U+2588  | `cell_color(T)` | `cell_color(B)` | both different; top color on upper half, bottom color behind it |

Implementation shape:

```rust
fn half_block_glyph(top: CellKind, bottom: CellKind) -> (char, Color, Color) {
    match (top, bottom) {
        (CellKind::Empty, CellKind::Empty) => (' ', Color::Reset, Color::Reset),
        (top_kind, CellKind::Empty) => ('▀', cell_color(top_kind), Color::Reset),
        (CellKind::Empty, bottom_kind) => ('▄', cell_color(bottom_kind), Color::Reset),
        (kind_a, kind_b) if kind_a == kind_b => ('█', cell_color(kind_a), Color::Reset),
        (top_kind, bottom_kind) => ('█', cell_color(top_kind), cell_color(bottom_kind)),
    }
}
```

### 4.4 Packed row count

`HEIGHT` is 80 (even). The number of packed terminal rows is `HEIGHT / 2 = 40`. Document this as a module-level assumption; no panic path is required.

---

## 5. Border & Score Rendering

### 5.1 Borders

- Top border: `+` + `-` repeated `WIDTH` times + `+` → exactly 82 characters.
- Bottom border: identical.
- Left/right vertical borders: single `|` per packed row.
- Borders are always rendered with default colors (`ResetColor` prefix per Section 3).

Old double-width border (`-` repeated `2 * WIDTH` times) is replaced by single-width border.

### 5.2 Score line

Unchanged text: `Score: {score}` (e.g., `Score: 0`). Rendered with default colors (`ResetColor` prefix). It is the last line of the 43-row frame.

---

## 6. Frame Layout

A complete frame is:

```text
Row  0: +----------------------------------------+  (top border, 82 chars)
Rows 1..40: |<80 packed cells>|                  (40 packed rows, 82 chars each)
Row 41: +----------------------------------------+  (bottom border, 82 chars)
Row 42: Score: 0
```

Total: **43 rows × 82 columns**.

Line endings remain `\r\n` (`const LINE_BREAK: &str = "\r\n"`).

`MoveTo(0, 0)` is queued before the first line, so the frame redraws in place.

---

## 7. Test Contract (Headless `Vec<u8>`)

Tests must drive `Renderer::new(&mut buffer)` over an in-memory `Vec<u8>` and assert the produced byte stream. Crossterm 0.29 emits the following ANSI color sequences:

| Color | Foreground sequence | Background sequence |
|-------|---------------------|---------------------|
| `Color::Yellow` | `\x1B[38;5;11m` | `\x1B[48;5;11m` |
| `Color::Green`  | `\x1B[38;5;10m` | `\x1B[48;5;10m` |
| `Color::Red`    | `\x1B[38;5;9m`  | `\x1B[48;5;9m`  |
| `Color::Reset`  | `\x1B[39m`      | `\x1B[49m`      |

Combined examples from `SetColors(Colors::new(fg, bg))`:

- Yellow fg, Reset bg (head top-half): `\x1B[38;5;11;49m`
- Green fg, Reset bg (body top-half): `\x1B[38;5;10;49m`
- Red fg, Reset bg (food top-half): `\x1B[38;5;9;49m`
- Reset fg, Reset bg (empty cell): `\x1B[39;49m`
- Yellow fg, Green bg (head on top of body): `\x1B[38;5;11;48;5;10m`

`ResetColor` emits `\x1B[0m`.

### 7.1 Updated existing snapshot test: `render_writes_the_full_frame_into_the_buffer`

Use `fresh_game()` (initial snake head at `(10, 12)`, body at `(9, 12)` and `(8, 12)`, food at `(20, 12)`). All three snake segments and the food are in logical row 12 → packed row `6`, top half only; bottom half (row 13) is empty.

Required assertions:

```rust
assert_eq!(count_occurrences(&buffer, "+"), 4);
assert_eq!(count_occurrences(&buffer, "-"), 160);       // 80 per border * 2 borders
assert_eq!(count_occurrences(&buffer, "|"), 80);        // 2 per packed row * 40
assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
assert_eq!(count_occurrences(&buffer, "\r\n"), 43);     // 1 top + 40 packed + 1 bottom + 1 score

// Half-block glyphs in the initial state
assert_eq!(count_occurrences(&buffer, "▀"), 4);         // head, 2× body, food (all top-half)
assert_eq!(count_occurrences(&buffer, "▄"), 0);
assert_eq!(count_occurrences(&buffer, "█"), 0);

// Color sequences tied to the occupied top-half cells
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 1); // yellow head
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 2); // green body (two segments)
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 1);  // red food
```

### 7.2 Updated existing snapshot test: `tick_renders_one_consistent_frame_of_glyphs`

After one `tick` to the right, the snake is head `(11, 12)`, body `(10, 12)` and `(9, 12)`; food remains `(20, 12)`. All still top-half of packed row 6.

Required assertions:

```rust
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 1);
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 2);
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 1);
assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
```

It is acceptable to keep a `count_occurrences(&buffer, "\r\n") == 43` assertion as well.

### 7.3 Updated existing snapshot test: `two_renders_reuse_the_frame_without_scrolling`

Render the same state twice. Required assertions:

```rust
assert_eq!(count_occurrences(&buffer, "+"), 8);
assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;49m▀"), 2);
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;10;49m▀"), 4);
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;9;49m▀"), 2);
assert_eq!(count_occurrences(&buffer, "\r\n"), 86);   // 43 * 2
```

### 7.4 New snapshot test: vertical packing produces a full block

Add a test that constructs a `GameState` with the head directly above a body segment and asserts the `█` glyph with combined foreground/background colors.

Suggested state (build via `GameStateSetup`):

- Snake segments: `[Position { x: 10, y: 12 }, Position { x: 10, y: 13 }]` (head at top).
- Food anywhere valid outside those cells (e.g., `Position { x: 0, y: 0 }`).
- Direction `Right`.

Assertion:

```rust
assert_eq!(count_occurrences(&buffer, "\x1B[38;5;11;48;5;10m█"), 1);
```

This confirms the `█` path and the foreground-top / background-bottom rule.

### 7.5 New snapshot test: empty-cell default color

Add a test that asserts the default color sequence appears for empty cells. Using the initial state:

```rust
// 80 columns * 40 packed rows = 3200 board cells; 4 are occupied in the initial state.
assert_eq!(count_occurrences(&buffer, "\x1B[39;49m "), 3196);
```

This is optional but recommended to lock in the per-cell default-color behavior.

---

## 8. Terminal-Size Acceptance

The new frame is **82 columns × 43 rows**.

- Fits any console window that can display at least **84 columns × 45 rows** (including scrollbar/border margins).
- This replaces the previous README/docs note of ~164×84 required for the 162×83 double-width frame.
- Documentation updates are out of scope for this spec; the docs-specialist step will update `README.md`, `docs/terminal-ui.md`, `.agent/project-info/architecture.md`, and `.agent/project-info/tech.md`.

---

## 9. Visual Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| a | The whole 43-row frame is visible without scrolling in a standard Windows console. | Manual validation pending (user). |
| b | The snake body is contiguous: vertical neighbors render as `█`; horizontal neighbors are adjacent columns. | Covered by the vertical-packing test (#7.4) and by general glyph counts. Manual validation pending for on-screen appearance. |
| c | Head, body, and food are distinguishable by color (Yellow, Green, Red). | Covered by color-sequence tests (#7.1–7.3). Manual validation pending for actual terminal colors. |
| d | One logical vertical step is visually ~half a terminal row, preserving approximate parity with the horizontal step width. | Architectural consequence of the packing; manual validation pending. |
| e | No legacy two-column spans (`██`, `●●`, `◆◆`) remain in the renderer output. | Verified by tests: `count_occurrences(&buffer, "●") == 0`, `count_occurrences(&buffer, "◆") == 0`, and the old wide body span pattern is gone. |

---

## 10. Implementation Constraints

- Source code stays under `src/`.
- Methods/functions must remain ≤50 lines; nesting depth ≤2.
- Prefer private members (the new `CellKind`, helpers, and color constants are private).
- No comments that merely restate code; prefer self-documenting names.
- No commented-out code.
- No new dependencies.
- Do not modify `src/game/state.rs` or any other `src/game/**` file.

---

## 11. Deliverable Status

- **This file** is the Front-end Technical Specification for step 4.1a.
- It is saved uncommitted; later workflow steps will commit it.
- Implementation, verification, and documentation updates are NOT part of this step.
