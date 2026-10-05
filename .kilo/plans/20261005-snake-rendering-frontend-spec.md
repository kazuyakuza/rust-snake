# Front-end Technical Specification — Snake Terminal Rendering Fix

**Task:** Task 4 of `20261005-todo-1.md` (snake visualization + velocity fix)  
**Workflow step:** 4.1a Front-end Technical Specification  
**Date:** 2026-10-05  
**Scope:** Terminal presentation layer only (`src/terminal/renderer.rs` and its headless tests in `tests/terminal_modules.rs`).

---

## 1. Scope & Summary

Fix the two user-reported terminal rendering symptoms:

1. **Disjoint snake body** — body cells render as separate 1-column squares with visible gaps.
2. **Faster vertical motion** — one logical cell vertically covers roughly twice the pixel distance of one logical cell horizontally because Windows terminal cells are ~2× taller than wide.

The domain (`src/game/`) and the game tick timing (`src/terminal/game_loop.rs`) are correct and must not change. The fix is purely in the renderer: **each logical board cell maps to exactly 2 terminal columns**, so horizontal and vertical pixel-steps become approximately equal and adjacent body cells become contiguous.

---

## 2. Target Stack

- **Language / framework:** Rust 2021, package `snake`.
- **Terminal library:** `crossterm 0.29` (already a dependency; no new crates allowed).
- **Output target:** any `W: std::io::Write` — real `stdout()` in the binary, `&mut Vec<u8>` in headless tests.
- **Unicode output:** already used by the existing renderer; the new glyphs are standard Unicode box-drawing/block characters supported by Windows Terminal and modern consoles.

---

## 3. Component Boundaries

Only `src/terminal/renderer.rs` changes. All other files are read-only for this task:

- `src/game/*` — source of truth for board dimensions (`WIDTH`, `HEIGHT`), snake position, food, score, status.
- `src/terminal/game_loop.rs` — consumes `Renderer::render`; unchanged.
- `src/terminal/input.rs` — unchanged.
- `src/terminal/lifecycle.rs` — unchanged.
- `src/main.rs` — unchanged; the start / game-over screens already use the lifecycle handle's output and are not affected by glyph widths.
- `tests/terminal_modules.rs` — updated assertions only; no test structure changes.

`Renderer` remains a stateless-per-call wrapper around an `io::Write`. It continues to expose exactly:

- `Renderer::new(output: W) -> Renderer<W>`
- `Renderer::render(&mut self, state: &GameState) -> io::Result<()>`

No new public methods, no new constructor variants.

---

## 4. Cell-to-Terminal Mapping (Decision)

**Approach chosen: A — color-free glyph spans.**

Rationale: crossterm colors are available without adding dependencies, but color-free glyphs keep the implementation and the headless `Vec<u8>` snapshot tests simpler (no ANSI color sequences to account for). The selected glyphs still satisfy all visual acceptance criteria.

Each logical cell renders as a **2-character span**:

| Logical cell | Span | Unicode | Visual purpose |
| --- | --- | --- | --- |
| Empty | `"  "` | U+0020 SPACE ×2 | Background / no snake, no food. |
| Snake body | `"██"` | U+2588 FULL BLOCK ×2 | Contiguous block; adjacent body cells touch horizontally and vertically. |
| Snake head | `"●●"` | U+25CF BLACK CIRCLE ×2 | Distinct from body; wider than the original single dot. |
| Food | `"◆◆"` | U+25C6 BLACK DIAMOND ×2 | Distinct from both head and body. |

**Exact two-character spans to encode in the source:**

```rust
const EMPTY_SPAN: &'static str = "  ";
const BODY_SPAN: &'static str = "██";
const HEAD_SPAN: &'static str = "●●";
const FOOD_SPAN: &'static str = "◆◆";
```

The original single-character constants (`HEAD_GLYPH`, `BODY_GLYPH`, `FOOD_GLYPH`, `EMPTY_GLYPH`) are replaced by these span constants. Border glyphs remain single characters (see §5).

---

## 5. Border Format

Borders frame the doubled cells. A board row is: single `|` + `WIDTH` two-column spans + single `|`. Therefore the border row must be exactly the same width: single `+`, then `-` repeated `2 × WIDTH` times, then single `+`.

Border constants remain:

```rust
const CORNER_GLYPH: char = '+';
const HORIZONTAL_GLYPH: char = '-';
const VERTICAL_GLYPH: char = '|';
```

**Exact border row format (for `WIDTH = 80`):**

```text
+----------------------------------------------------------------------------------------------------------------------------------------------------------------+
```

- Length: `2 × WIDTH + 2 = 162` characters.
- Corners are single `+`.
- Horizontal bar is `-` repeated `160` times.
- Vertical sides are single `|` at the start and end of every board row.

No doubling of border glyphs; the single `|` aligns with the outer edge of the doubled cell columns.

---

## 6. Score Line & Screens

- **Score line:** unchanged. Rendered exactly as `Score: N` followed by the existing `LINE_BREAK` (`"\r\n"`).
- **Start screen:** unchanged (`Press any key to start`).
- **Game-over screen:** unchanged (`GAME OVER`, blank line, `Score: N`, blank line, `Press any key to exit`).

These screens are produced by private helpers in `src/main.rs` and are outside the renderer; they do not need double-width handling because they contain only text.

---

## 7. Frame Dimensions & Terminal Size Requirement

With `WIDTH = 80` and `HEIGHT = 80`:

- **Rendered frame width:** `2 × WIDTH + 2 = 162` terminal columns.
- **Rendered frame height:** `HEIGHT` board rows + 2 border rows + 1 score line = `83` terminal rows.

**Documentation note for step 4.4:** update the README / `docs/terminal-ui.md` terminal-size guidance to recommend a terminal window of at least **~164 × 84 character cells** (slightly larger than the 162 × 83 frame to avoid clipping from window chrome / scrollbars). The previous note of ~82 × 83 is obsolete because each board cell now occupies two columns.

---

## 8. Renderer Internals Guidance

Keep the current structure; only the glyph-mapping and border-length logic change.

- `render()` must continue to:
  1. `queue!(self.output, MoveTo(0, 0))?;`
  2. `write_border_row()?;`
  3. `write_board_rows(state)?;`
  4. `write_border_row()?;`
  5. `write_score_line(state)?;`
  6. `self.output.flush()`

- `write_board_rows` iterates `row in 0..HEIGHT`; inside, build each row as a `String`.

- `render_board_row(state, row)` builds the row string:
  1. `push(VERTICAL_GLYPH)`
  2. for `column in 0..WIDTH`:
     - `push_str(self.cell_span(state, Position { x: column, y: row }))`
  3. `push(VERTICAL_GLYPH)`
  4. return the `String`

- `cell_glyph(state, cell) -> char` becomes `cell_span(state, cell) -> &'static str` and returns one of the four span constants from §4.

- `rendered_border_row()` builds:
  1. `push(CORNER_GLYPH)`
  2. for `_ in 0..(WIDTH * 2)`:
     - `push(HORIZONTAL_GLYPH)`
  3. `push(CORNER_GLYPH)`
  4. return the `String`

- Continue using `write!(self.output, "{content}{LINE_BREAK}")` for line output.

- **Do not** introduce per-cell `MoveTo` calls, cursor gymnastics, or color control sequences.

- **Do not** leave the old single-character glyph constants as commented-out code; remove them.

---

## 9. Headless Test Expectations

The snapshot tests in `tests/terminal_modules.rs` remain over `Vec<u8>` and must be updated to match the doubled output. The `count_occurrences` helper can stay unchanged (it counts byte windows).

### 9.1 Per-frame counts for one `render()` call

With the initial state (head at one cell, 2 body cells, 1 food cell, `WIDTH = 80`, `HEIGHT = 80`):

```rust
assert_eq!(count_occurrences(&buffer, "+"), 4);      // 4 corners
assert_eq!(count_occurrences(&buffer, "-"), 320);    // 160 per border row * 2 borders
assert_eq!(count_occurrences(&buffer, "|"), 160);    // 2 per board row * 80 rows
assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
assert_eq!(count_occurrences(&buffer, "\r\n"), 83);  // 80 board + 2 border + 1 score lines

// Each logical cell is now 2 terminal columns, so single-character counts double:
assert_eq!(count_occurrences(&buffer, "●"), 2);      // head span = 2 circles
assert_eq!(count_occurrences(&buffer, "█"), 4);      // 2 body cells * 2 full blocks each
assert_eq!(count_occurrences(&buffer, "◆"), 2);      // food span = 2 diamonds
```

**Important:** remove the old `assert_eq!(count_occurrences(&buffer, "■"), 2);` assertion — U+25A0 is no longer emitted.

### 9.2 Per-frame counts after `tick()`

In `tick_renders_one_consistent_frame_of_glyphs`:

```rust
assert_eq!(count_occurrences(&buffer, "●"), 2);
assert_eq!(count_occurrences(&buffer, "█"), 4);
assert_eq!(count_occurrences(&buffer, "◆"), 2);
assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
```

### 9.3 Two-frame counts

In `two_renders_reuse_the_frame_without_scrolling`:

```rust
assert_eq!(count_occurrences(&buffer, "+"), 8);
assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
assert_eq!(count_occurrences(&buffer, "●"), 4);     // 2 frames * 2 head chars
assert_eq!(count_occurrences(&buffer, "\r\n"), 166); // 83 lines * 2 frames
```

### 9.4 Row-length sanity (optional but recommended)

The first 82 lines (top border, 80 board rows, bottom border) each contain exactly 162 characters before the `\r\n`. The score line contains 8 characters (`Score: 0`). This can be verified by decoding the buffer as UTF-8 and splitting lines, but the byte-count assertions above are the required minimum.

---

## 10. Explicitly Out of Scope

The following are **not** part of this front-end spec and must not be modified:

- Domain logic in `src/game/*` (board dimensions are already `80 × 80`; no further change).
- Tick timing / game speed (`TICK_DURATION = 120 ms` in `src/terminal/game_loop.rs`).
- Input handling (`src/terminal/input.rs`).
- Terminal lifecycle / raw mode / alternate screen (`src/terminal/lifecycle.rs`).
- Start / game-over screen text (private helpers in `src/main.rs`).
- Adding new dependencies (crossterm only).
- Using crossterm colors (decided out in §4).
- `ratatui` or any other terminal widget library.

---

## 11. Visual Acceptance Criteria

Mapped directly to the user's reported symptoms:

1. **Contiguous body:** consecutive snake body cells render with no visible gaps horizontally and vertically. `BODY_SPAN = "██"` (U+2588 FULL BLOCK ×2) guarantees the filled block fills both terminal columns of the cell.
2. **Equal visual speed:** one logical cell of horizontal motion (2 terminal columns) covers approximately the same pixel distance as one logical cell of vertical motion (1 terminal row), neutralizing the ~2:1 Windows terminal cell aspect ratio.
3. **Head distinct from body:** the head uses `"●●"` while the body uses `"██"`; the different glyph shapes make the head visually distinguishable.
4. **Food distinct from snake:** food uses `"◆◆"`, distinct from both `"●●"` and `"██"`.
5. **Frame alignment:** the full rendered frame is exactly 162 columns × 83 rows and fits without truncation when the terminal is at least ~164 × 84 cells.

---

## 12. Implementation Constraints

- Max 200 lines per source file; `src/terminal/renderer.rs` is currently 108 lines and must remain under 200 after the change.
- Method bodies ≤ 50 lines.
- Max nesting depth ≤ 2.
- Private by default; only `Renderer::new` and `Renderer::render` are public.
- No commented-out code.
- Self-documenting code: use the exact constant names `HEAD_SPAN`, `BODY_SPAN`, `FOOD_SPAN`, `EMPTY_SPAN`.
- All source code stays in `src/` per project structure rule.

---

## 13. Deliverable

This specification is saved as:

```text
C:\repo\rust-snake\.kilo\plans\20261005-snake-rendering-frontend-spec.md
```

It is intended for use by the architector in step 4.1b to produce the implementation plan. No code was implemented and no commit was made.
