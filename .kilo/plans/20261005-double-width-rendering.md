# Implementation Plan — Task 4: Double-Width Snake Rendering (visualization + velocity fix)

**Workflow step:** 4.1b Implementation Plan (Task 4 of `.agent/todos/20261005/20261005-todo-1.md`, line 4)
**Authoritative front-end input:** `.kilo/plans/20261005-snake-rendering-frontend-spec.md` (step 4.1a) — every decision in it is FINAL; this plan only encodes it as concrete edits. No open alternatives.
**Target implementer:** JUNIOR developer under 50% restriction (4.2 / 4.4 / 4.6 sub-agents).
**Workspace:** `C:\repo\rust-snake`, branch `feat/terminal-rendering-and-board`.

---

## 1. Preconditions (verify before any edit)

| # | Check | Expected | Verified by research |
| --- | --- | --- | --- |
| 1.1 | Branch | `feat/terminal-rendering-and-board` | `git status` → "On branch feat/terminal-rendering-and-board" |
| 1.2 | HEAD | `1b471eb` "docs: mark square-board task done" (on top of `381c34c`, `0d46b5b`, `72dca22` square-board chain) | `git log --oneline -8` |
| 1.3 | Working tree | Clean except ONE untracked file: `.kilo/plans/20261005-snake-rendering-frontend-spec.md` (the 4.1a spec). This plan file `.kilo/plans/20261005-double-width-rendering.md` will also be untracked after creation | `git status` |
| 1.4 | Version | `Cargo.toml` = `0.3.1`, already committed (`78c9d61`). **NO version bump in this task** | `git log --oneline -4 -- Cargo.toml` |
| 1.5 | Board constants | `src/game/state.rs` lines 18–19: `WIDTH: i32 = 80`, `HEIGHT: i32 = 80` — read-only, do not touch | direct read |
| 1.6 | Initial state | `src/game/setup.rs`: head (10,12), body (9,12), body (8,12), food (20,12), direction Right, score 0 → 1 head cell + 2 body cells + 1 food cell on screen | direct read |
| 1.7 | Gitignore | `dist/` and `target/` are ignored (`.gitignore` lines 31, 34) — never stage them | `.gitignore` read |
| 1.8 | Push | Restricted to Critical Workflow step 5. This task NEVER pushes | caller instruction |

Files to touch (complete list, grep-verified):

- `src/terminal/renderer.rs` (108 lines — code change, 4.2)
- `tests/terminal_modules.rs` (189 lines — 3 test functions updated, 4.2)
- `README.md` line 27 (terminal-size note, 4.4)
- `docs/terminal-ui.md` lines 61–62 (glyph list; grep found NO frame-size mention there, 4.4)
- `.agent/project-info/architecture.md` lines 75 and 102 (glyph lists; grep found NO frame-size mention there, 4.4)
- `.agent/project-info/tech.md` line 52 (glyph list in the resolved-decisions bullet — grep-discovered, 4.4)
- `.agent/project-info/context.md` (one NEW Recent Changes bullet, 4.6)
- `.agent/todos/20261005/20261005-todo-1.md` (append ` [DONE]` to line 4, 4.6)

Files NOT to touch (grep-verified rationale):

- `tests/gameplay_flow.rs` — checked: its only renderer-output assertions are `Score: N` counts (lines 98–99, 127) and the score line format is unchanged; no glyph or frame-size-dependent assertions exist. **Zero edits.**
- `src/game/*`, `src/terminal/{game_loop,input,lifecycle}.rs`, `src/main.rs`, `src/lib.rs`, `Cargo.toml`, `compose.yaml`, `Dockerfile`, `brief.md` — out of scope (§4 of spec + caller).

---

## 2. Root cause (why the change is renderer-only)

One logical cell = 1 terminal column today. Windows terminal cells are ~2× taller than wide, so a 1-cell vertical step covers ~2× the pixels of a 1-cell horizontal step (the "velocity error" perception), and single-column body glyphs (`■`) leave visible gaps (the "disjoint squares"). Mapping every logical cell to **2 terminal columns** makes horizontal/vertical pixel-steps ≈ equal and makes `██` spans contiguous. Domain and tick timing are correct and untouched (spec §1, §10).

---

## 3. Step allocation — which file is edited in WHICH step (anti-double-editing decision)

DECISION (resolves the caller's open point; spec §7 itself assigns docs to 4.4):

| Workflow step | Files edited | Commit |
| --- | --- | --- |
| **4.2 Implementation** | `src/terminal/renderer.rs`, `tests/terminal_modules.rs` | Commit 1: `feat: draw snake cells double-width for visual speed parity` |
| **4.3 Review/Simplify** | none (review-only; fixes via separate plan if needed) | — |
| **4.4 Documentation** | `README.md`, `docs/terminal-ui.md`, `.agent/project-info/architecture.md`, `.agent/project-info/tech.md` | Commit 2: `docs: update rendering docs for double-width cells` |
| **4.5a/4.5b Verification** | none (report-only) | — |
| **4.6 Task Completion** | `.agent/project-info/context.md` (new bullet only), `.agent/todos/20261005/20261005-todo-1.md` (`[DONE]` on line 4 only) | Commit 3: `docs: mark double-width rendering task done` |

Rules: each file is edited in exactly ONE step. `.kilo/plans/*.md` files (the 4.1a spec and this 4.1b plan) are NEVER staged by the implementer — plan/spec commits are the Planner's own pattern (cf. Task 3 commit `381c34c`). `context.md` historical bullets (e.g. the Phase 1B glyph log at line 48) are history — NEVER rewritten; only the new 4.6 bullet describes this change.

---

## 4. Out of scope (restated — implementer must NOT touch)

`src/game/` (all domain), `src/terminal/game_loop.rs` timing (`TICK_DURATION = 120 ms`), `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `src/main.rs` screens, `src/lib.rs`, new dependencies (crossterm 0.29 only), crossterm colors, `ratatui`, `brief.md`, `Cargo.toml` version, `tests/gameplay_flow.rs`, git push.

---

## 5. STEP 4.2 — Exact code change: `src/terminal/renderer.rs`

Five surgical edits. Everything else in the file (module doc comment lines 1–2, imports line 4–9, `Renderer` struct, `new`, `render`, `write_border_row`, `write_board_rows`, `write_score_line`, `write_line`, `is_snake_head`, `is_snake_body`, `is_food`) stays byte-identical. The file is UTF-8 — copy the glyph characters from the spec/plan text, do not retype from memory (mojibake risk).

### 5.1 Edit A — replace the four cell-glyph constants (current lines 11–14)

DELETE exactly:
```rust
const HEAD_GLYPH: char = '●';
const BODY_GLYPH: char = '■';
const FOOD_GLYPH: char = '◆';
const EMPTY_GLYPH: char = ' ';
```

INSERT in its place (order and type annotation exactly as spec §4 code block):
```rust
const EMPTY_SPAN: &'static str = "  ";
const BODY_SPAN: &'static str = "██";
const HEAD_SPAN: &'static str = "●●";
const FOOD_SPAN: &'static str = "◆◆";
```

Span meanings (spec §4): empty = 2×U+0020 SPACE; body = 2×U+2588 FULL BLOCK; head = 2×U+25CF BLACK CIRCLE; food = 2×U+25C6 BLACK DIAMOND. Border constants (lines 15–17 `CORNER_GLYPH '+'`, `HORIZONTAL_GLYPH '-'`, `VERTICAL_GLYPH '|'`) and `SCORE_PREFIX`/`LINE_BREAK` (lines 18–19) remain unchanged.

### 5.2 Edit B — `render_board_row` switches `push` → `push_str` (current lines 64–72)

New body (unchanged signature):
```rust
    fn render_board_row(&self, state: &GameState, row: i32) -> String {
        let mut row_text = String::new();
        row_text.push(VERTICAL_GLYPH);
        for column in 0..WIDTH {
            row_text.push_str(self.cell_span(state, Position { x: column, y: row }));
        }
        row_text.push(VERTICAL_GLYPH);
        row_text
    }
```

### 5.3 Edit C — rename `cell_glyph` → `cell_span`, `char` → `&'static str` (current lines 74–85)

```rust
    fn cell_span(&self, state: &GameState, cell: Position) -> &'static str {
        if self.is_snake_head(state, cell) {
            return HEAD_SPAN;
        }
        if self.is_snake_body(state, cell) {
            return BODY_SPAN;
        }
        if self.is_food(state, cell) {
            return FOOD_SPAN;
        }
        EMPTY_SPAN
    }
```

Precedence stays head → body → food → empty (the head is a segment, so the head check must come first). The three `is_*` helpers are untouched.

### 5.4 Edit D — `rendered_border_row` doubles the horizontal run (current lines 100–108)

```rust
fn rendered_border_row() -> String {
    let mut border_row = String::new();
    border_row.push(CORNER_GLYPH);
    for _ in 0..(WIDTH * 2) {
        border_row.push(HORIZONTAL_GLYPH);
    }
    border_row.push(CORNER_GLYPH);
    border_row
}
```

Result for `WIDTH = 80`: `+` + 160×`-` + `+` = 162 chars. Corners single `+`; board-row verticals stay single `|` (spec §5 — no doubling of border glyphs).

### 5.5 Edit E — nothing else

`render()` sequence (`queue!(MoveTo(0,0))` → border → board rows → border → score → `flush()`), `write_line` (`"{content}{LINE_BREAK}"`), `LINE_BREAK = "\r\n"`, score line format — all unchanged. No per-cell `MoveTo`, no colors, no cursor gymnastics (spec §8). Old constants are fully removed — no commented-out code (rule + spec §8).

### 5.6 Constraint self-check (after edits)

- File line count ≤ 200 (expect ~108, unchanged).
- Longest method body well under 50 lines; max nesting depth ≤ 2; ≤ 2 params per method (all unchanged shapes).
- No new comments added; no commented-out code.
- Only `pub` items remain `Renderer::new` and `Renderer::render`.

---

## 6. STEP 4.2 — Exact test updates: `tests/terminal_modules.rs`

Grep verification result: exactly **3** of the 11 test functions assert renderer bytes; the other 8 (`arrow_presses_map_to_the_four_directions`, `non_arrow_press_maps_to_none`, `key_release_maps_to_none`, `tick_applies_directions_before_one_step_and_rejects_reversal`, `tick_advances_exactly_one_step_per_call`, `tick_returns_game_over_when_the_head_exits_the_boundary`, `tick_is_a_no_move_before_playing`, `tick_returns_the_new_status_playing`) never read the buffer and get **zero edits**. The `count_occurrences` helper (lines 23–28) stays unchanged (byte-window counting — works as-is). Tests remain headless over `Vec<u8>`. NO new test functions are added — spec §3 says "updated assertions only; no test structure changes"; spec §9.4's row-length sanity is explicitly optional and its required-minimum is already covered by the byte counts below (DECISION: do not add the optional row-length test).

### 6.1 `render_writes_the_full_frame_into_the_buffer` (lines 141–156)

Current assertion block (lines 151–155):
```rust
    assert_eq!(count_occurrences(&buffer, "+"), 4);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
    assert_eq!(count_occurrences(&buffer, "●"), 1);
    assert_eq!(count_occurrences(&buffer, "■"), 2);
    assert_eq!(count_occurrences(&buffer, "◆"), 1);
```

REPLACE with exactly (spec §9.1, verbatim including comments):
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

Math proven by the initial state (§1.6): 1 head cell → one `"●●"` span = 2×`●`; 2 body cells → 2×`"██"` = 4×`█`; 1 food cell → `"◆◆"` = 2×`◆`. `■` (U+25A0) is no longer emitted anywhere.

### 6.2 `tick_renders_one_consistent_frame_of_glyphs` (lines 158–173)

Current (lines 169–172):
```rust
    assert_eq!(count_occurrences(&buffer, "●"), 1);
    assert_eq!(count_occurrences(&buffer, "■"), 2);
    assert_eq!(count_occurrences(&buffer, "◆"), 1);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
```

REPLACE with exactly (spec §9.2):
```rust
    assert_eq!(count_occurrences(&buffer, "●"), 2);
    assert_eq!(count_occurrences(&buffer, "█"), 4);
    assert_eq!(count_occurrences(&buffer, "◆"), 2);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
```

(One tick Right: head (11,12), body (10,12)+(9,12) — still 1 head, 2 body, food uneaten at (20,12).)

### 6.3 `two_renders_reuse_the_frame_without_scrolling` (lines 175–189)

Current (lines 186–188):
```rust
    assert_eq!(count_occurrences(&buffer, "+"), 8);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
    assert_eq!(count_occurrences(&buffer, "●"), 2);
```

REPLACE with exactly (spec §9.3 — keep the two first lines' values, change `●` 2→4, ADD the `\r\n` line):
```rust
    assert_eq!(count_occurrences(&buffer, "+"), 8);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
    assert_eq!(count_occurrences(&buffer, "●"), 4);     // 2 frames * 2 head chars
    assert_eq!(count_occurrences(&buffer, "\r\n"), 166); // 83 lines * 2 frames
```

### 6.4 Test-file self-check

`grep "■" tests/terminal_modules.rs` → 0 matches. `grep "cell_glyph"` → 0 matches project-wide. Imports at lines 1–9 unchanged (`HEIGHT` import stays used by the boundary test).

---

## 7. STEP 4.2 — Verification (optional, non-blocking) + commit 1

### 7.1 Optional VM Docker build (caller item 5)

- Tool: `alpine-vm` MCP. Call `alpine-vm_vm_status` FIRST; only if running+SSH-reachable, run:
  `sh -c "cd /rust-snake && docker compose run --rm build"` (allowlist prefix `sh` — permitted).
- Purpose: prove compilation after the renderer change. Release build only (`compose.yaml` `build` service). Record the exit code in the step summary. `cargo test` is NEVER run (build command does not run tests). If the VM/tooling is unavailable, report it and continue — NON-BLOCKING.
- `dist/snake.exe` may be regenerated by the build — `dist/` is gitignored (`.gitignore` line 31); NEVER stage it.

### 7.2 Commit 1 (end of 4.2)

1. Read `.gitignore`; run `git status`; stage ONLY: `src/terminal/renderer.rs`, `tests/terminal_modules.rs` (never `.kilo/plans/*`, never `dist/`).
2. `git commit -m "feat: draw snake cells double-width for visual speed parity"`
3. Verify with `git status` (clean except the two untracked plan/spec .md files) and `git log --oneline -2`.

---

## 8. STEP 4.4 — Exact documentation edits + commit 2

Four files, exact replacements (single edit each; no other lines change).

### 8.1 `README.md` line 27

Current:
```
- Terminal size: with the 80 x 80 board the full frame spans about 82 columns x 83 rows (borders + score line); use a terminal window of at least ~84 x 84 character cells, and enlarge or resize the window (e.g. in Windows Terminal or the classic console) if the frame looks cut off.
```
New (spec §7 wording — frame 162×83, window ≥ ~164×84):
```
- Terminal size: with the 80 x 80 board the full frame spans about 162 columns x 83 rows (each board cell renders two columns wide; borders + score line); use a terminal window of at least ~164 x 84 character cells, and enlarge or resize the window (e.g. in Windows Terminal or the classic console) if the frame looks cut off.
```
(Rest of `## Game Rules & Controls` unchanged; the Terminal UI section's Renderer bullet at line 40 names no glyphs — leave it.)

### 8.2 `docs/terminal-ui.md` lines 61–62

Current:
```
  - Glyphs: head `●`, body `■`, food `◆`, borders `+ - |`, score line
    `Score: <n>`.
```
New:
```
  - Glyphs: each cell is a two-column span — head `●●`, body `██`, food `◆◆`,
    empty `  `; borders stay single-character `+ - |` (border row is `-` × (2×WIDTH));
    score line `Score: <n>`.
```
No frame-size mention exists in this file — none added beyond the glyph line (grep-verified).

### 8.3 `.agent/project-info/architecture.md` — two lines

Line 75 (Terminal UI layer bullet), current fragment `per-cell glyphs (head `●` U+25CF, body `■` U+25A0, food `◆` U+25C6, empty ` `)` → new fragment:
```
per-cell two-column glyph spans (head `●●` U+25CF×2, body `██` U+2588×2, food `◆◆` U+25C6×2, empty two spaces)
```

Line 102 (Game Layout section), current:
```
- Cell glyphs (decided in Phase 1B): snake head `●` (U+25CF), snake body `■` (U+25A0), food `◆` (U+25C6), empty space ` `; score line rendered as `Score: N` (brief §5, §10, §13).
```
New:
```
- Cell glyphs (double-width since 2026-10-05, Task 4): each logical cell renders as a two-column span — snake head `●●` (U+25CF ×2), snake body `██` (U+2588 ×2), food `◆◆` (U+25C6 ×2), empty space two spaces; borders stay single-character (`+` corners, `-` ×(2×WIDTH) horizontal rows, `|` verticals); score line rendered as `Score: N` (brief §5, §10, §13).
```
(architecture.md has no frame-size note — grep-verified; line 101 board bullet stays 80×80.)

### 8.4 `.agent/project-info/tech.md` line 52 (grep-discovered glyph list)

Current fragment inside the resolved bullet: `head `●` (U+25CF), body `■` (U+25A0), food `◆` (U+25C6), empty ` ` (U+0020); ASCII borders `+` corners, `-` horizontal, `|` vertical; score line `Score: N`.` → new fragment:
```
head `●●` (U+25CF ×2), body `██` (U+2588 ×2), food `◆◆` (U+25C6 ×2), empty two spaces (U+0020 ×2); ASCII borders `+` corners, `-` ×(2×WIDTH), `|` verticals; score line `Score: N`.
```
Keep the rest of the bullet (crossterm note) unchanged; adjust only the lead-in if needed to remain grammatical, e.g. "**resolved in Phase 1B, updated 2026-10-05 (Task 4 double-width spans)** (see `src/terminal/renderer.rs`):".

### 8.5 Commit 2

1. `git status` + `.gitignore` check; stage ONLY the four docs files.
2. `git commit -m "docs: update rendering docs for double-width cells"`

---

## 9. STEP 4.6 — Completion edits + commit 3

1. `.agent/todos/20261005/20261005-todo-1.md` line 4: append ` [DONE]` at end of the line (line-item format). Preserve every other byte of the file (overwrite-prevention rule). Result: line 4 ends `...visually equal vertical/horizontal speed) [DONE]`.
2. `.agent/project-info/context.md`: append ONE new bullet at the END of `## Recent Changes` (after the square-board bullet, line 38), describing: double-width rendering done on `feat/terminal-rendering-and-board` — per-cell 2-column spans (`●●`/`██`/`◆◆`/2 spaces), border rows `+` + `-`×160 + `+` (162 cols), frame 162×83, README/docs terminal-size note → ~164×84, tests doubled counts, commits 1–2 hashes, build verification exit code. Do NOT touch the historical Phase 1B bullet (line 48) or any other section.
3. `git status` + `.gitignore` check; stage ONLY `context.md` + the TODO file; `git commit -m "docs: mark double-width rendering task done"`.

---

## 10. Reviewer checkpoints (4.3 code-reviewer / code-simplifier, 4.5a/4.5b)

- `grep -c "GLYPH" src/terminal/renderer.rs` → only `CORNER_GLYPH`, `HORIZONTAL_GLYPH`, `VERTICAL_GLYPH` remain; `HEAD_GLYPH`/`BODY_GLYPH`/`FOOD_SPAN`-family naming exactly `HEAD_SPAN`, `BODY_SPAN`, `FOOD_SPAN`, `EMPTY_SPAN`.
- Border row length in tests: `-` = 320/frame, `|` = 160/frame, frame = 162×83, `\r\n` = 83/frame.
- No `■`, no colors/ANSI SGR, no per-cell `MoveTo`, `queue!` + single `flush` preserved.
- No edits outside §3's allocation table.

---

## 11. Acceptance mapping (TODO line 4 ↔ spec ↔ plan)

| Symptom | Spec fix | Plan step |
| --- | --- | --- |
| Disjoint small squares on a vertical run | `BODY_SPAN = "██"` contiguous blocks (spec §11.1) | §5.1, §5.3 |
| Vertical faster than horizontal | 2 columns per cell ≈ equalizes ~2:1 cell aspect (spec §11.2) | §5.1–§5.4 |
| Head/food distinct | `●●` / `◆◆` (spec §11.3–11.4) | §5.1 |
| Frame fits | 162×83; docs note ≥ ~164×84 (spec §11.5, §7) | §8.1 |
| Velocity/domain untouched | out of scope (spec §10) | §4 |

Plan self-check vs TODO line 4 and spec §1–§13: every spec decision (spans, border math, test counts, docs note, out-of-scope list) is encoded above with no alternatives left open. Complete.
