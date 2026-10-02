# Simplification Plan — Phase 1B Group A: Terminal Primitives (Sub-step 4.3, code-simplifier)

- **Scope**: implementation of TODO Tasks 1, 2, 7 only (Group A) — `src/terminal.rs`, `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `src/lib.rs`.
- **Contract sources**: `.kilo/plans/20261001-phase1b-terminal-game-groupa.md` (implementation plan), `.kilo/plans/20261001-phase1b-terminal-game-groupa-frontend-spec.md` (SPEC).
- **Frozen and NOT revisited here**: decisions D1–D12, all public API signatures (SPEC §4.6/§5.2/§5.3/§6.3), glyph values, frame byte layout, `Duration::ZERO` by-value poll, Drop-guard semantics, module layout, file budgets.
- **Mode**: NO local toolchain / NO Docker. All steps are static text edits; NO cargo/build/test commands. Do NOT push.
- **Implementer level**: JUNIOR, 50% restriction. Every edit below is fully specified (exact old string → exact new string). If any old string does not match byte-for-byte, STOP and report to the caller. No other file may change.

---

## 1. Review Result — Rule & Duplication Audit (per file, at current HEAD state)

| File | Lines | ≤200 | bodies ≤50 | ≤2 params | depth ≤2 | single-section bools | dead code | duplication | verdict |
|---|---|---|---|---|---|---|---|---|---|
| `src/lib.rs` | 4 | PASS | n/a | n/a | n/a | n/a | none | none | **no change** |
| `src/terminal.rs` | 7 | PASS | n/a | n/a | n/a | n/a | none | none | **no change** |
| `src/terminal/renderer.rs` | 111 | PASS | PASS | PASS | PASS | PASS | none | **2 identical private border writers** + public items `Renderer`/`new` missing `///` docs | **2 edits (§3)** |
| `src/terminal/input.rs` | 60 | PASS | PASS | PASS | PASS | PASS | none | none (verbatim corrected §3.2 sketch) | **no change** |
| `src/terminal/lifecycle.rs` | 60 | PASS | PASS | PASS | PASS | PASS | none | none (`write_setup_commands` vs `write_restore_commands` emit different commands — not duplication) | **no change** |

Supporting audits already run (static):
- Forbidden-pattern grep over `src/terminal/` + `src/terminal.rs` + `src/lib.rs` (`collides|collision|score +=|SCORE_INCREMENT|opposite()|start_playing|advance_one_step|use crate::game::collision|println!|eprintln!|dbg!|execute!|unsafe|GameStatus`) → **0 hits**.
- Glyph audit: `'●'`, `'■'`, `'◆'` present; no full-width `｜ ＋ －` anywhere; ASCII `+ - |` only.
- Domain-call audit vs `src/game/*`: `snake().head() == cell` (Position: Copy + PartialEq), `segments().contains(&cell)`, `food().occupies(cell)`, `state.score() -> i32`, `0..HEIGHT` / `0..WIDTH` over `i32` consts — all signatures match the tree; compiles as written.
- Every private helper in all files is reachable (each is called at least once) — no dead code to remove.

## 2. Changes Deliberately NOT Proposed (out of simplification authority)

These were evaluated and rejected to stay inside the frozen plan/SPEC contract — do NOT "improve" them:

1. `input.rs` line 4 `use std::io::{self};` — cosmetic (could be `use std::io;`) but the import list is frozen verbatim in implementation plan §2.6; changing it would fail the 4.5a adherence review.
2. `input.rs` double press-check in the drain path (`is_arrow_key_press` → `is_key_press` + `is_arrow_key` → `map_key_event_to_direction` → `is_key_press` again) — this indirection is the plan's deliberate single-source-of-truth design (§2.6 "what is an arrow key") and the corrected sketch in §3.2. Behaviorally irrelevant cost; keep.
3. `renderer.rs` `cell_glyph` guard chain — frozen by decision **D3** (three single-condition guards, head→body→food→empty).
4. `renderer.rs` `rendered_border_row` push-loop vs. `str::repeat` — loop form is the §3.1 sketch and avoids an `as usize` cast from the `i32` `WIDTH`; keep.
5. `renderer.rs` `is_snake_head`/`is_snake_body`/`is_food` taking `&self` unused — §3.1 sketch shape; converting to free functions is churn the 4.5a reviewer would flag as deviation; keep.
6. `write_line` one-line wrapper — §2.5 required helper; encodes the `LINE_BREAK` CRLF constant (D1); keep.
7. `lifecycle.rs` setup/restore writers — required by §2.7 with D7/D8 error semantics; keep.

## 3. Approved Edits — ONLY `src/terminal/renderer.rs`

Apply the four edits below **in order**. Each old string occurs exactly once in the file (verified). No other line of any file may change.

### Edit R1 — merge the two identical border writers (duplication removal)

**R1a. Replace the duplicated pair** (currently the block starting `fn write_top_border` through the closing brace of `fn write_bottom_border`, including the blank line between them):

Old (exact):

```rust
    fn write_top_border(&mut self) -> io::Result<()> {
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }

    fn write_bottom_border(&mut self) -> io::Result<()> {
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }
```

New (exact):

```rust
    fn write_border_row(&mut self) -> io::Result<()> {
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }
```

**R1b. Update the three sequence lines inside `render`**:

Old (exact):

```rust
        self.write_top_border()?;
        self.write_board_rows(state)?;
        self.write_bottom_border()?;
```

New (exact):

```rust
        self.write_border_row()?;
        self.write_board_rows(state)?;
        self.write_border_row()?;
```

**Behavior note (why this is safe)**: the emitted byte stream is identical — the same `rendered_border_row()` string + CRLF is still written once before the board rows and once after them. Only a private helper is merged and renamed; `render`'s write order, `MoveTo(0, 0)` home, and single `flush()` are unchanged (D2 preserved). Plan §2.5 lists the old private names, but its hard-freeze applies to public signatures (SPEC §4.6); the 4.3 simplification step is the sanctioned place to collapse this exact 4-line duplication. This note is the audit trail for the 4.5a reviewer.

### Edit R2 — add the missing `///` docs on the two public items (comment-policy consistency)

Every other public item in Group A carries a one-line `///` (module doc rule in plan §4 A8 matrix: `///` on public items). `Renderer` and `Renderer::new` are the only ones without.

**R2a. Insert a doc line above the struct**:

Old (exact):

```rust
pub struct Renderer<W: Write> {
```

New (exact):

```rust
/// Full-frame renderer that writes the board frame and score line to an owned output.
pub struct Renderer<W: Write> {
```

**R2b. Insert a doc line above `new`**:

Old (exact):

```rust
    pub fn new(output: W) -> Renderer<W> {
```

New (exact):

```rust
    /// Create a renderer that writes its frames to `output`.
    pub fn new(output: W) -> Renderer<W> {
```

## 4. Implementer Verification Checklist (static only — run in order)

1. `rg -n "write_top_border|write_bottom_border" src/terminal/renderer.rs` → **0 hits**.
2. `rg -n "write_border_row" src/terminal/renderer.rs` → **exactly 3 hits** (1 definition + 2 calls in `render`).
3. `rg -n "^pub |    pub " src/terminal/renderer.rs` → public items are exactly `Renderer`, `new`, `render` (3 lines) — no visibility change was made.
4. Line count of `renderer.rs` → **108** (111 − 5 removed duplicate-fn block lines − 2 call sites renamed in place + 2 doc lines). Budget ≤200 PASS.
5. Diff is confined to `src/terminal/renderer.rs`: `git diff --stat` lists ONLY that file. `src/terminal/input.rs`, `src/terminal/lifecycle.rs`, `src/terminal.rs`, `src/lib.rs`, and everything else show zero changes.
6. Constants unchanged: `rg -n "const " src/terminal/renderer.rs` → same 9 constants, same glyph chars `● ■ ◆`, `LINE_BREAK: &str = "\r\n"` intact.
7. `render` body still reads, in order: `queue!(self.output, MoveTo(0, 0))?` → `write_border_row()?` → `write_board_rows(state)?` → `write_border_row()?` → `write_score_line(state)?` → `self.output.flush()`.
8. STOP conditions (any of these → do not proceed, report to caller): an old string in §3 not found or found multiple times; verification step 1–7 fails; any file other than `renderer.rs` touched.

## 5. Commit Guidance (timing owned by the caller's workflow)

Single logical change. Suggested message: `refactor: merge duplicate border writers and document renderer public items`. No push. No amend of the A4 commit unless the caller instructs.

## 6. Explicit Out of Scope for This Plan

- Groups B/C/D (game loop, main wiring, start/game-over screens, tests) — untouched by design at this stage.
- README/`.agent/project-structure.md`: no sync needed — neither document names the private helper functions (checked: structure line describes the renderer only as "full-frame board renderer over an io::Write output").
- Re-running plan Step A8 in full: the 4.5a reviewer owns the final adherence audit.
