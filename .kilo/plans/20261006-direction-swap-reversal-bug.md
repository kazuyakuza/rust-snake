# Global Plan — Rapid-Direction-Swap Reversal Bug (Circling Steers the Snake Into Its Own Body)

- Date: 2026-10-06
- TODO file: `.agent/todos/20261006/20261006-todo-1.md` (Pattern B: each `##` section = 1 task; 3 tasks)
- Trigger: user chat request. Branch: none exists yet; created in Step 2.
- Version bump: patch (0.3.2 → 0.3.3) executed inside Task 2's 4.2 cycle (bug-fix release), per Step 3 policy (fix → patch).

## Pre-Analysis (global)

Build/test execution requires Docker via the Alpine VM MCP (`alpine-vm`; verify with `vm_status`; the shared host→VM folder is `/rust-snake`) — no local Rust toolchain on the host. `docker compose run --rm build` builds only the release binary, so test runs use a one-off Docker invocation against the pinned image or a compose service with a different entrypoint — the architector decides the exact commands, following `docs/BUILD.md` and the MCP allowlist (prefix `docker`).

### Bug analysis (domain/terminal)

`src/game/state.rs`:

```rust
pub fn change_direction(&mut self, new_direction: Direction) -> bool {
    if is_immediate_reversal(self.current_direction, new_direction) { return false; }
    self.current_direction = new_direction;
    true
}
```

`src/terminal/game_loop.rs`:

```rust
pub fn tick(...) -> ... {
    apply_directions(state, directions);   // applies ALL buffered directions within ONE tick
    state.advance_one_step();
    ...
}
```

- The reversal check compares the candidate only against the tick's CURRENT direction — which buffered swaps may have already changed within the same tick. `Right→Up` turns, then `Left` is no longer an "immediate reversal" of `Up`, so it is accepted; the snake then steps Left onto its own neck → GameOver.
- Candidate fix approaches (architector MUST encode exactly ONE):
  - (a) Geometric neck-based rejection: reject any candidate direction whose offset would move the head onto a currently occupied body cell (minimum: the neck cell; optionally all body cells — architector picks one; the neck-based minimum is the smallest rule that kills the bug).
  - (b) Cap one ACCEPTED direction change per tick in the drain/apply path (`game_loop::apply_directions` or domain-side), so at most one turn happens between two moves.
  - Note on (a)-only vs (b): with `last_moved_direction`-style sequential checks (updated only on actual moves), the `[Up, Left]`-between-ticks sequence can STILL end moving Left into the neck, so a pure "check vs last moved direction without geometry" does not kill the bug — the chosen approach must make the `[Up, Left]`, `[Up, Left, Down]`, and full-circle sequences survive while legit per-tick double turns that head to FREE cells (e.g. `Right→Down`, a single 90° turn, or a U-shape over two ticks) keep working. A combined approach (neck geometry check on the final applied direction + possibly one-change-per-track cap) is acceptable.
- Constraints: keep-it-simple (brief §19); reversal rejection stays centralized in the domain (`src/game/`); the terminal layer changes ONLY if the chosen approach requires it; new functions must respect the max-2-params rule; source files must respect max-lines (200) / max-depth (2) rules.
- Files in play: `src/game/state.rs` (`change_direction`, `is_immediate_reversal`, possibly a tracking field), `src/game/direction.rs` (if a helper is needed), `src/terminal/game_loop.rs` (only if the approach requires it), `tests/direction_swap_reversal.rs` (new), plus docs (`README.md`, `docs/terminal-ui.md`, `.agent/project-info/context.md`) in Task 3.

## Tasks Workflow (steps 2–6)

- Step 2: Git Feature Branch Setup => implementer → branch `fix/circling-reversal-death`
- Step 3: Version Update => implementer — merged into Task 2's 4.2 (patch bump committed with the fix); no separate cycle.
- Task 1: 4.1b Analysis & Planning => architector
- Task 1: 4.2 Implementation => implementer (write the failing tests reproducing the bug; execute in VM Docker; confirm FAIL; commit)
- Task 1: 4.3 Code Review & Simplification => code-reviewer & code-simplifier (concurrent); 4.3-fix => implementer if fix plans produced
- Task 1: 4.4 Documentation => docs-specialist (module doc-comments for the new test file; AI-agent guidance)
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion => implementer (`[DONE]` on Task 1 heading; commit)
- Task 2: 4.1b Analysis & Planning => architector (dedicated 4.1–4.6 cycle; include the version bump as part of the implementation step)
- Task 2: 4.2 Implementation => implementer (fix the reversal logic in the domain per the plan's single approach; run new tests green; run FULL suite green in VM Docker; patch bump 0.3.2 → 0.3.3 committed with the fix)
- Task 2: 4.3 Code Review & Simplification => code-reviewer & code-simplifier; 4.3-fix => implementer
- Task 2: 4.4 Documentation => docs-specialist (module doc-comments for changed domain files; the user-facing docs are Task 3's body)
- Task 2: 4.5b Overall Plan Adherence => architector
- Task 2: 4.6 Task Completion => implementer (`[DONE]` on Task 2 heading; commit)
- Task 3: 4.1b Analysis & Planning => architector (docs + context update plan; no front-end)
- Task 3: 4.2 Implementation => implementer (README gameplay rules, `docs/terminal-ui.md` reversal-rule wording, `.agent/project-info/context.md` Recent Changes/Implementation Status/Next Steps per instructions.md "Critical Closing Step")
- Task 3: 4.3 Code Review & Simplification => code-reviewer & code-simplifier (docs-only: code-simplifier confirms no source simplification needed; code-reviewer checks doc accuracy vs the final code)
- Task 3: 4.4 Documentation => docs-specialist (this task's body IS documentation; if 4.2 already fully covers it, docs-specialist reviews/polishes rather than duplicating)
- Task 3: 4.5b Overall Plan Adherence => architector
- Task 3: 4.6 Task Completion => implementer (`[DONE]` on Task 3 heading; commit)
- Step 5: TODO File Completion => implementer (rename with `-DONE` suffix, tmp cleanup, ensure all committed, merge `fix/circling-reversal-death` → `main`, push `origin` ONLY)

## Pre-analysis per task

- **Task 1 (failing tests)**: new `tests/direction_swap_reversal.rs`. Build the initial state via `initial_setup()` (three-segment head-first snake facing Right at mid-board: head (41,40), neck (40,40), tail (39,40); food irrelevant far away — the initial board is 80×80 so short bursts stay well inside bounds). Domain-level cases: apply a swap sequence via `change_direction` then call `advance_one_step`; sequences must include at least: one tick `[Up, Left]` applied back-to-back then a step; `[Up, Left, Down]` then a step; a full circle applied one-per-tick over four ticks (Right→Up→Left→Down→Right, the continuous circling reported); assert `status() != GameOver`, `score() == 0`, snake length stays 3, and the snake's segments contain no self-overlap. Loop-level: same bursts through `game_loop::tick` with `Renderer::new(Vec::new())` — must not end in GameOver. Note headlessly, without the real timed loop, "circling" = repeated apply-one-direction-then-step cycles; both layers must be covered per the TODO. The tests must compile against the CURRENT code and assert the CORRECT behavior, so they MUST FAIL initially (bug reproduction confirmed by execution). DO NOT touch `src/` in Task 1.
- **Task 2 (fix)**: implement the single approach chosen by the architector in the domain; keep `game_loop.rs` unchanged unless the approach requires the apply-path adaptation. After the fix the Task-1 tests must go green; then run the FULL suite (`cargo test`) in the VM Docker — all 59 pre-existing + the new tests must pass. Version bump `0.3.2` → `0.3.3` in `Cargo.toml`, committed with the fix (restricted to this cycle).
- **Task 3 (docs)**: state the corrected rule in README's rules/gameplay section and `docs/terminal-ui.md`: the head can never begin a move going the direction opposite to the direction of its actual last move (i.e. it can never step onto the cell occupied by its neck/body), no matter how many direction presses are buffered between ticks. Update `.agent/project-info/context.md` per instructions.md.
- **Step 5 (completion)**: rename TODO with `-DONE`; no tmp artifacts expected (Docker runs happen inside the VM's mounted folder; `target/` and `dist/` are gitignored); review `git status` and `.gitignore` compliance before each commit; merge to `main`; push to `origin` only (push restricted to this step; if the VM/host blocks the push, notify the user and pause).
- Repo state at plan time: branch `main`, HEAD even with `origin/main`, working tree clean (verified `git status`).

## Notes

- From the user request: "generate testings to found and then fix this error" — the failing-test-first flow is explicit.
- `cargo test` full-suite target: all 59 existing + new tests green in the VM Docker run (first full-suite green expected in Task 2).
- The Alpine VM MCP (`alpine-vm`) must be verified with `vm_status` before each Docker/test run; if the VM is down, PAUSE and ask the user to start it.
