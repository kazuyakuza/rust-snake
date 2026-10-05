# Implementation Plan — Tasks 1+2: VM Context Registration & Crossterm Research

> Workflow step 4.1b (grouped tasks 1 & 2) — produced by architector, 2026-10-05.
> TODO: `.agent/todos/20261005/20261005-todo-1.md` (lines 1–2 only).
> Global plan: `.kilo/plans/20261005-snake-rendering-and-board.md` (Pre-Analysis section).
> Branch: `feat/terminal-rendering-and-board` (already created in step 2; HEAD `78c9d61` "chore: bump version to 0.3.1").
> DO-NOT-CROSS: tasks 3 (board 80×80) and 4 (renderer/velocity) have their own 4.1–4.6 cycles — NOT planned here.

---

## Scope

- **Pure documentation + research task.** NO product code changes: no edits to `src/**`, `tests/**`, `Cargo.toml`, `compose.yaml`, `Dockerfile`, or `brief.md`.
- Exactly two repo files are modified plus this plan file is committed:
  1. `.agent/project-info/context.md` — Task 1: register the verified alpine-vm MCP availability.
  2. `.agent/project-info/tech.md` — Task 2: encode the terminal-rendering-library research outcome under **Pending Decisions**.
- Every insertion is an **addition only**; no existing line may be deleted or rewritten (overwrite-prevention rule).

---

## High-Level Approach

Both TODO lines ask for facts that already exist (Planner verified the VM facts on 2026-10-05; the crossterm dependency and its usage are in the code) or that this planning step established via `context7` research (recorded in the "Crossterm Research Findings" section below). Therefore the implementer's job is a mechanical, verifiable insertion of three verbatim text blocks (one bullet into `context.md`, two bullets into `tech.md`) followed by one git commit.

---

## Crossterm Research Findings (Task 2 — established by this 4.1b step)

Research method: `context7` MCP — `resolve-library-id "crossterm"` → `/crossterm-rs/crossterm` (2 doc queries); `resolve-library-id "ratatui"` → `/ratatui/ratatui` (1 doc query). Codebase cross-check: `Cargo.toml` (crossterm = "0.29") and grep of `src/`.

### A. crossterm fits a terminal Snake's needs (character-cell drawing + cursor positioning)

- crossterm is a **pure-Rust, cross-platform, character-cell-based** terminal manipulation library: cursor control (move/position/hide/show), terminal commands (clear, alternate screen, raw mode), and an event Poll/read API (README feature list, confirmed in `/crossterm-rs/crossterm` docs).
- A full-screen render in crossterm is done by **queueing commands** into a `Write` handle and flushing once: `queue!(output, MoveTo(0, 0), Clear(ClearType::All), …print…)` then `output.flush()`. The feature list explicitly guarantees "Full control over writing and flushing output buffer" — exactly the pattern `src/terminal/renderer.rs` already uses (`MoveTo(0, 0)` at line 34) and `src/main.rs` screens use (`queue!(output, Clear(ClearType::All), MoveTo(0, 0))` at lines 45 and 63).
- **Double-width glyphs:** crossterm exposes **no display-width measurement API and no glyph-width notes** — cursor positioning is strictly per terminal cell (`MoveTo(col, row)`), and how a Unicode glyph (e.g. `■` U+25A0 vs. `█` U+2588) visually fills a cell is decided by the terminal emulator. Consequence (for the record; implementation belongs to Task 4): achieving a contiguous, aspect-corrected snake is a **renderer-side glyph mapping decision** (each logical cell mapped to 2 terminal columns), and crossterm's `MoveTo` supports it because the renderer writes the doubled glyphs at computed columns — no library feature is missing.

### B. crossterm 0.29 API-stability confirmation (every API the project uses)

| API used by project | File/usage (verified by grep) | Research status |
| --- | --- | --- |
| `queue!` macro + `flush()` | `renderer.rs:6,34`, `lifecycle.rs:41,53`, `main.rs:45,63` | Confirmed as the documented queued-write mechanism (control over flush; used across its examples) |
| `cursor::MoveTo` | `renderer.rs:34`, `main.rs:7` | Confirmed: cursor "Set/get the cursor position", command form usable in `queue!` |
| `terminal::Clear` + `ClearType::All` | `main.rs:45,63` | Confirmed: "Clear (all lines, …)" — `ClearType::All` is the all-lines variant |
| `terminal::enable_raw_mode` / `disable_raw_mode` | `lifecycle.rs:20,22,31` | Confirmed (wiki snippet; replaces old `RawScreen`) |
| `terminal::EnterAlternateScreen` / `LeaveAlternateScreen` | `lifecycle.rs:8,41,53` | Confirmed (source: CSI `?1049h/?1049l`, Windows ScreenBuffer fallback) |
| `cursor::Hide` / `cursor::Show` | `lifecycle.rs:41,53` | Confirmed: cursor feature list "Hide/show the cursor" |
| `event::poll` (+ `event::read`) | `input.rs:35,66` (`Duration::ZERO` non-blocking drain) | Confirmed: "Poll/read API" event feature |
| `event::{KeyCode, KeyEvent, KeyEventKind}` | `input.rs:7` | Confirmed: key events with advanced modifier support; `KeyEventKind` (press/release/repeat) is the crossterm mechanism for distinguishing event kinds on terminals that emit extras (Windows) — the project already filters on the press kind |

Conclusion: **nothing the project uses is deprecated or missing in 0.29; no upgrade and no change of library is required.**

### C. ratatui evaluated — rejected (keep-it-simple, brief §19)

- Fact from `/ratatui/ratatui` docs: ratatui is an **immediate-mode widget framework** that sits ON TOP of a Backend — the crossterm backend still performs all terminal manipulation, and apps go through ratatui's `Terminal`/`Frame`/widget model (`terminal.draw(render)`, `frame.render_widget(...)`), plus higher-level setup helpers (`ratatui::init()` / `ratatui::restore()`).
- This game already renders **its own full-frame ASCII board** (border rows, per-cell glyphs, score line) directly into a `Vec<u8>` buffer via the generic `Renderer<W: Write>` — that exact design is what makes the 59 headless tests possible. Migration would force a `CrosstermBackend` + ratatui `Terminal`/`Buffer` indirection, replace the renderer with widgets, and rework the test harness — for zero required functionality (the board is a single full-width text grid; brief §19: "simple Rust → understandable code → successful Windows build → working Snake game").
- **Decision: STAY with crossterm 0.29; ratatui migration rejected** (unnecessary framework overhead).

### D. Visual problems → rendering decisions belong to Task 4 (recorded here for scope separation only)

The two user-reported visual issues (disjoint 1-column square glyphs on a vertical run; vertical movement visually ~2× faster due to a terminal cell's ~1:2 pixel aspect) are **renderer mapping** problems, confirmed by the research above (cells are ~taller than wide; crossterm does per-cell positioning). The chosen fix (double-width cells, contiguous glyphs) is specified in Task 4's own 4.1a/4.1b cycle — **this plan does not implement or plan those code changes.**

---

## Implementation Steps (executed in step 4.2 by implementer)

### Step 0 — Preconditions check (abort if any fails)

1. Run `git status` in `C:\repo\rust-snake`: expect `On branch feat/terminal-rendering-and-board`, working tree clean. If dirty or on another branch, STOP and report to caller (branch creation is step-2 restricted; not your job).
2. Run `git log --oneline -3`: expect HEAD `78c9d61 chore: bump version to 0.3.1`.
3. Read `.agent/project-info/context.md` and `.agent/project-info/tech.md` in full (they must exist with the sections named below).

### Step 1 — Edit `.agent/project-info/context.md` (Task 1)

- In the `## Current Work Focus` section (starts at line 3, currently 3 bullets), insert exactly **one new bullet as the last bullet of that section**, i.e. after the bullet starting "- Next up: see **Immediate Next Steps** below" (line 7) and before the `---` separator (line 9).
- Insert this bullet verbatim (real newlines, no `\n` literals):

```markdown
- **Alpine VM MCP (`alpine-vm`) availability registered (verified 2026-10-05):** `vm_status` returned `vmState: running | sshReachable: true` (SSH reachable on `127.0.0.1:3022`); `vm_run_command` with `ls /rust-snake` returned exactly `AGENTS.md`, `Cargo.lock`, `Cargo.toml`, `Dockerfile`, `LICENSE`, `README.md`, `compose.yaml`, `dist`, `docs`, `src`, `tests` — matching the host repo tree, so `/rust-snake` in the VM is the same project folder (host→VM shared folder). Docker availability is granted through this MCP (`docker` is in the `vm_run_command` allowlist `docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami`).
```

- Do NOT touch any other line/section of the file.

### Step 2 — Verify context.md edit

1. Re-read `.agent/project-info/context.md` and confirm the new bullet is placed exactly as specified.
2. Run `git diff --stat -- .agent/project-info/context.md` and `git diff -- .agent/project-info/context.md`: the diff must contain ONLY added lines (one `+` bullet); zero `-` removal lines. If any removal appears, fix the edit before continuing.

### Step 3 — Edit `.agent/project-info/tech.md` (Task 2)

- The `## Pending Decisions` section (near the end of the file) uses the format `~~struck question~~ — resolved …`. The insertion anchor is the bullet "- ~~Random number generation approach~~ — resolved in Phase 1A: the `rand` crate (see `Cargo.toml`).". Insert exactly **two new bullets immediately AFTER that `rand` bullet and BEFORE the "**Open:** first real compilation…" bullet.
- Insert these two bullets verbatim, keeping the file's existing formatting style (struck-through resolved prefix + `**Open**` state untouched):

```markdown
- ~~Terminal drawing library for better console rendering~~ — **resolved 2026-10-05 (crossterm research, critical-workflow 4.1b for Tasks 1–2): stay with `crossterm 0.29`; `ratatui` rejected as an unnecessary widget framework on top of crossterm** (the game already renders its own full-frame ASCII board via `Renderer<W: Write>`; migration would add `CrosstermBackend`/`Terminal`/`Frame` indirection and rework the headless test buffers for zero required functionality — keep-it-simple, brief §19).
- crossterm 0.29 research findings (2026-10-05, via context7 `/crossterm-rs/crossterm`): crossterm is character-cell-based and has no display-width measurement API — cursor positioning is strictly per terminal cell (`MoveTo(col, row)`), and glyph width/aspect rendering is decided by the terminal emulator; fixing the reported disjoint-glyph and ~2× vertical-speed perception therefore requires a renderer-side glyph mapping (each logical cell → 2 terminal columns; implemented in Task 4, not here). All APIs the project uses are confirmed stable in 0.29: `queue!`/`flush`, `cursor::MoveTo`/`Hide`/`Show`, `terminal::Clear(ClearType::All)`, `enable_raw_mode`/`disable_raw_mode`, `EnterAlternateScreen`/`LeaveAlternateScreen`, `event::poll`/`event::read`, `KeyEventKind` (no deprecations; no upgrade needed).
```

- Do NOT modify the existing bullets inside Pending Decisions: leave the struck-through ones and the `**Open:**` bullet untouched.

### Step 4 — Verify tech.md edit

1. Re-read the `## Pending Decisions` section of `.agent/project-info/tech.md` and confirm both new bullets sit between the `rand` bullet and the `**Open:**` bullet.
2. Run `git diff -- .agent/project-info/tech.md`: again only `+` additions.

### Step 5 — Gitignore compliance + commit

1. Run `git check-ignore .agent/project-info/context.md .agent/project-info/tech.md .kilo/plans/20261005-vm-context-and-crossterm-research.md` — expected: no output (none ignored). (Pre-verified by planner; re-run per the compliance rule.)
2. Run `git status` and stage exactly: `.agent/project-info/context.md`, `.agent/project-info/tech.md`, `.kilo/plans/20261005-vm-context-and-crossterm-research.md` (this plan file). Nothing else (`dist/` and `target/` stay gitignored).
3. Commit with message: `docs: register alpine-vm mcp access and crossterm research findings` (a single commit — the two doc changes and the plan file form one logical docs unit; the plan file is new and not yet committed).
4. Run `git diff HEAD~1 --stat` to confirm exactly the three files changed.

### Step 6 — Report

- Report to caller: files changed, commit hash, and the statement that Tasks 3 and 4 (lines 3–4 of the TODO) were NOT touched.
- Do NOT edit the TODO file — `[DONE]` marks on lines 1–2 belong to step 4.6.

---

## Verification Checklist (plan vs the two TODO lines)

- TODO line 1 (VM MCP registration): encoded verbatim as Step 1 into `context.md`, inside **Current Work Focus** only; existing content preserved; `brief.md` untouched. ✓
- TODO line 2 (terminal drawing library research): research executed here via context7 (crossterm confirmed fit + all used APIs stable; ratatui evaluated and rejected per keep-it-simple); outcome encoded as Step 3 into `tech.md` Pending Decisions (resolved-note, consistent formatting) and summarized in this plan file. ✓
- No `src/`, `tests/`, `Cargo.toml`, `TODO` file, or `brief.md` changes; version bump was step 3 (already done at `78c9d61`); push is step 5 (not here). ✓
- Markdown files touched: only the two project-info files + this plan file. ✓
