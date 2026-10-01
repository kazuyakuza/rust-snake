# Simplification Plan — Initialize Project Info (step 4.3, docs variant)

Date: 2026-10-01
Scope of review: files produced in step 4.2 (commits `72952e3`, `8a3c550`):
`.agent/project-info/product.md`, `context.md`, `architecture.md`, `tech.md`, root `AGENTS.md`, root `README.md`.
Source of truth: `.agent/project-info/brief.md` (MUST NOT be modified — it stays authoritative).

## Purpose

Remove duplicated content and a structural regression found in the step 4.2 docs.
All edits are tiny, local, fully specified string replacements. No restructuring, no new files, no section moves.

## Hard constraints for the implementer (50% restriction)

- Apply ONLY the steps below, exactly as written, in the given order. No judgment calls, no extra edits.
- Use the structured edit tool with the exact `old` string; do not rewrite whole files.
- Do NOT touch: `brief.md`, `context.md`, any TODO file, any `.kilo/` rule/agent file, `README.md` sections not named below.
- No invented requirements: every kept/rewritten fact must remain traceable to `brief.md` (§ numbers cited in the plan).
- No commits/pushes in this plan; the Critical Workflow step that applies these fixes owns the commit.
- Content uses real newlines only; no commented-out markdown.

---

## Step 1 — `AGENTS.md` line 8: restore the Project Brief link (regression from `72952e3`)

Commit `72952e3` turned the linked heading into an empty section (`## Project Brief` with no body). Restore the original linked heading.

- File: `AGENTS.md`
- Replace exactly:

  `## Project Brief`

- With exactly:

  `## [Project Brief](.agent/project-info/brief.md)`

- Uniqueness note: the string `## Project Brief` occurs only once (the list entry on line 18 starts with `- [Project Brief]` and must stay untouched).
- Verify: line 8 is `## [Project Brief](.agent/project-info/brief.md)`; file still has 22 lines; the 5-item Project Info list is unchanged.

## Step 2 — `architecture.md`: remove the duplicated status bullet (line 8)

The bold status line on line 3 and the bullet on line 8 state the same fact verbatim in the same file.

- File: `.agent/project-info/architecture.md`
- Delete exactly this line (the whole bullet, line 8):

  `- Current status: planned / not yet implemented — the planned files \`Cargo.toml\`, \`Dockerfile\`, \`compose.yaml\`, and \`src/main.rs\` do not exist yet.`

- Keep line 3 (`**Status:** Planned — ...`) — it is the single status marker for this file.
- Verify: `## System Architecture` section contains only one bullet (the logical-grid one, brief §5); file is 98 lines.

## Step 3 — `architecture.md` line 67: drop food-spawn clause duplicated in its own "Collision & Food Rules" section

Food spawn rules are already stated in full under `## Collision & Food Rules` (`always exactly one food ... never on the snake`, brief §9). Remove only the trailing food clause from the layout bullet.

- File: `.agent/project-info/architecture.md`
- Replace exactly:

  `- Snake starts with 1 head + 2 body segments = 3 blocks; initial direction \`RIGHT\`; food spawns at a random valid non-overlapping position (brief §4, §13).`

- With exactly:

  `- Snake starts with 1 head + 2 body segments = 3 blocks; initial direction \`RIGHT\` (brief §4, §13).`

- Verify: the bullet no longer mentions food; the `## Collision & Food Rules` section is unchanged.

## Step 4 — `architecture.md` line 91: point the Non-Goals cross-reference at the source of truth

Step 7 turns `product.md` Non-Goals into a pointer to `brief.md` §16; reference the brief directly so the pointer stays precise.

- File: `.agent/project-info/architecture.md`
- Replace exactly:

  `- No menus/pause/settings/config files — see \`product.md\` Non-Goals.`

- With exactly:

  `- No menus/pause/settings/config files — see \`brief.md\` §16 Non-Goals.`

- Verify: after Steps 2–4 the file is 98 lines and ends with the unchanged `## Critical Paths` section.

## Step 5 — `tech.md`: remove redundant status bullet (line 8)

`## Build & Output` already ends with "Note: none of these files exist yet — implementation pending." (line 25); the Stack-section status marker duplicates it inside the same file.

- File: `.agent/project-info/tech.md`
- Delete exactly this line (the whole bullet, line 8):

  `- Status marker: the stack is defined, but the project build does not exist yet (no \`Cargo.toml\`).`

- Verify: `## Stack` section keeps its 3 bullets (Language / Target platform / UI).

## Step 6 — `tech.md`: remove Docker-image constraint duplicated from Development Setup (line 31)

Identical fact already stated in `## Development Setup` ("The Docker image contains everything required to compile the Rust project for the Windows target (brief §3)").

- File: `.agent/project-info/tech.md`
- Delete exactly this line (the whole bullet, first item of `## Technical Constraints`, line 31):

  `- The Docker image must contain everything needed to compile for the Windows target (brief §3).`

- Verify: `## Technical Constraints` keeps the 3 remaining bullets (fixed speed / no restart-persistence-config / main.rs budget); file is 48 lines.

## Step 7 — `product.md`: replace verbatim copy of brief §16 Non-Goals with grouped summary + pointer

The current 19-bullet list is a verbatim copy of `brief.md` §16 (maintenance/drift hazard; scope detail belongs to the brief).

- File: `.agent/project-info/product.md`
- Replace the block that starts with the line:

  `The initial version should not include:`

  and ends with the line:

  `These may be considered in future experiments, but they are outside the initial scope.`

  (that whole block, 23 lines including the bullet list and blank lines)

- With exactly:

```markdown
Not in scope for the initial version — authoritative list: `brief.md` §16 (19 items, grouped by category below, nothing dropped):

- **Graphics & media**: graphical UI, game engine, sprites, textures, sound, music, animations beyond normal terminal rendering, Windows-specific GUI APIs.
- **Multiplayer & persistence**: multiplayer, networking, save games, high-score persistence.
- **Game depth**: multiple levels, increasing difficulty, power-ups.
- **UX surface**: menus, pause functionality, settings, configuration files.

These may be considered in future experiments, but they are outside the initial scope.
```

- Fidelity check (do before saving): group counts sum to 8 + 4 + 3 + 4 = 19, equal to the bullet count of `brief.md` §16; every category word appears verbatim in brief §16.

## Step 8 — `product.md`: replace verbatim copy of brief §18 Definition of Done with grouped summary + pointer

The section claims a "Summary of its 17 points" but reproduces all 17 items verbatim.

- File: `.agent/project-info/product.md`
- Replace the block that starts with the line:

  `\`brief.md\` §18 "Definition of Done" is the authoritative checklist. Summary of its 17 points:`

  and ends with the line:

  `17. A key press exits the application after Game Over.`

  (that whole block, 19 lines including the numbered list and the blank line)

- With exactly:

```markdown
`brief.md` §18 "Definition of Done" is the authoritative checklist (17 items, grouped below, nothing dropped):

- **Build chain** (items 1–3): the project builds successfully using Docker; Docker produces a Windows `.exe`; the `.exe` runs directly on Windows.
- **Start & initial state** (items 4–6): `Press any key to start` screen; the snake starts with exactly 3 blocks; the snake moves continuously.
- **Controls** (items 7–8): arrow keys control the direction; the snake cannot immediately reverse direction.
- **Food & score** (items 9–12, 15): food appears at a valid random position; eating food increases score by 1 and length by 1; the food respawns; the score is displayed during gameplay.
- **Game over** (items 13–14, 16–17): hitting the boundary and hitting the snake's own body each cause Game Over; Game Over displays the final score; a key press exits the application.
```

- Fidelity check (do before saving): item coverage is 1–3, 4–6, 7–8, 9–12 + 15, 13–14 + 16–17 = all 17 brief §18 items, each worded per brief; no item added or dropped.

- Verify after Steps 7–8: `product.md` is 58 lines; sections in order: `# Product — Rust Snake`, Problem Definition, Product Goals, User Experience, Non-Goals, Success Criteria.

## Step 9 — `README.md` line 81: drop transient status clause duplicated in Getting Started #2

Initialization status is stated twice in README (Project Structure bullet and "Getting Started" step 2); keep the step-2 mention only.

- File: `README.md`
- Replace exactly:

  `plus the behavior guide \`instructions.md\` live here; Project Info is initialized — all of them exist.`

- With exactly:

  `plus the behavior guide \`instructions.md\` live here.`

- Uniqueness note: the substring `all of them exist.` occurs only in this line.
- Verify: the `.agent/` bullet ends at "live here."; `## Getting Started` step 2 is unchanged; README still has 201 lines.

---

## Global verification after all steps

1. `git status` + `git diff --stat` — exactly these 5 files modified, nothing else, nothing new staged:
   `AGENTS.md` (22 lines), `README.md` (201 lines),
   `.agent/project-info/product.md` (58), `.agent/project-info/tech.md` (48), `.agent/project-info/architecture.md` (98).
2. Unchanged files confirmed: `.agent/project-info/brief.md`, `.agent/project-info/context.md`, all `.agent/todos/*`, `.kilo/*` rules.
3. Spot-check traceability: every numeric fact still present in the edited files matches brief: 19 non-goals, 17 DoD items, 3-block snake, 40×25 board, 120 ms, `docker compose run --rm build`, `dist/snake.exe`.
4. `git diff` contains no added lines that introduce new requirements (only reworded/grouped restatements of brief content and deletions).
5. No commit of ignored paths (`.gitignore` compliance): `dist/` never appears as an artifact here — docs-only change.

## Explicitly NOT simplified (recorded for completeness)

- `architecture.md` `## Game Loop` tick sequence and `## Screens & States`: intentional architecture/runtime content (brief §7, §10–§12), referenced with §-citations; kept.
- `product.md` `## User Experience` vs `architecture.md` layout/screens sections: same brief facts seen from product-UX vs system-state roles per `instructions.md`; overlap is by design; kept.
- `context.md` (31 lines): factual log, already minimal; kept as-is. Its "Current Work Focus" refresh is owned by the workflow's context-update step, not this plan.
- README intro vs `## About this Project`: conventional human-facing duplication; kept.
- `brief.md`: never edited by definition (source of truth).
