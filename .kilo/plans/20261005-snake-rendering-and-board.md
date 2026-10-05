# Global Plan — 20261005 Snake Improvements (board square, rendering, VM context)

TODO: `.agent/todos/20261005/20261005-todo-1.md` (Line Items format — each line is a task)

## Pre-Analysis (global)

### Current state (verified 2026-10-05)

- Git: branch `main`, clean tree, up to date with `origin/main`, HEAD = `c2930d5` line of history; latest tracked TODO `.agent/todos/20261004/20261004-todo-1-DONE.md`.
- Version: `Cargo.toml` → `0.3.0` (semver: patch-level gameplay/rendering fixes → `0.3.1` expected at step 3).
- Stack: Rust 2021 (package `snake`), deps `rand 0.8`, `crossterm 0.29`. Build: `docker compose run --rm build` → `dist/snake.exe` (pinned `rust:1.98.1-slim-bookworm` + mingw-w64). No host Rust toolchain.
- Alpine VM MCP (`alpine-vm`): **verified by Planner** — `vm_status` → `vmState: running | sshReachable: true` (127.0.0.1:3022); `vm_run_command("ls /rust-snake")` → same file list as the host repo (`AGENTS.md`, `Cargo.lock`, `Cargo.toml`, `Dockerfile`, `LICENSE`, `README.md`, `compose.yaml`, `dist`, `docs`, `src`, `tests`). Conclusion: the VM reflects this same working directory (host→VM shared folder). Docker availability inside `/rust-snake` is therefore granted. **This registration is a documentation task** (project context update), NOT a code task.
- Code touchpoints for the board/rendering tasks:
  - `src/game/state.rs` — `pub const WIDTH: i32 = 40;` / `pub const HEIGHT: i32 = 25;` and playable-bounds predicates (`is_inside_board`, `is_within_bounds`). Single source of truth for board dimensions; collision and food placement derive from it.
  - `src/game/setup.rs` — `INITIAL_SNAKE_HEAD {x:10,y:12}`, `BODY_AHEAD {9,12}`, `BODY_BEHIND {8,12}`, `INITIAL_FOOD_POSITION {20,12}` — all remain valid inside an 80×80 playfield.
  - `src/terminal/renderer.rs` — full-frame renderer: border rows (`+`/`-`/`|`), per-cell glyphs (head `●` U+25CF, body `■` U+25A0, food `◆` U+25C6), one `char` per logical cell, score line; queued via crossterm at `MoveTo(0,0)` and flushed.
  - `src/terminal/game_loop.rs` — fixed `TICK_DURATION = 120ms`; uniform per-cell movement in the domain (brief §7). Domain speed is already constant; the reported "vertical faster" issue is a **rendering/aspect perception** problem, not a domain logic bug.
  - `src/main.rs` — screen wiring; unaffected except size-related revalidation of the frame.

### Root-cause analysis (from user's screenshot)

1. **Disjoint squares**: a snake body rendered as a single 1-column glyph per cell (`■`) leaves visual gaps/fragmentation on Windows terminals; adjacent cells don't connect. The screenshot also shows a compact tall body column — rendering is width-1 per logical cell.
2. **Vertical faster than horizontal**: on Windows terminal, a character cell is roughly 2× taller than wide (pixel aspect 1:2). The domain moves exactly 1 cell/tick in all directions (checked `advance_one_step` — uniform `offset()`), so on-screen: vertically 1 cell ≈ 16 px, horizontally ≈ 8 px, hence vertical ~2× faster visually. The fix is **visual**: render each logical cell as **2 terminal columns wide** so horizontal pixel-step ≈ vertical pixel-step.
3. Therefore T4 (rendering) and T3 (board size) are coupled in the visible result; but the code changes are orthogonal (domain constants vs. terminal glyph mapping).

### Technical & architecture decisions (global)

- Stay with **crossterm** (already a dependency, handles raw mode / alternate screen / cursor / events / queueing). No ratatui: the game renders a full-frame ASCII board itself; ratatui would add a widget framework/middleman for zero required functionality (keep-it-simple, brief §19). crossterm research must confirm API surfaces used: `queue!`, `MoveTo`, `Clear(ClearType::All)`, `event::poll`/`event::read`, `EnterAlternateScreen`, raw mode — all current usages.
- Render cells **double-width horizontally** (each logical cell maps to 2 terminal columns) — standard snake-in-terminal approach that fixes both visual speed parity and glyph continuity. Glyph choice per cell (e.g. glyph + trailing space, doubled glyph `██`) is a 4.1b decision, encoding the exact final mapping in the plan.
- Board: `WIDTH = 80; HEIGHT = 80` (double + square, exactly per user request). Ripple: tests that reference constants, docs (`architecture.md`, README), and the terminal-size sanity note: a full 80 columns × 82 rows frame (80+2 border, +score line) requires a Windows terminal at least ~82×83 cells — document it; not a blocker.
- VM context registration (T1): insert into `.agent/project-info/context.md` only (Planner's closing duty per `instructions.md`, assigned to docs step for execution, Planner reviews). Never touch `brief.md` (source of truth); board-size recommendation in brief §5 is "recommended", not a hard constraint, and stays unmodified.
- Constraints encoded from rules: max 200 lines/file, methods ≤ 50 lines, private-by-default, no commented code, markdown edits limited to this plan/TODO/docs.

## Front-end classification

- T1 (VM registration): not front-end.
- T2 (crossterm research): not front-end (library research).
- T3 (board constants): not front-end (domain constants + docs).
- T4 (rendering/velocity): **front-end related** (terminal presentation layer) → 4.1a + 4.5a apply to T4 only.

## Global execution order

- Step 2: Git Feature Branch Setup => implementer (`feat/terminal-rendering-and-board`)
- Step 3: Version Update => implementer (0.3.0 → 0.3.1)
- Task 1 (VM context registration) — **grouped with Task 2** (extremely short, related docs/research work):
  - 4.1b Analysis & Planning (T1 + T2) => architector
  - 4.2 Implementation (T1 + T2): update `.agent/project-info/context.md` with VM MCP availability + research note => implementer
  - 4.3 Review & Simplification => code-reviewer + code-simplifier
  - 4.4 Documentation => docs-specialist
  - 4.5b Overall Plan Adherence => architector
  - 4.6 Task Completion => implementer (`[DONE]` on lines 1–2)
- Task 3 (square board 80×80):
  - 4.1b => architector (constants, tests ripple, docs ripple, terminal-size note)
  - 4.2 => implementer
  - 4.3 => code-reviewer + code-simplifier (+fixes)
  - 4.4 => docs-specialist
  - 4.5b => architector
  - 4.6 => implementer (`[DONE]` line 3)
- Task 4 (visualization + velocity, front-end):
  - 4.1a Front-end Technical Spec => frontend-specialist (double-width cell mapping; contiguous glyphs; FallEyes screenshot issues resolved; crossterm-only)
  - 4.1b Implementation Plan => architector (uses 4.1a spec path)
  - 4.2 => implementer (renderer changes; test updates; screenshots-equivalent buffer assertions)
  - 4.3 => code-reviewer + code-simplifier (+fixes)
  - 4.4 => docs-specialist (README + docs/ updates; screenshots-in-docs note)
  - 4.5a Front-end Verification => frontend-specialist (spec vs implementation report)
  - 4.5b Overall Plan Adherence => architector
  - 4.6 => implementer (`[DONE]` line 4)
- Step 5: TODO File Completion => implementer (rename `-DONE`, cleanup, merge `feat/...` → `main`, push `origin` only)
- Step 6: Finish => Planner (context.md closing update incl. VM registration cross-check; short resume)

## Per-task constraints (all sub-agents)

- Follow Sub-Task Prompt Requirements verbatim (critical-workflow step 4).
- Respect single-source-of-truth: board dims only in `src/game/state.rs`; renderer reads them.
- Tests must stay headless (no real terminal); run/no-run status unchanged (authored-only until user runs cargo test in Docker).
- Do not modify `brief.md`.
- `dist/` never committed; `target/` never staged; gitignore compliance before every commit.
- Any ambiguity → return question to Planner; never assume.
