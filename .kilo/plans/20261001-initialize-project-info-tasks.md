# Implementation Plan — Initialize Project Info Files & Update README (TODO Tasks 1+2, grouped)

- TODO file: `.agent/todos/20261001/20261001-todo-1.md` (tasks 1 and 2, executed in ONE 4.x cycle per global plan)
- Global plan: `.kilo/plans/20261001-initialize-project-info.md` (DO NOT overwrite it — this is the separate per-task plan)
- Source of truth for all content: `.agent/project-info/brief.md` (Rust Snake brief)
- Behavior contract: `.agent/project-info/instructions.md`
- Branch: `feat/initialize-project-info` (already created in step 2 — do NOT create/switch branches)
- Git push: forbidden in this step (restricted to step 5)
- Front-end related: NO → 4.1a/4.5a do not apply

---

## 1. Current Repository Snapshot (verified 2026-10-01)

- `.agent/project-info/` contains exactly: `brief.md`, `instructions.md`, `.initialized` (all three git-tracked via `git ls-files`).
  - `.initialized` content: `THIS MARKS THE FILE AS DEFAULT VERSION` — it is the template default-brief marker; deleting it completes initialization (per `instructions.md` "Core workflows" and `README.md` note-on-project-info).
- Missing core files (required by `instructions.md` "Core Files (Required)"): `product.md`, `context.md`, `architecture.md`, `tech.md`.
- `README.md` is the untouched template ("Base Project for AI Agent Driven Development") — no Rust Snake content yet.
- Implementation does NOT exist yet:
  - no `Cargo.toml`, no `Cargo.lock`, no `Dockerfile`, no `compose.yaml`
  - `src/` contains only `.gitkeep`
  - `dist/` does not exist (and `dist/` is listed in `.gitignore` — build output stays out of git)
- `.agent/project-structure.md` documents folders only: `.agent/`, `.kilo/`, `.opencode/`, `docs/`; `src/` has no folders yet.
- Working tree is clean except untracked `.agent/todos/20261001/` and `.kilo/plans/20261001-initialize-project-info.md` (Planner Agent's files; the implementer must NOT commit them in this step's commits).
- `AGENTS.md` currently links only `brief.md` + `instructions.md` under "Project Info"; `instructions.md` section "Integration in AGENTS.md" mandates: "All project info files must be referenced/linked in AGENTS.md file at root."

**Consequence encoded in this plan:** all 4 project-info files describe the PLANNED architecture/tech from `brief.md` and explicitly mark implementation as pending. No file may claim game code or build files exist. No `Cargo.toml`/`Dockerfile`/`compose.yaml`/Rust code is created in this step.

---

## 2. Fixed Scope & Structural Decisions (binding for the implementer)

1. Create exactly 4 new Markdown files: `.agent/project-info/product.md`, `.agent/project-info/context.md`, `.agent/project-info/architecture.md`, `.agent/project-info/tech.md`. No additional project-info files.
2. Delete `.agent/project-info/.initialized` (it is git-tracked; the deletion must be staged in git).
3. Update `AGENTS.md` "Project Info" section to reference/link the 4 new files (mandated by `instructions.md` "Integration in AGENTS.md"). No other `AGENTS.md` edits.
4. Rewrite/extend `.agent/README`-adjacent root `README.md` per the exact structure in Section 4 of this plan.
5. Do NOT modify: `brief.md`, `instructions.md`, `.agent/project-structure.md` (folder map — no folder added or removed, so no update required per `.kilo/commands/project-structure.md`), `docs/*`, `.kilo/*`, `.opencode/*`, `.gitignore`.
6. Use real newline characters in every written file (`.kilo/rules/newline-prevention.md`); never embed literal `\n`.
7. Writing method: use the dedicated `write`/`edit` tools only — no PowerShell content cmdlets, no here-strings.
8. Every factual claim in the new files must trace back to `brief.md`, `instructions.md`, `AGENTS.md`, current `README.md`, `.agent/project-structure.md`, or `.kilo/commands/critical-workflow.md`. If a fact is not traceable, do not write it.
9. Markdown style inside the new docs: sentence-case section titles are not required — reuse the exact headings given below; use bullet lists and code-block ```text``` fences for literals/paths/commands; add a horizontal rule `---` between top-level sections like other repo docs do.
10. Keep every new file under ~200 lines (they are docs, the hard rule applies to `src/`, but stay concise).

---

## 3. Part A — Task 1: Project Info Files

### 3.1 `.agent/project-info/product.md`

Required sections (exact headings, in this order):

```markdown
# Product — Rust Snake

## Problem Definition
## Product Goals
## User Experience
## Non-Goals
## Success Criteria
```

Content directives (bullet-level, all sourced from `brief.md`):

- `Problem Definition`:
  - Project is a small, self-contained Snake game in Rust used as a practical learning exercise (brief §1).
  - The goal is learning Rust — structs, enums, collections, ownership/borrowing, loops, input handling, timers, modules, Windows compilation — not producing a production-quality game (brief §1, §15).
  - Problem it solves for the user: a playable, understandable terminal game that exercises core Rust concepts and Docker-driven Windows cross-compilation (brief §1, §3, §19).
- `Product Goals`:
  - Playable terminal/console Snake that runs directly on Windows (brief §1, §2).
  - Straightforward, understandable Rust code over sophisticated game architecture (brief §14, §19).
  - Successful build chain: Docker compiles → `dist/snake.exe` runs on Windows (brief §3, §19 first milestone).
  - Include the guiding principle chain from brief §19: `Simple Rust → Understandable code → Successful Windows build → Working Snake game` (as a ```text``` block or bullet list).
  - Quote the first milestone: "Compile a Rust program in Docker, produce a Windows `.exe`, run it locally, and play Snake in the terminal." (brief §19).
- `User Experience`:
  - Terminal/console only; no graphical window, no engine, no sprites/assets; ASCII/Unicode glyphs may represent snake, food, borders, UI (brief §2).
  - Launch shows a start screen: `Press any key to start`; game starts on key press (brief §11).
  - Player controls direction with arrow keys; snake moves continuously; immediate reversal into itself is forbidden (brief §6).
  - Exactly one food item on the board always; eating it: score +1, snake length +1, food respawns at a random valid position (brief §4, §9).
  - Score is visible during gameplay, above or below the board, e.g. `Score: 7` (brief §10).
  - Boundary or self collision → immediate `GAME OVER` screen with final score; application exits on key press; no restart system (brief §8, §12).
- `Non-Goals`:
  - Reproduce the full list from brief §16 as bullets: graphical UI, game engine, sprites, textures, sound, music, multiplayer, networking, save games, high-score persistence, multiple levels, increasing difficulty, power-ups, menus, pause functionality, settings, configuration files, animations beyond normal terminal rendering, Windows-specific GUI APIs.
  - Note that these may be considered in future experiments but are outside the initial scope (brief §16 closing note).
- `Success Criteria`:
  - Reference `brief.md` §18 "Definition of Done" and summarize its 17 points as a numbered list (builds with Docker; produces Windows `.exe`; runs on Windows; start screen; snake starts with 3 blocks; continuous movement; arrow control; no immediate reverse; valid random food; +1 score; +1 length; food respawn; boundary → game over; self → game over; score displayed; final score on game over; key press exits).
  - State that `brief.md` §18 is the authoritative checklist.

### 3.2 `.agent/project-info/context.md`

Required sections (exact headings, in this order) — these mirror `instructions.md` ("context.md — Factual log: Current work focus, recent changes, and immediate next steps"):

```markdown
# Context — Rust Snake

## Current Work Focus
## Recent Changes
## Implementation Status
## Immediate Next Steps
```

Content directives:

- `Current Work Focus`:
  - Executing TODO `.agent/todos/20261001/20261001-todo-1.md` on branch `feat/initialize-project-info`: initializing project info files and updating `README.md`.
- `Implementation Status`:
  - **Not implemented yet**: no `Cargo.toml`, no `Cargo.lock`, no `Dockerfile`, no `compose.yaml`, no Rust source (`src/` contains only `.gitkeep`); `dist/` output directory does not exist yet.
  - All build/game runtime details described in `architecture.md` and `tech.md` are the PLANNED design from `brief.md`, marked as pending until implemented.
- `Immediate Next Steps` (order matters, this is the project's live roadmap):
  1. Implement the Docker build environment: `Dockerfile` + `compose.yaml` so that `docker compose run --rm build` produces `dist/snake.exe` (brief §3).
  2. Implement the game in Rust: `Cargo.toml` and `src/main.rs` (code may live entirely in `main.rs` initially; split into modules only if justified later — brief §17).
  3. Verify the game against the Definition of Done checklist in `brief.md` §18.
  4. After implementation, update project info per `instructions.md` ("Project Info Update") — especially `context.md`.
- `Recent Changes`:
  - Project brief customized for the Rust Snake game; template leftovers (`CHANGELOG.md`, `TBD`) removed — commit `732419b`.
  - Project info initialized this cycle: created `product.md`, `context.md`, `architecture.md`, `tech.md`; removed `.initialized` marker; linked the files from `AGENTS.md`.
  - `README.md` adapted from the template to describe the Rust Snake project.

### 3.3 `.agent/project-info/architecture.md`

Required sections (exact headings, in this order) — mapped to `instructions.md` ("architecture.md — System architecture, paths, design patterns, and critical paths"):

```markdown
# Architecture — Rust Snake

## System Architecture
## Planned Repository Layout
## Core Components
## Game Loop
## Game Layout & Input Rules
## Screens & States
## Design Patterns / Constraints
## Critical Paths
```

Content directives:

- `System Architecture`:
  - Single terminal application; logical grid game state, not pixels (brief §5).
  - Status line (mandatory at top): describe the current status — planned/not yet implemented; planned files `Cargo.toml`, `Dockerfile`, `compose.yaml`, `src/main.rs` do not exist yet.
- `Planned Repository Layout` (brief §17) — reproduce as a ```text``` tree:
  ```text
  rust-snake/
  ├── Cargo.toml
  ├── Cargo.lock
  ├── Dockerfile
  ├── compose.yaml
  ├── src/
  │   └── main.rs
  └── dist/
      └── snake.exe
  ```
  - Note: code starts entirely in `main.rs`; module split only if justified later (brief §17).
  - Note that `dist/` is a host-mounted build output folder, ignored by git (`.gitignore`).
- `Core Components` (brief §14 conceptual model) — bullets, marked "suggested, exact structure left to implementation":
  - `Position` — `x`, `y` coordinates.
  - `Direction` — enum `Up | Down | Left | Right`.
  - `Snake` — `Vec<Position>`.
  - `Food` — a `Position`.
  - `Game` — holds `Snake`, `Food`, `Direction`, `Score`, `GameState`.
  - Constraint: favor straightforward Rust over unnecessary abstraction (brief §14).
- `Game Loop` (brief §7):
  - Fixed tick interval: 120 ms per movement (~8.3 moves per second); constant speed; no acceleration (brief §7).
  - Per-tick sequence as numbered list, verbatim semantics from brief §7: (1) read/process input, (2) calculate new head position, (3) boundary check, (4) self collision check, (5) move snake, (6) food check, (7) if eaten: score +1, grow snake, generate new food, (8) render, (9) wait for next tick.
  - Note: implementation may adjust ordering to accommodate terminal input handling (brief §7).
- `Game Layout & Input Rules`:
  - Board: logical grid, initial `WIDTH = 40`, `HEIGHT = 25` cells; visible boundaries; no screen wrap (brief §5, §8).
  - Cell states: empty, snake head, snake body, food; exact terminal characters decided during implementation (brief §5, §13).
  - Snake starts with 1 head + 2 body segments = 3 blocks; initial direction `RIGHT`; food spawns at a random valid non-overlapping position (brief §4, §13).
  - Controls: arrow keys only; movement is continuous; immediate reversal (e.g. `RIGHT` → `LEFT` in one move) must be rejected (brief §6).
- `Collision & Food Rules`:
  - Head outside the play area → game over; no wrap-around (brief §8, §9).
  - Head onto own body segment → game over (brief §8).
  - Always exactly one food on the board; spawns randomly, never outside the play area, never on the snake; handles the edge case of no free cells left (brief §9).
- `Design Patterns / Constraints`:
  - Keep-it-simple principle (brief §19): simple Rust → understandable code → successful Windows build → working game.
  - No menus/pause/settings/config files — see `product.md` Non-Goals.
- `Critical Paths` (bullet list of flows):
  - Build: `docker compose run --rm build` → host-mounted output `dist/snake.exe` (brief §3).
  - Run: execute `dist/snake.exe` directly on Windows terminal, outside the container (brief §3).
  - Develop: read `AGENTS.md` → `.agent/WORKFLOWS.md` → Critical Workflow (`.kilo/commands/critical-workflow.md`); project context lives in `.agent/project-info/*`.

### 3.4 `.agent/project-info/tech.md`

Required sections (exact headings, in this order) — mapped to `instructions.md` ("tech.md — Stack, development setup, technical constraints, and tool usage patterns"):

```markdown
# Tech — Rust Snake

## Stack
## Development Setup
## Build & Output
## Technical Constraints
## Tool Usage Patterns
## Pending Decisions
```

Content directives:

- `Stack`:
  - Language: Rust (brief §2).
  - Target platform: Windows (brief §2).
  - UI: terminal/console only — no graphical window, no external game engine, no sprites/graphical assets; ASCII/Unicode characters represent game elements (brief §2).
  - Status marker: stack is defined but the project build does not exist yet (no `Cargo.toml`).
- `Development Setup`:
  - Docker is used only as the compilation environment; the resulting executable must run directly on Windows outside the container (brief §3).
  - The Docker image contains everything required to compile the Rust project for the Windows target (brief §3) — no local Rust toolchain is described by the brief.
  - AI-agent tooling: Kilo Code and/or opencode, per `AGENTS.md` (both already configured in this repo; just link, do not re-explain).
- `Build & Output`:
  - Build command: ```docker compose run --rm build``` (brief §3 expected workflow).
  - Output: Windows executable in a host-mounted output directory `dist/snake.exe`; run it directly with ```snake.exe``` from Windows (brief §3).
  - Required project files for this to work: `Dockerfile`, `compose.yaml`, `Cargo.toml`, `src/` (brief §3 minimum list).
  - Note: none of these files exist yet — implementation pending.
- `Technical Constraints`:
  - The Docker image must contain everything needed to compile for the Windows target (brief §3).
  - Fixed game speed 120 ms per movement; no acceleration required (brief §7).
  - No restart system, no persistence, no configuration files (brief §12, §16).
  - Terminal-only rendering budget: keep the implementation in `main.rs` unless growth justifies modules (brief §17).
- `Tool Usage Patterns`:
  - Git flow and commits follow `.agent/WORKFLOWS.md` and `.kilo/commands/critical-workflow.md` (feature branches `feat/*` or `fix/*`; meaningful commit messages).
  - Agents must evaluate project info files at the start of every task (`instructions.md` "Initial Instruction") and keep `context.md` current.
   - CI/CD and additional dev tooling: none defined by the brief — only the Docker build workflow (brief §3) is defined so far.
- `Pending Decisions` (traceable deferrals — each cites its brief section):
  - Exact terminal characters for snake body/head, food, and borders — decided during implementation (brief §5).
  - Exact Rust data-structure layout for `Snake`/`Game` — left to the implementation (brief §14).
  - Random number generation approach (a Rust learning objective in brief §15; crate vs. hand-rolled choice is an implementation decision).

### 3.5 `AGENTS.md` — link the new files (commit A)

In section `## Project Info` (currently 2 sentences, lines 14–16), extend so all five core project-info files are linked. Required exact shape (keep existing sentences; add a short list in the same style — one bullet per line):

```markdown
## Project Info

All agents must read the [Project Info Instructions](.agent/project-info/instructions.md) at the beginning of each task. This ensures that the agent has a complete understanding of the project's context, architecture, and technical specifications.

- [Project Brief](.agent/project-info/brief.md) — core requirements and scope (source of truth)
- [Product](.agent/project-info/product.md) — user experience, problem definition, product goals
- [Context](.agent/project-info/context.md) — current work focus, recent changes, next steps
- [Architecture](.agent/project-info/architecture.md) — system architecture and critical paths
- [Tech](.agent/project-info/tech.md) — stack, setup, constraints, tooling
```

- Line 8 of `AGENTS.md` currently reads `## [Project Brief](.agent/project-info/brief.md)`. Replace that exact line with the plain heading `## Project Brief` (no link — the Project Brief link then lives in exactly one place: the Project Info bullet list below). Do NOT remove the section itself; do NOT touch any other `AGENTS.md` line.
- Keep the existing `## [Workflows](.agent/WORKFLOWS.md)` and `## [Rules](.agent/RULES.md)` lines unchanged.

### 3.6 Delete `.agent/project-info/.initialized`

- Delete the file (it currently reads `THIS MARKS THE FILE AS DEFAULT VERSION`).
- Rationale to include in the commit body (optional): the marker is the template default-brief detector (`.kilo/commands/project-info-init.md` trigger condition 2); its removal completes Project Info initialization per `instructions.md` "Core workflows".

---

## 4. Part B — Task 2: Update `README.md`

Rewrite root `README.md` with this EXACT section order and headings (kept sections preserve current relative order; new sections inserted after "About this Project"; template wording adapted only where noted):

1. `# Rust Snake` — title (replaces "Base Project for AI Agent Driven Development").
2. Intro paragraph (2–3 sentences): Rust Snake is a terminal Snake game in Rust, built by AI agents via the Critical Workflow; compiled inside Docker for Windows and run directly as `dist/snake.exe`. Keep the existing `**Attention AI Agents:**` paragraph verbatim (lines 5 of current README) — it is not template-specific.
3. `## Table of Contents` — regenerate matching the exact heading list below (anchors are lowercase, spaces → `-`):
   - Compatibility / Prerequisites / About this Project / Game Rules & Controls / Build & Run / Project Structure / Getting Started / The Critical Workflow / Agent Models / How to Start a Task (with its two `###` option sub-entries) / AI Agent Plans / Troubleshooting.
4. `## Compatibility` — KEEP content verbatim, with two minimal adaptations:
   - "This template is used on a daily basis..." → "This project setup is used on a daily basis..." (drop the word "template" wherever it refers to this repo).
   - Leave every link, version number (Kilo Code > 7.4.22), and the opencode Go affiliate link untouched.
5. `## Prerequisites` — KEEP Git + AI-agent-tool bullets; ADD one bullet: `Docker — required to build: Windows Cross compilation runs in a Docker build environment producing a Windows .exe` (traceable to brief §3). Reword any "template" phrasing.
6. `## About this Project` — REPLACE the template prose with Rust Snake description:
   - What: classic Snake game (brief §4) in a Windows terminal (brief §1).
   - Purpose: learning/experimentation project for Rust fundamentals (brief §1, §15) — not a production-quality game.
   - Build approach: Docker as compile environment only; output runs directly on Windows (brief §3).
   - Drop the template `### Design Principles` subsection entirely.
7. `## Game Rules & Controls` — NEW section, bullets only, each traceable to the brief:
   - Board: 40 x 25 logical grid with visible boundaries and no wrap-around (brief §5, §8).
   - Snake starts at length 3 (1 head + 2 body), moving continuously, initial direction right (brief §4, §13).
   - Controls: arrow keys change direction; movement continues between ticks; immediate reversal into itself is rejected (brief §6).
   - Speed: fixed 120 ms per move (~8.3 moves/s), constant, no acceleration (brief §7).
   - Food: exactly one on the board; spawns randomly, never on the snake; eating it = score +1, length +1, respawn (brief §4, §9).
   - Score displayed during gameplay, e.g. `Score: 7` (brief §10).
   - Game over on boundary hit or self collision; shows final score; exits on key press; no restart in the initial version (brief §8, §12).
   - Start screen: press any key to start (brief §11).
8. `## Build & Run` — NEW section:
   - Build: ```docker compose run --rm build``` described as the expected build workflow (brief §3).
   - Output: Windows executable `dist/snake.exe` in a host-mounted output directory (brief §3).
   - Run: execute `dist/snake.exe` directly from Windows — the container is only the compile environment (brief §3).
   - Status note (one sentence): the Docker build environment and Cargo project are being implemented; see `.agent/project-info/tech.md` for the full build contract.
   - Note that `dist/` is git-ignored (`.gitignore`), so binaries never get committed.
9. `## Project Structure` — KEEP the 5 existing config-dir bullets (`.agent/`, `.kilo/`, `.opencode/`, `.ignore`, `.kilocodeignore`) with their links; UPDATE the `.agent/` bullet phrasing "are created during Project Info initialization (see below)" to state they now exist (project info is initialized); ADD a short subsection `### Application Files (Planned)` listing `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `compose.yaml`, `src/main.rs`, `dist/snake.exe` with one-line purposes from brief §17, explicitly marked as not yet implemented.
10. `## Getting Started` — REPLACE "Getting Started (New Project Setup)" (template clone-flow) with an in-project flow:
   - Heading is exactly `## Getting Started` (drop "(New Project Setup)").
   - Steps: (1) read `AGENTS.md`; (2) the project brief (`.agent/project-info/brief.md`) and project info are already initialized (`product.md`/`context.md`/`architecture.md`/`tech.md` exist, marker removed); (3) follow the Critical Workflow via [How to Start a Task](#how-to-start-a-task); (4) build and play via [Build & Run](#build--run) once implementation lands; (5) every task adds entries/updates to `.agent/project-info/context.md` per `instructions.md`.
   - REWRITE the `> **Note on Project Info:**` blockquote: remove the "When cloning this template..." phrasing and the `/critical-workflow` init instructions; new meaning — `brief.md` is the source of truth; project info lives in `.agent/project-info/`; agents read instructions and keep `context.md` up to date.
11. `## The Critical Workflow` — KEEP the section, `mermaid` graph, and the [`critical-workflow.md`](.kilo/commands/critical-workflow.md) link verbatim.
12. `## Agent Models` — KEEP verbatim.
13. `## How to Start a Task` — KEEP verbatim including both `###` options (anchor links `#option-1-using-a-todo-file-recommended` and `#option-2-direct-chat-request` still work after ToC regeneration).
14. `## AI Agent Plans` — KEEP verbatim.
15. `## Troubleshooting` — KEEP verbatim.
16. Footer note line (`*Note: This workflow is actively maintained...*`) — KEEP.

Additional binding rules for README editing:

- Do not delete or rewrite the Critical Workflow mermaid block.
- Preserve every relative link target that exists in the repo (`AGENTS.md`, `docs/how-to-set-up-git.md`, `docs/how-to-write-todo-files.md`, `.kilo/commands/critical-workflow.md`, `.opencode/opencode.json`, `.agent/project-structure.md`, `.kilo/agents/*.md`, `.kilo/rules/tool-selection-priority.md`).
- Every TOC entry must have a matching `##`/`###` heading, in-document order, with a valid GitHub-style anchor.
- All new prose uses real newlines; no literal `\n` sequences (newline rules).

---

## 5. Git Handling (this step only; no branch ops; no push)

Order of operations:

1. Create/edit all files first (both A and B parts), WITHOUT staging.
2. Commit A (Task 1 artifacts) — stage exactly:
   - `.agent/project-info/product.md`
   - `.agent/project-info/context.md`
   - `.agent/project-info/architecture.md`
   - `.agent/project-info/tech.md`
   - `AGENTS.md`
   - deletion of `.agent/project-info/.initialized`
   - Add command (single path scope): `git add .agent/project-info AGENTS.md`
   - Message: `docs: initialize project info files and remove template marker`
   - Commit body (one short paragraph): initializes the five core project-info files for the Rust Snake project per `.agent/project-info/instructions.md`; deletes the `.initialized` default-template marker; links the new files from `AGENTS.md` per its "Integration in AGENTS.md" requirement.
3. Commit B (Task 2 artifact) — stage exactly `README.md`:
   - `git add README.md`
   - Message: `docs: adapt README to the rust snake project`
   - Commit body (optional): rewords template pages to project pages (Rust Snake description, game rules/controls, build/run contract, in-project getting started) while preserving the AI-agent workflow documentation.
4. Gitignore compliance (`.kilo/rules/gitignore-compliance.md`) before each commit:
   - Run `git status` and inspect what is staged; `dist/` never appears (it is not created at all in this step).
   - Do NOT stage the untracked `.agent/todos/20261001/` folder or `.kilo/plans/20261001-initialize-project-info.md` in commits A/B (they belong to the Planner Agent and step 4.6/5 bookkeeping).
   - Do NOT stage anything outside the exact file lists above.
5. Verify both commits exist: `git log --oneline -5` shows A then B at top, branch still `feat/initialize-project-info`.
6. NO push, NO merge, NO branch creation/switch, NO TODO file edits in this step (all restricted to other steps).

---

## 6. Verification Checklist (execute after writing files, before/after commits)

1. Files exist: `product.md`, `context.md`, `architecture.md`, `tech.md`, under `.agent/project-info/` (read each via the read tool — this also doubles as the real-newline check: multi-line content must show separate lines, never literal `\n`).
2. `.initialized` no longer exists on disk and `git status` shows its deletion staged for commit A.
3. Content alignment vs `brief.md` (spot-check each fact): board 40x25; initial length 3 (head + 2 body); speed 120 ms (~8.3 moves/s), constant, no acceleration; arrows-only controls; immediate reversal rejected; exactly one food; eating = score +1 + length +1 + respawn; boundary and self collision → GAME OVER; start screen text `Press any key to start`; game-over screen with final score and key-to-exit; no restart; non-goals list matches brief §16; docker command is exactly `docker compose run --rm build`; output is `dist/snake.exe`.
4. Pending-state honesty: every file states that `Cargo.toml`/`Dockerfile`/`compose.yaml`/`src` implementation does not exist yet; no file claims the game runs or the build works today.
5. No invented requirements: pick each factual statement in the new files and locate its origin in `brief.md` or an existing repo doc; any statement without a traceable origin gets rewritten or removed.
6. Link validity: every relative link added in `AGENTS.md` and `README.md` resolves to an existing file; every README TOC anchor matches its heading (lowercase, `-` for spaces, parentheses stripped, e.g. `Build & Run` → `#build--run`).
7. README sanity: TOC order equals section order; no template phrases remain where caller demanded adaptation ("This template...", "New Project Setup" heading, template Design Principles); Critical Workflow section still contains one full mermaid block.
8. Line-count sanity: each new project-info file reads clean and under ~200 lines; no dead code blocks or commented-out content anywhere (`.kilo/rules/no-commented-code.md`).
9. Git: `git log --oneline -10` shows the two new commits with the exact messages from Section 5; working tree contains only the Planner-owned untracked paths (`.agent/todos/20261001/`, `.kilo/plans/*.md`).
10. Sequence correctness inside `context.md`: "Immediate Next Steps" list matches the true next critical-workflow steps (implement Docker build env → implement Rust game → verify DoD → update project info).

---

## 7. Documentation Update Steps (post-write, low risk)

1. Update `AGENTS.md` Project Info list (Section 3.5 above) — done as part of commit A.
2. `.agent/project-structure.md` — NO change required: this task adds files only inside an existing documented folder, and adds no folders (verified against `.kilo/commands/project-structure.md` update trigger). Leave untouched.
3. Note for the Docs Specialist (step 4.4): after implementation, cross-check `context.md` freshness per `instructions.md` "Critical Closing Step"; do not perform that update in this step.

---

## 8. Test / Build Steps

- There is NO build, test, or console verification possible in this step: no `Cargo.toml`, no Dockerfile/compose.yaml, no source code exist (this is exactly why the docs must mark everything pending). Do not run `docker`, `cargo`, or any build command.
- The only executable validations are the git commands and read-based checks in Section 6.

---

## 9. Out of Scope (hard blocks for the implementer)

- Creating `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `compose.yaml`, any Rust source, or `dist/`.
- Any change to `brief.md`, `instructions.md`, `.agent/WORKFLOWS.md`, `.agent/project-structure.md`, `.kilo/**`, `.opencode/**`, `docs/**`.
- TODO file edits (`[DONE]` marking) — that is step 4.6.
- Git push, merge, branch creation/switch — restricted to steps 2/5 of the Critical Workflow.
- Editing or deleting `.kilo/plans/20261001-initialize-project-info.md` (global plan) or any other plan file.

---

## 10. Deliverables Summary

| # | Deliverable | Commit |
| --- | --- | --- |
| 1 | `.agent/project-info/product.md` created with the exact heading set and content maps above | A |
| 2 | `.agent/project-info/context.md` created | A |
| 3 | `.agent/project-info/architecture.md` created | A |
| 4 | `.agent/project-info/tech.md` created | A |
| 5 | `.agent/project-info/.initialized` deleted | A |
| 6 | `AGENTS.md` Project Info section links all five core files | A |
| 7 | `README.md` restructured for Rust Snake per Section 4 | B |
