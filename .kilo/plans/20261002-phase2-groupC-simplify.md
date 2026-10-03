# Simplification Plan — Phase 2 Group C Documentation (workflow 4.3)

**Role:** code-simplifier (static review; Docker not available).
**Scope reviewed:** `docs/BUILD.md` (new, 64 lines), `README.md` edits, `.agent/project-structure.md`, `.agent/project-info/{tech,architecture,context}.md` edits — commits `6a89098`, `191ec3d`, `ae0beb5` on `feat/phase2-docker-windows-build` (HEAD `ae0beb5`).
**Authority plan:** `.kilo/plans/20261002-phase2-groupC-documentation.md`. This plan only trims/repeats-free wording; it does NOT alter approved decisions: the command `docker compose run --rm build` stays verbatim everywhere it appears, paths (`dist/snake.exe`, `docs/BUILD.md`), pinned image (`rust:1.98.1-slim-bookworm`), target (`x86_64-pc-windows-gnu`) and all status/honesty claims are preserved.
**Executor:** JUNIOR developer, 50% restriction. Apply steps exactly as written, in order. Each step is an atomic find/replace — zero judgment calls. If any `OLD` string is not found byte-exact, STOP and return a question to the caller.

---

## Findings

- **F1 (README ↔ docs/BUILD.md duplication — primary):** README `Build & Run` (lines 47–63) re-narrates `docs/BUILD.md` content — the Cargo-lock/git-ignore paragraph (≈ BUILD.md lines 46–49, near-verbatim), implicit image build (≈ BUILD.md workflow step 1 + prerequisites), transfer + manual-validation steps (≈ BUILD.md lines 51–62) — and internally repeats facts already carried by README's own `Project Structure` items (lines 78–81: Dockerfile/compose image, `Cargo.lock` generated-then-committed, `dist/` git-ignored). Review criterion (source plan Step 11): README must stay a compact summary + link.
- **F2 (README line 3 grammar fragment):** the approved replacement clause produced a subjectless fragment: `... are implemented; *covered by the Docker build infrastructure — see [Build & Run](#build--run)*.` Repair wording only (fact unchanged); drop the odd italics around the link.
- **F3 (context.md internal duplication, introduced by `ae0beb5`):** `Current Work Focus` bullet 2 ("Next up: user runs … then manual Windows validation …") restates `Immediate Next Steps` items 1–3 of the same file. Replace with a pointer.
- **F4 (version contradiction surfaced by `ae0beb5`):** `tech.md` line 28 now says `Cargo.toml` (v0.3.0) while line 5 still says version `0.2.0` (`Cargo.toml` is 0.3.0); `context.md` line 36 keeps "(now version `0.2.0`)" contradicting the same file's new Recent Changes bullet ("version bump to 0.3.0 (`4ac280a`)"). Single-source the version: remove the stale copies (0.3.0 remains in tech.md Status and context.md Recent Changes history).
- **F5 (architecture.md transition meta-note):** the bullet `The former "Planned additions (Phase 2)" (…) are now implemented and shown as the entries in the layout above.` is changelog noise in architecture.md (history belongs to context.md) and duplicates the layout tree. No other file references it (grep-verified). Delete.

**Reviewed, intentionally NOT changed:** `docs/BUILD.md` (matches approved draft; 64 lines, TOC-free is correct; every fact single-sourced; the target triple appears in prerequisites and in the `cargo build` explanation because both need it verbatim); pinned-tuple phrasing across tech/architecture/context status paragraphs (by-design perspective per file); README `Project Structure` items (one-liner-per-file convention); README's two "build does not run tests" disclaimers in Terminal UI / Project Structure sections (section-scoped guardrails, not duplicated narrative); context.md claim "all 10 tasks `[DONE]`" (becomes true at workflow step 4.6 — out of this step's scope).

---

## Steps

### S1 — README: condense `Build & Run` to summary + link (fixes F1)

Replace the entire content between the `## Build & Run` heading and the `## Project Structure` heading (everything after the heading line up to, but NOT including, `## Project Structure`) with exactly:

```markdown
## Build & Run

- Build the Windows executable with the single documented command (no host
  Rust toolchain needed — the pinned image builds implicitly from
  [`Dockerfile`](Dockerfile) via [`compose.yaml`](compose.yaml); define all
  build parameters there, not on the command line):

      docker compose run --rm build

- Artifact on the host: `dist/snake.exe` — copy this single file to a
  Windows machine and run it there; Windows runtime validation is a
  separate manual step, NOT part of the Docker build.
- Full prerequisites and step-by-step workflow (including `dist/`
  git-ignoring and `Cargo.lock` generation): [`docs/BUILD.md`](docs/BUILD.md).
```

Rules: the command line stays an indented code block exactly as shown; do not touch the TOC, `## Build & Run` heading text, or `## Project Structure` and its items. Facts removed from this section remain single-sourced: prerequisites/implicit build in `docs/BUILD.md`, image/target links + `Cargo.lock`/`dist/` handling in README `Project Structure` items and BUILD.md.

### S2 — README line 3: repair fragment (fixes F2)

OLD (line 3, second sentence of the paragraph):

`The core game model and its deterministic tests are implemented; *covered by the Docker build infrastructure — see [Build & Run](#build--run)*.`

NEW:

`The core game model and its deterministic tests are implemented; the Windows build is covered by the Docker build infrastructure — see [Build & Run](#build--run).`

Nothing else on line 3 changes.

### S3 — Commit S1+S2

1. `git add README.md`
2. `git status` — verify ONLY `README.md` is staged (gitignore compliance).
3. Commit message: `docs: condense readme build summary to avoid duplication with build guide`

### S4 — context.md: "Next up" bullet becomes a pointer (fixes F3)

OLD (bullet 2 of `## Current Work Focus`):

`- Next up: user runs \`docker compose run --rm build\` (first real compilation + \`Cargo.lock\` generation), then manual Windows validation of \`dist/snake.exe\` per brief §18.`

NEW:

`- Next up: see **Immediate Next Steps** below — first Docker build run, \`Cargo.lock\` commit, manual Windows validation per brief §18.`

Section keeps exactly two bullets. Bullet 1 of the section is NOT changed.

### S5 — tech.md: drop stale version from Stack line (fixes F4)

OLD (line 5):

`- Language: Rust (brief §2). Edition 2021, package \`snake\`, version \`0.2.0\`.`

NEW:

`- Language: Rust (brief §2). Edition 2021, package \`snake\`.`

The only remaining package-version statement in tech.md is `Status: \`Cargo.toml\` (v0.3.0) …` (line 28) — correct per `Cargo.toml`.

### S6 — context.md: drop stale version phrase from Phase 1A status bullet (fixes F4)

OLD (fragment inside `**Implemented (Phase 1A Group A — TODO tasks 1–4):**` bullet):

`the Cargo foundation (\`Cargo.toml\`, package \`snake\`, \`rand\` dep, now version \`0.2.0\`)`

NEW:

`the Cargo foundation (\`Cargo.toml\`, package \`snake\`, \`rand\` dep)`

Do NOT touch the Phase 1B Recent Changes bullet (`version bump \`e3c239b\` (→ \`0.2.0\`)`) — that is historical log, by design.

### S7 — architecture.md: delete transition meta-bullet (fixes F5)

Delete this entire line (the bullet between the repository-layout code block and the "Library/binary split" bullet):

`- The former "Planned additions (Phase 2)" (\`Cargo.lock\`, \`Dockerfile\`, \`compose.yaml\`, \`dist/\`) are now implemented and shown as the entries in the layout above.`

Keep the tree comment `├── dist/  # gitignored build output` and the existing bullet `- \`dist/\` is a host-mounted build output folder, ignored by git (\`.gitignore\`)` as-is.

### S8 — Commit S4–S7

1. `git add .agent/project-info/context.md .agent/project-info/tech.md .agent/project-info/architecture.md`
2. `git status` — verify ONLY those three files are staged (gitignore compliance).
3. Commit message: `docs: dedupe repeated build and version facts in project info`

### S9 — Verification (static only, no docker/cargo)

Run each check and confirm the expectation; any mismatch → STOP, report to caller:

1. `docker compose run --rm build` appears verbatim in `README.md` (Build & Run) and `docs/BUILD.md` — unchanged strings.
2. In `README.md`: `written back` → 0 matches; `Rust \`1.98.1\` + mingw-w64` → 0 matches (removed duplication); `docs/BUILD.md` → still linked from About, Build & Run.
3. `Cargo.lock` generation/commit fact now appears once per file section purpose: README `Project Structure` item only (not Build & Run), `docs/BUILD.md` container-details paragraph only.
4. `0.2.0` in `tech.md` → 0 matches; in `context.md` → only the historical Phase 1B bullet.
5. `Planned additions` in `.agent/` → 0 matches.
6. `docs/BUILD.md` untouched (64 lines, `git diff HEAD -- docs/BUILD.md` empty at the end of S8).
7. README still ≤ ~80 lines, TOC anchors unchanged (heading set not modified).

---

## Constraints for the implementer

- Markdown edits ONLY in `README.md`, `.agent/project-info/context.md`, `.agent/project-info/tech.md`, `.agent/project-info/architecture.md`. Do NOT touch `docs/BUILD.md`, `.agent/project-structure.md`, TODO files, or any code/config.
- No decisions changed: commands, paths, pinned image, target, IDE-agnostic tone, "build command does not run tests" honesty, "runtime validation is a separate manual step" (kept in both README and BUILD.md), and the task-8/9/10 deliverables (prerequisites, workflow, artifact location, transfer guidance, validation distinction — all still present in `docs/BUILD.md`).
- Real newlines in all written content (newline-prevention rule); keep the indented command block exactly as in S1.
- Stage only the listed files per commit; never stage `dist/` or `target/`.

## Non-findings recorded

- No vague placeholders, no duplicated command tables, no orphaned cross-links found. `docs/BUILD.md` matches the approved draft byte-for-byte in structure and passes the <100-line TOC rule.
