# Simplification Plan — Tasks 1+2 docs change-set (20261005, step 4.3)

Reviewer: code-simplifier. Contract: TODO `.agent/todos/20261005/20261005-todo-1.md` lines 1–2 +
implementation plan `.kilo/plans/20261005-vm-context-and-crossterm-research.md`.
Change reviewed: commit `d72c53a` ("docs: register alpine-vm mcp access and crossterm research
findings") on branch `feat/terminal-rendering-and-board` in `C:\repo\rust-snake`.

No file was modified by this step. This file is the only deliverable.

## 0. Verdict

SIMPLIFICATION REQUIRED — two small, atomic, fact-preserving rewrites of the three bullets added
by `d72c53a` (Sections 3–4). The 124-line plan file `.kilo/plans/20261005-vm-context-and-crossterm-research.md`
is intentionally NOT touched (it is the process record for the 4.x cycle and the verbatim archive of every
detail removed below). Neither step is a redesign; a junior implementer can execute both verbatim.

## 1. Duplication findings (basis for the rewrites)

| Fragment in `d72c53a` | Duplicated by | Action |
| --- | --- | --- |
| context.md L8: SSH endpoint `127.0.0.1:3022`, tool names `vm_status`/`vm_run_command`, registration of `alpine-vm` | context.md L5 (pre-existing bullet) | keep endpoint once; drop registration restatement (new fact is *verification*, not registration — the bullet said "availability registered", duplicating L5's topic) |
| context.md L8: full allowlist `docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami` | context.md L16 (Recent Changes, pre-existing) | replace with a pointer to the allowlist already documented |
| context.md L8: raw strings `vmState: running | sshReachable: true` + 11-item `ls /rust-snake` listing | plan file Step 1 block (verbatim archive, line 78) | keep only the conclusion ("same files as the host repo", "shared host→VM folder"); cross-reference the plan file for the transcript |
| tech.md L55: "critical-workflow 4.1b for Tasks 1–2" | workflow process metadata — file convention is "resolved in Phase X" | replace with cross-reference to the research plan file |
| tech.md L56: enumeration of 8 APIs (`queue!`/`flush`, `cursor::MoveTo`/`Hide`/`Show`, `Clear(ClearType::All)`, raw mode, alternate screen, `event::poll`/`read`, `KeyEventKind`) | tech.md L10 (Stack — same feature list in words) + tech.md L52 (pre-existing struck bullet — same features) + plan §B (exact evidence table) | keep the summary claim ("all used APIs confirmed stable; no upgrade"); drop the enumeration |
| tech.md L56: research method `via context7 /crossterm-rs/crossterm` | plan "Research method" paragraph (line 29) | drop from the doc bullet |

Every dropped fragment remains recoverable verbatim in the (unchanged) plan file or pre-existing doc lines.

## 2. Execution rules for the implementer (50%-restricted)

- Apply S1 then S2, in order. Each is ONE exact-string replacement via the `edit` tool: the `before`
  blocks below are byte-exact current file content (single long lines — do not wrap them), the `after`
  blocks are byte-exact replacements (also single lines, real newlines only at line ends).
- Do NOT touch any other line of either file; do NOT delete a bullet; do NOT move bullets between
  sections; do NOT touch `.kilo/plans/**`, `.agent/todos/**`, or anything under `src/`.
- No judgment calls allowed: if an `edit` fails on exact match, STOP and report to caller.

## 3. Step S1 — `.agent/project-info/context.md` (the L8 VM bullet)

Before (entire line 8):

```markdown
- **Alpine VM MCP (`alpine-vm`) availability registered (verified 2026-10-05):** `vm_status` returned `vmState: running | sshReachable: true` (SSH reachable on `127.0.0.1:3022`); `vm_run_command` with `ls /rust-snake` returned exactly `AGENTS.md`, `Cargo.lock`, `Cargo.toml`, `Dockerfile`, `LICENSE`, `README.md`, `compose.yaml`, `dist`, `docs`, `src`, `tests` — matching the host repo tree, so `/rust-snake` in the VM is the same project folder (host→VM shared folder). Docker availability is granted through this MCP (`docker` is in the `vm_run_command` allowlist `docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami`).
```

After:

```markdown
- **Alpine VM MCP (`alpine-vm`) availability verified 2026-10-05:** `vm_status` → running with SSH reachable on `127.0.0.1:3022`; `ls /rust-snake` in the VM lists the same files as the host repo, so `/rust-snake` is the project folder shared host→VM; Docker commands run through the MCP's `vm_run_command` allowlist (documented under Recent Changes). Full verification transcript: `.kilo/plans/20261005-vm-context-and-crossterm-research.md`.
```

Facts preserved: verification date; VM running + SSH endpoint; host↔VM shared-folder conclusion;
Docker availability through the MCP. ~35% shorter; zero restatement of L5/L16 content.

## 4. Step S2 — `.agent/project-info/tech.md` (the two new Pending Decisions bullets, L55–L56)

One atomic replacement of the two-line block. Before (lines 55–56 verbatim):

```markdown
- ~~Terminal drawing library for better console rendering~~ — **resolved 2026-10-05 (crossterm research, critical-workflow 4.1b for Tasks 1–2): stay with `crossterm 0.29`; `ratatui` rejected as an unnecessary widget framework on top of crossterm** (the game already renders its own full-frame ASCII board via `Renderer<W: Write>`; migration would add `CrosstermBackend`/`Terminal`/`Frame` indirection and rework the headless test buffers for zero required functionality — keep-it-simple, brief §19).
- crossterm 0.29 research findings (2026-10-05, via context7 `/crossterm-rs/crossterm`): crossterm is character-cell-based and has no display-width measurement API — cursor positioning is strictly per terminal cell (`MoveTo(col, row)`), and glyph width/aspect rendering is decided by the terminal emulator; fixing the reported disjoint-glyph and ~2× vertical-speed perception therefore requires a renderer-side glyph mapping (each logical cell → 2 terminal columns; implemented in Task 4, not here). All APIs the project uses are confirmed stable in 0.29: `queue!`/`flush`, `cursor::MoveTo`/`Hide`/`Show`, `terminal::Clear(ClearType::All)`, `enable_raw_mode`/`disable_raw_mode`, `EnterAlternateScreen`/`LeaveAlternateScreen`, `event::poll`/`event::read`, `KeyEventKind` (no deprecations; no upgrade needed).
```

After:

```markdown
- ~~Terminal drawing library for better console rendering~~ — **resolved 2026-10-05 (research record: `.kilo/plans/20261005-vm-context-and-crossterm-research.md`): stay with `crossterm 0.29`; `ratatui` rejected as an unnecessary widget framework on top of crossterm** (the game already renders its own full-frame ASCII board via `Renderer<W: Write>`; migration would add `CrosstermBackend`/`Terminal`/`Frame` indirection and rework the headless test buffers for zero required functionality — keep-it-simple, brief §19).
- crossterm 0.29 research findings (2026-10-05): crossterm is character-cell-based with **no display-width measurement API** — cursor positioning is per terminal cell (`MoveTo(col, row)`) and glyph width/aspect is decided by the terminal emulator, so the reported disjoint-glyph and ~2× vertical-speed perception issues require a renderer-side glyph mapping (each logical cell → 2 terminal columns; implemented in Task 4, not here). All APIs used by `src/terminal/` and `src/main.rs` are confirmed stable in 0.29 (no deprecations; no upgrade needed) — per-API evidence table in the research record.
```

Facts preserved: the decision + its rationale clauses; the no-display-width-API finding; terminal-emulator
aspect behavior; the 2-columns-per-cell fix direction and its Task 4 attribution; the all-APIs-stable
conclusion; traceability (research record path, replacing process metadata). The per-API enumeration is
the only removal — double-covered locally (L10, L52) and in plan §B.

## 5. Verification block (run from `C:\repo\rust-snake`, after S1+S2)

1. `git diff --stat` — exactly: `.agent/project-info/context.md` and `.agent/project-info/tech.md`, 2 insertions + 2 deletions total (one-for-one line rewrites). Nothing else dirty.
2. `git diff -- .agent/project-info/context.md` — the `-` line is only the old L8 bullet, the `+` line only its S1 replacement; no pre-existing line appears in the diff.
3. `git diff -- .agent/project-info/tech.md` — the `-` block is exactly L55–L56, the `+` block exactly the S2 replacement; the `rand` bullet (L54) and `**Open:**` bullet remain neighbors.
4. `git check-ignore .agent/project-info/context.md .agent/project-info/tech.md` — no output (not ignored).

## 6. Commit step

Stage only the two files and commit:

```
git commit -m "docs: dedup alpine-vm and crossterm project-info bullets"
```

Then `git show --stat HEAD` — exactly 2 files changed. Do NOT amend `d72c53a`. Do NOT edit the TODO file.

## 7. Explicitly out of scope (do NOT do)

- No edits to `.kilo/plans/20261005-vm-context-and-crossterm-research.md` (verbatim archive + process record).
- No changes to context.md L5/L16 (the pre-existing bullets the new ones duplicate from) — preserved-content rule.
- No merging/removal of a whole bullet; no section moves; no wording changes beyond S1/S2.
- Tasks 3–4 (board 80×80, renderer/velocity) and all of `src/`, `tests/`, `Cargo.toml` — untouched, different cycle.
