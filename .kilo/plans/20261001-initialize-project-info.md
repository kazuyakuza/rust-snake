# Global Plan — Initialize Project Info & Update README (20261001)

- TODO file: `.agent/todos/20261001/20261001-todo-1.md`
- Source of truth: `.agent/project-info/brief.md` (Rust Snake brief)
- Front-end related: **NO** (documentation/context files only)

## Global Pre-Analysis

**Current repository state:**
- Repo is the customized "Base Project for AI Agent Driven Development" template. `brief.md` is fully defined for a **Rust Snake** game (terminal Snake in Rust, compiled in Docker, run as a Windows `.exe`).
- Uncommitted working-tree changes exist: `brief.md` modified, `CHANGELOG.md` deleted, `TBD` deleted (template cleanup by user). These will be committed during Step 2.
- `.agent/project-info/` currently contains only `brief.md` + `instructions.md` + `.initialized` marker. Missing core files: `product.md`, `context.md`, `architecture.md`, `tech.md`.
- No game implementation exists yet: no `Cargo.toml`, `Dockerfile`, `compose.yaml`, no Rust source (only `src/.gitkeep`). The project-info files must describe the *planned* architecture/tech from the brief and mark the implementation as pending.
- `README.md` still describes the base template, not the Rust Snake project.

**Technical & architecture decisions (Planner, fixed for all tasks):**
1. Project-info files are Markdown, created under `.agent/project-info/` with these exact names: `product.md`, `context.md`, `architecture.md`, `tech.md`.
2. `brief.md` is source of truth; in case of conflict, missing files must align with it (per `.agent/project-info/instructions.md`).
3. All content derived from: `brief.md`, `AGENTS.md`, `README.md`, `.agent/project-structure.md`, `.kilo/commands/critical-workflow.md` — no invented requirements.
4. `.agent/project-info/.initialized` is git-tracked and MUST be deleted (initialization completes it) and the deletion committed.
5. README keeps the AI-agent/collaboration sections (Compatibility, Critical Workflow, Agent Models, How to Start a Task, Troubleshooting) — they remain valid — and gains/updates Rust Snake project sections (About, Structure, Build & Run, Game rules/controls). "Getting Started (New Project Setup)" must be reworded to reflect that brief/project-info are already initialized.
6. Step 3 (Version Update): **skipped** — no versioned manifest exists yet (no `package.json`/`Cargo.toml`).
7. Feature branch name: `feat/initialize-project-info`.

## Grouping Decision

Task 1 (initialize project info) and Task 2 (update README) are extremely short and related (both documentation/context work driven by the same brief). Per Critical Workflow §1.2 they are **joined into a single 4.1–4.6 cycle** below.

## Plan Steps (steps 2–6)

- Step 2: Git Feature Branch Setup => implementer
  - Commit unstaged changes (`brief.md` modification, `CHANGELOG.md`/`TBD` deletions) with message `chore: customize project brief for rust snake and remove template leftovers`.
  - Verify on `main` (already there), create and switch to `feat/initialize-project-info`.
- Step 3: Version Update => SKIPPED (no version manifest exists; nothing to bump)
- Task 1+2 (grouped): 4.1–4.6 cycle
  - 4.1b Analysis & Planning => architector → `.kilo/plans/20261001-initialize-project-info.md` (this file serves as basis; architector refines into per-task plan or confirms this plan as the task plan)
  - 4.2 Implementation => implementer (create 4 project-info files, delete `.initialized`, update README)
  - 4.3 Code Review & Simplification => code-reviewer + code-simplifier (docs quality; fix plan if needed => implementer)
  - 4.4 Documentation => docs-specialist (cross-links between project-info files and README/AGENTS.md; TOC check)
  - 4.5b Overall Plan Adherence => architector (verify adherence; front-end verification 4.5a N/A)
  - 4.6 Task Completion => implementer (add `[DONE]` to both TODO lines, commit)
- Step 5: TODO File Completion => implementer
  - Rename TODO to `20261001-todo-1-DONE.md`, clean tmp files, merge `feat/initialize-project-info` to `main`, push `origin` if configured.
