# Group A Plan — TODO tasks 1–4: Windows Build Infrastructure Core

**Source TODO:** `.agent/todos/20261001/20261001-todo-4.md` (tasks 1–4 only)
**Global plan (approved decisions, do not re-litigate):** `.kilo/plans/20261002-phase2-docker-windows-build.md`
**Branch:** `feat/phase2-docker-windows-build` — steps 2 & 3 already done (`4ac280a chore: bump version to 0.3.0`).
**Date:** 2026-10-02

---

## 4.1b — Pre-Analysis & Design Decisions

### Environment constraints (hard, verified)

- Docker is NOT available in this implementation context → **NO docker build/run/test may be executed**, ever.
- Rust/Cargo are NOT installed → **no cargo commands** may be executed.
- Therefore all implementation is **static file authoring`; verification is static (file-content review).
- No test execution. Out of scope: CI/CD, cloud builds, orchestration beyond Compose, Node/npm tooling, VM-in-Docker, Windows runtime testing, installer/signing/distribution, gameplay changes. Keep build infra deliberately small.

### Repo state (verified today)

- `Cargo.toml` v0.3.0, package `snake`, deps `rand 0.8`, `crossterm 0.29`. No `Cargo.lock` yet.
- `.gitignore` already contains `dist/` and `target/` — nothing to change for Group A.
- Working tree clean on `feat/phase2-docker-windows-build` at `4ac280a`.

### Group A design decisions (final, zero ambiguity)

1. **Target triple (Task 1): `x86_64-pc-windows-gnu`.**
   - Only practically cross-compilable Windows target from a Linux Docker container; `x86_64-pc-windows-msvc` needs Microsoft's proprietary `link.exe`/Windows SDK, which cannot run on Linux.
   - Linker: `mingw-w64` GCC (`x86_64-w64-mingw32-gcc`), available as a Debian `bookworm` apt package; configured via a **committed repo-root `.cargo/config.toml`** (explicit, reproducible, in-repo — NOT an image-side `~/.cargo/config.toml`). Cargo resolves `.cargo/config.toml` starting from the build working directory (`/project`), so the committed file is picked up automatically inside the container.
   - Target std libs installed via `rustup target add x86_64-pc-windows-gnu`.
   - Produces a standard standalone 64-bit Windows `.exe`; `crossterm 0.29` + `rand 0.8` are pure-Rust/std on this target (crossterm links WinAPI through mingw import libraries) — **no extra runtime DLL dependencies required**.
2. **Toolchain pinning (Tasks 1–2):** image tag **`rust:1.98.1-slim-bookworm`** — an exact-patch tag, i.e. maximum reproducibility (the floating `1.98` minor tag silently moves across point releases, so it is avoided; exact version + exact Debian distro). Rationale: Rust 1.98.x has been stable for over a month (patched 1.98.1 released 2026-09-03); 1.99.0 went stable only on 2026-10-01 (the day before this work) and is pinned OUT for the next phase to avoid the same-week-release risk (tag publication lag, missing hardening). **Never `latest`. The image is the versioning mechanism; NO `rust-toolchain.toml`** (single source of truth).
3. **Dockerfile (Task 2):** slim Debian Rust image + `apt-get install mingw-w64` + `rustup target add`. Compilation-only — no dev tooling beyond the compiler/linker, no runtime deps.
4. **compose.yaml (Tasks 3–4):** single service `build`:
   - `build: .` (uses the Dockerfile).
   - Project root mounted **read-write** at `/project` → `Cargo.lock` is written back to the host (brief §17 lists `Cargo.lock`).
   - `CARGO_TARGET_DIR=/tmp/target` (environment) → build intermediates stay inside the container; no `target/` junk reaches the host.
   - Separate mount `./dist:/out` → final artifact lands in host `dist/snake.exe` while appearing at `/out` inside the container (host-isolated, mirrors global plan decision).
   - Command = `mkdir -p /out` + `cargo build --release --target x86_64-pc-windows-gnu` + `cp` of the exe to `/out/snake.exe`. That one command is the entire release-build configuration (Task 4).
5. **Artifact name (Task 4): `snake.exe`** — Cargo auto-names the bin from package name `snake`; on a Windows target it is emitted as `snake.exe` inside `release/`. Final host path: **`dist/snake.exe`**.
6. **Target-rationale documentation placement (Task 1):** minimal, as short factual comment lines at the top of the `Dockerfile` (and one comment line in `compose.yaml`). Narrative docs (README `Build & Run` rewrite, prerequisites, workflow diagrams) are **deferred to Group C (tasks 8–10)** — no duplication created now. Rule compliance: these are explaining comments, not commented-out code.
7. **Committed files (exactly 3):** `.cargo/config.toml`, `Dockerfile`, `compose.yaml`. Nothing else. Directory `dist/` is NOT committed (gitignored).

---

## Exact file contents (Group A)

### File 1 — `.cargo/config.toml` (repo root; `.cargo` is a new top-level dir, not gitignored, must be committed)

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

That is the entire file. No other sections.

### File 2 — `Dockerfile` (repo root)

```dockerfile
# Windows cross-compilation build image.
# Target: x86_64-pc-windows-gnu — the only practically cross-compilable
# Windows target from Linux (the msvc target needs Microsoft's link.exe
# and Windows SDK, unavailable here). mingw-w64 GCC links the binary;
# the result is a standard, standalone 64-bit Windows .exe.
# The pinned image tag is the Rust toolchain versioning mechanism;
# no rust-toolchain.toml is used.
FROM rust:1.98.1-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /project
```

Notes bound to this content:
- `mingw-w64` in Debian bookworm provides `/usr/bin/x86_64-w64-mingw32-gcc` (exactly the linker name in `.cargo/config.toml`).
- No `EXPOSE`, no `USER` juggling, no entrypoint, no `cargo install` tools, no caching plugins — image is compile-focused per TODO task 2.

### File 3 — `compose.yaml` (repo root)

```yaml
# Windows release build workflow: docker compose run --rm build
# Output on the host: dist/snake.exe (artifact only — target/ stays
# inside the container; Cargo.lock is written back to the project root).
services:
  build:
    build: .
    working_dir: /project
    environment:
      CARGO_TARGET_DIR: /tmp/target
    volumes:
      - .:/project
      - ./dist:/out
    command:
      - bash
      - -c
      - |
        mkdir -p /out
        cargo build --release --target x86_64-pc-windows-gnu
        cp /tmp/target/x86_64-pc-windows-gnu/release/snake.exe /out/snake.exe
```

Notes bound to this content:
- No `version:` key (obsolete in the Compose spec).
- Service name `build` → invocation is exactly `docker compose run --rm build` (brief §3).
- The `mkdir -p /out` first line guarantees the host `./dist` dir and its mount exist even from a clean checkout.
- The command uses `--release` (Task 4 release profile) and an explicit `--target` (Task 1 explicit target — never host defaults).

### Shell commands executed inside the container (documentation only — implementer must NOT run these)

1. `mkdir -p /out`
2. `cargo build --release --target x86_64-pc-windows-gnu`
3. `cp /tmp/target/x86_64-pc-windows-gnu/release/snake.exe /out/snake.exe`

The container exits cleanly after the `cp` (last line of the command); `--rm` removes it.

---

## 4.2 — Implementation steps (implementer, in order)

1. **Verify context:** read `AGENTS.md`, `.agent/WORKFLOWS.md`, `.agent/project-structure.md`, `.agent/project-info/instructions.md`, the TODO file, and this plan fully before any change.
2. **Verify git state (hard gate):** run `git status` and `git log --oneline -3`; branch must be `feat/phase2-docker-windows-build` with HEAD `4ac280a` and a clean tree. If anything differs → STOP and report to caller. (Do not branch/switch — restricted to already-done step 2.)
3. **Create `.cargo/config.toml`** at repo root with exactly the File 1 content (real newlines, no literal `\n`).
4. **Gitignore compliance:** read `.gitignore`; run `git status`; confirm `.cargo/config.toml` matches no ignore pattern. Stage it and check what is staged: only `.cargo/config.toml`. Then commit:
   ```
   git commit -m "build: configure mingw-w64 linker for windows-gnu target"
   ```
5. **Create `Dockerfile`** at repo root with exactly the File 2 content.
6. **Gitignore compliance** again (`git status`; only `Dockerfile` new to stage). Commit:
   ```
   git commit -m "build: add Docker image for Windows GNU cross-compilation"
   ```
7. **Create `compose.yaml`** at repo root with exactly the File 3 content.
8. **Gitignore compliance** again (`git status`; only `compose.yaml` new to stage; `dist/`, `target/` must NOT appear). Commit:
   ```
   git commit -m "build: add compose build service producing dist/snake.exe"
   ```
9. **Do NOT:** run docker/cargo, create dist/ manually, touch `.gitignore`, `Cargo.toml`, `src/`, `tests/`, README, docs, or project-info files (docs are Group C / step 5 of the global plan). Do not push (restricted to step 5).
10. **Report back:** summary of what was done, git log of the three commits (`git log --oneline -5`).

---

## 4.3 — Review (code-reviewer + code-simplifier, static only)

Reviewer must verify, against this plan and the rules:

- File contents match this plan exactly (diff-level), files named exactly `.cargo/config.toml`, `Dockerfile`, `compose.yaml` at repo root, no extra files created.
- Rules adherence: no commented-out code (comments-that-explain are legal); self-documenting; no unnecessary abstraction; scope guard respected (no gameplay/CI/docs changes leaked in).
- Compose format sanity: valid YAML (2-space indent, service map, list `command`), mount paths consistent (`/project` ↔ Dockerfile `WORKDIR`, `/out` ↔ cp destination, `CARGO_TARGET_DIR=/tmp/target` ↔ cp source path `/tmp/target/x86_64-pc-windows-gnu/release/snake.exe`).
- `.cargo/config.toml` linker value equals Debian-bookworm mingw binary name `x86_64-w64-mingw32-gcc`.
- No gitignore-pattern file is committed (`git status` clean afterwards; `dist/`, `target/`, `.cargo/` config not ignored).
- Simplifier pass: confirm nothing further can be removed (image stays compile-only; compose has no extra services/keys). If ADHERENT and nothing to simplify, report so — do NOT invent changes.

---

## 4.4 — Documentation pass (docs-specialist, MINIMAL for Group A)

- **No edits in Group A.** Full docs (README `Build & Run` rewrite, prerequisites, workflow, project-structure/project-info updates) belong to Group C (tasks 8–10) and post-phase step 5 per the global plan.
- Read-only verification only: confirm that (a) the Dockerfile/compose.yaml comments state the target rationale (Task 1 deliverable) and there is no other provisional doc added to README or `docs/` in this group; (b) report once that README `Build & Run` still describes the *planned* workflow and will be rewritten in Group C — that is expected, not a defect.

---

## 4.5b — Architector adherence check (static; 4.1b/4.5a N/A — no front-end tasks in this TODO)

Verifier executes without docker/cargo, purely by reading files and running `git log --oneline`:

| Check | Expectation |
|---|---|
| TODO task 1 (target defined, documented) | Target triple + rationale present as Dockerfile header comments; target named explicitly in `.cargo/config.toml` header keys, `compose.yaml` command, and README plan (Group C pending); no `latest`, no `rust-toolchain.toml` added |
| TODO task 2 (Docker image) | `Dockerfile` exists at repo root; pinned `rust:1.98.1-slim-bookworm`; `mingw-w64` installed; `rustup target add x86_64-pc-windows-gnu`; no runtime/dev tooling; no apt cache left behind |
| TODO task 3 (Compose workflow) | `compose.yaml` exists; single `build` service; `build: .`; root rw mount; `CARGO_TARGET_DIR=/tmp/target`; `./dist:/out`; invocation is exactly `docker compose run --rm build` |
| TODO task 4 (release build) | Command contains `cargo build --release --target x86_64-pc-windows-gnu` and cp of `snake.exe` → artifact ends at host `dist/snake.exe`; nothing container-resident needed after |
| Reproducibility (TODO §Ensure Reproducibility) | Toolchain pinned by image tag; target explicit; build deps explicit (mingw-w64); build command explicit; artifact location explicit |
| Scope (TODO §Implementation Constraints / Out of Scope) | No CI/CD, no orchestration beyond Compose, no npm, no VM-in-Docker, no runtime testing hooks, no gameplay change, only 3 new files |
| Git | Branch `feat/phase2-docker-windows-build`; 3 Group A commits atop `4ac280a`; no push happened |
| Generic rules | gitignore compliance at each commit; file/commit boundaries respected |

- If all checks pass, architector records an adherence report at `.kilo/plans/20261002-phase2-groupA-adherence.md` (allowed: plan files are architector-writable) with verdict **ADHERENT** and the table above plus findings. If any check fails, architector opens a corrective sub-plan for the implementer — no silent fixes.

---

## 4.6 — TODO update + commit (implementer)

1. Edit `.agent/todos/20261001/20261001-todo-4.md` **only** by adding `[DONE]` to the headings of tasks 1–4:
   - `### 1. Define the Windows Build Target` → `### 1. [DONE] Define the Windows Build Target`
   - `### 2. Create the Docker Build Image` → `### 2. [DONE] Create the Docker Build Image`
   - `### 3. Create the Docker Compose Build Workflow` → `### 3. [DONE] Create the Docker Compose Build Workflow`
   - `### 4. Configure the Release Build` → `### 4. [DONE] Configure the Release Build`
   - Do NOT touch tasks 5–10 or any other line of the file (overwrite-prevention rule).
2. Gitignore compliance: read `.gitignore`, `git status`, verify no `.gitignore`-matching file is staged.
3. Commit only this TODO change:
   ```
   git commit -m "docs: mark phase 2 group A tasks 1-4 done"
   ```
4. Do not push, do not rename the TODO file (renaming happens in global-plan step 5).

---

## Verification of the plan against original TODO tasks 1–4

- Task 1 → encoded in decisions 1, 2, 6 and checked in 4.5b row 1. ✓
- Task 2 → File 2 (Dockerfile) + decision 3, checked in 4.5b row 2. ✓
- Task 3 → File 3 (compose.yaml) + decision 4, checked in 4.5b row 3. ✓
- Task 4 → decision 5 + container command lines, checked in 4.5b row 4. ✓
- Constraints honored: static-only authoring (no docker/cargo/test), small infra (3 files), docs deferred to Group C. ✓
