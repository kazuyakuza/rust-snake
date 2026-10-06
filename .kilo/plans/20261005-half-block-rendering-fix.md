# Global Plan — 20261005 Half-Block Rendering Fix (board overflow)

TODO: `.agent/todos/20261005/20261005-todo-2.md` (Line Items format — one task)

## Pre-Analysis

### Root cause (verified against user screenshot, 2026-10-05)

- The current frame (Task 4 of the previous workflow) is **83 terminal rows × 162 cols**: top border + 80 board rows + bottom border + score line, each cell rendered as 2 columns.
- The user's console (classic conhost, "Original location of this command prompt") shows ~67 rows max on their screen (~1222 px vertical). Printing an 83-row frame with `\r\n` from `MoveTo(0,0)` scrolls the buffer every tick: the top of the frame (top border + snake at rows ~10–14 + most food positions) is **permanently above the visible window**. The screenshot shows exactly this: vertical side borders + bottom border + `Score: 1`, no snake/food/top-border. Gameplay itself works (score incremented).
- Screen-height arithmetic makes "resize the window" impossible: 83 rows at a readable font exceeds the user's physical screen. User decision (question tool): **Approach A — half-block packing**, keeping the 80×80 logical board.

### Chosen fix (encoded — no alternatives left open)

- Pack **two logical rows per terminal row**: terminal row `r` displays logical rows `2r` (top half) and `2r+1` (bottom half) via half-block glyphs:
  - both halves empty → space (default colors, no SGR)
  - only top occupied → `▀` (U+2580) with fg = top cell's color
  - only bottom occupied → `▄` (U+2584) with fg = bottom cell's color
  - both occupied, same kind → `█` (U+2588) with fg = that color
  - both occupied, different kinds → `█` with fg = top color, bg = bottom color (crossterm `SetColors`/`Colors` — the common case head-above-body is exactly this)
- Cell colors (classic snake palette, visible on black console bg): **head = Yellow**, **body = Green**, **food = Red**, empty = default/black.
- Board width back to **1 terminal column per logical cell** → border rows `+` + `-`×80 + `+` = 82 chars; verticals single `|`.
- Frame math: 1 (top border) + 40 packed rows (HEIGHT/2) + 1 (bottom border) + 1 (score) = **43 rows × 82 cols** — fits any console; each logical cell renders ≈9×8.5 px (visually square; keeps the vertical/horizontal speed parity won in Task 4).
- `HEIGHT = 80` is even → exactly 40 packed rows; encode a debug assertion or documented assumption (pick: documented module-doc note; no panic path — keep-it-simple).
- crossterm only (0.29 already a dependency; `SetColors`, `Color::{DarkYellow? no — Yellow, DarkGreen? no — Green, Red}` — encode exact color variants: `Color::Yellow`, `Color::Green`, `Color::Red`); no new dependencies; ratatui stays rejected.
- Renderer internals: per packed row, build with `queue!` of `SetColors` + `Print` per cell (or build String + explicit SGR) — 4.1a decides exact emission strategy; keep `MoveTo(0,0)` + single flush; methods ≤50 lines, depth ≤2, private members.

### Ripple

- `src/terminal/renderer.rs` — full rewrite of the row-drawing internals (span constants → half-block mapping + colors). `game_loop.rs`, `input.rs`, `lifecycle.rs`, `main.rs`, `src/game/**` untouched.
- `tests/terminal_modules.rs` — the 3 renderer-snapshot tests must be re-specified for the packed output (82-char borders, 43 lines, block-char/SGR expectations; keep headless `Vec<u8>`). `tests/gameplay_flow.rs` unaffected (asserts `Score: N` counts only — verify).
- Docs: README terminal-size note (162×83 / ~164×84 → 82×43, fits standard consoles), `docs/terminal-ui.md`, `.agent/project-info/architecture.md`, `.agent/project-info/tech.md` glyph lists → half-block + color scheme. `brief.md` untouched (prohibited).
- Version: patch bump **0.3.1 → 0.3.2** (bugfix).
- Verification: optional non-blocking VM Docker build (`sh -c "cd /rust-snake && docker compose run --rm build"`) + compile-only `cargo test --no-run` (no test execution; never stage `dist/`).

## Front-end classification

- The single TODO task is **front-end related** (terminal presentation layer) → 4.1a (frontend-specialist spec) + 4.5a (frontend-specialist verification) apply.

## Execution order

- Step 2: Git Feature Branch Setup => implementer (`fix/half-block-rendering` from `main`)
- Step 3: Version Update => implementer (0.3.1 → 0.3.2, commit `chore: bump version to 0.3.2`)
- Task 1 (half-block rendering fix, front-end):
  - 4.1a Front-end Technical Spec => frontend-specialist (`.kilo/plans/20261005-half-block-frontend-spec.md`)
  - 4.1b Implementation Plan => architector (`.kilo/plans/20261005-half-block-rendering.md`, uses 4.1a spec)
  - 4.2 Implementation => implementer (renderer rewrite + tests; commit `fix: pack board rows with half-block glyphs to fit short consoles`)
  - 4.3 Review & Simplification => code-reviewer + code-simplifier (+fixes via implementer)
  - 4.4 Documentation => docs-specialist (README/docs/terminal-ui/architecture/tech; commit `docs: update rendering docs for half-block packed board`)
  - 4.5a Front-end Verification => frontend-specialist (report vs 4.1a spec)
  - 4.5b Overall Plan Adherence => architector (report)
  - 4.6 Task Completion => implementer (`[DONE]` on TODO line; commit plan/report artifacts + context bullet)
- Step 5: TODO File Completion => implementer (rename `-DONE`, cleanup, merge `fix/half-block-rendering` → `main`, push `origin` only)
- Step 6: Finish => Planner (context.md closing update; short resume)

## Per-task constraints (all sub-agents)

- Sub-Task Prompt Requirements verbatim (critical-workflow step 4).
- Single source of truth: board dims stay in `src/game/state.rs` (80×80 unchanged — do NOT touch).
- Do NOT modify: `src/game/**`, `game_loop.rs` (tick timing), `input.rs`, `lifecycle.rs`, `main.rs`, `Cargo.toml` deps, `brief.md`, TODO file (except 4.6).
- Headless tests only (`Vec<u8>`); no real-terminal tests; no test execution (compile-only checks allowed in VM).
- gitignore compliance before every commit; `dist/`/`target/` never staged; push restricted to Step 5.
- Any ambiguity → question back to Planner; never assume.
