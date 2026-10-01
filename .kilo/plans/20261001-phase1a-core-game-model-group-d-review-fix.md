# Review Fix Plan — Phase 1A, Group D

## Rejected deviation

`README.md` contains a present-tense claim that the project is already compiled inside Docker and produces `dist/snake.exe`. The **Build & Run** status bullet correctly states the Docker build is planned for the next phase, but the intro paragraph contradicts it. This violates the review criterion **“no fabricated features / no claims that Docker build works.”**

The original plan allowed keeping lines 1–3 verbatim, but that allowance conflicts with the same plan’s requirement that Docker never be described as already working. The fix below resolves the contradiction with the smallest possible edit.

## Exact fix

Edit `README.md` only.

### 1. Intro paragraph (line 3)

**Current:**

```text
Rust Snake is a terminal Snake game written in Rust, built by AI agents through the Critical Workflow. It is compiled inside Docker for Windows, and the resulting `dist/snake.exe` runs directly in the Windows terminal.
```

**Replace with:**

```text
Rust Snake is a terminal Snake game written in Rust, built by AI agents through the Critical Workflow. The core game model and its deterministic tests are implemented; a Docker-based Windows build that produces `dist/snake.exe` is planned for the next phase.
```

### 2. About this Project, third paragraph

**Current:**

```text
Docker is used only as the compile environment; the build output runs directly on Windows (see [Build & Run](#build--run)).
```

**Replace with:**

```text
A Docker-based Windows build that produces `dist/snake.exe` is planned for a later phase (see [Build & Run](#build--run)).
```

## Scope fence

- No source code edits.
- No test file edits.
- No `Cargo.toml` changes.
- No new dependencies.
- No commits in this fix step; apply during the workflow’s fix/ completion pass.

## Verification

- Final section order remains: `# Rust Snake` intro → Attention AI Agents → Table of Contents → About this Project → Game Rules & Controls → Build & Run → Project Structure → AI Agents.
- The **Build & Run** status bullet remains the single source of current-phase truth.
- No base-project wording remains (already verified).
- No new fabricated feature claims are introduced.
