# Implementation Plan — Phase 1B Group C: Start Screen, Game-Over Flow, and `main.rs` Wiring (TODO Tasks 5, 6)

- **Sub-step**: 4.1b of Group C (Tasks 5 + 6 + the `src/main.rs` wiring that connects Groups A/B/C) — produced by the Architector.
- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` (Pattern C; ONLY tasks 5, 6 + "Implementation Constraints" + "Out of Scope" are in scope for this group; `main.rs` orchestration that connects Groups A/B/C).
- **Front-end input (mandatory follow)**: `.kilo/plans/20261001-phase1b-terminal-game-groupc-frontend-spec.md` (cited below as `SPEC §n`; it is the authoritative behavioral contract for this group).
- **Global plan**: `.kilo/plans/20261001-phase1b-terminal-game.md`.
- **Group A/B baseline**: `.kilo/plans/20261001-phase1b-terminal-game-groupa.md` + `.kilo/plans/20261001-phase1b-terminal-game-groupb.md`; the committed primitives and loop are the frozen start state this plan completes.
- **Branch**: `feat/phase1b-terminal-game`. It already exists; the version is already `0.2.0` (Group B finished: commits `80ae888`…`c4661fb`). NO branch creation/switching here. NO version bump here. NO push. NO merge.
- **Implementer level**: JUNIOR, hard-blocked from scope/architecture decisions; follow this plan exactly; STOP and ask the caller if anything is ambiguous or impossible.
- **Verification mode**: NO local Rust toolchain, NO Docker. There is **no compile/test execution** in this group. Verification = static review only (order in §10). Authored tests execute in Phase 2 (Docker). Do not author any test in this group (Group D owns tests).

---

## 0. Scope Guardrails (binding)

### 0.1 In scope (this group, sub-step 4.2 implementation)

| TODO task (20261001-todo-3.md) | Deliverable | File(s) |
|---|---|---|
| Task 5 — Implement the Start Screen | Start-screen helper + blocking any-key wait | `src/main.rs` (private `show_start_screen`, `wait_for_any_key_press`) |
| Task 5 — State transition | The ONLY production `start_playing()` call site | `src/main.rs` |
| Task 6 — Implement Game Over Flow | Game-over screen helper + exit wait | `src/main.rs` (private `show_game_over_screen`, reuse of `wait_for_any_key_press`) |
| Wiring | `main` orchestration: setup → enable → start screen → loop → game-over screen → cleanup/exit | `src/main.rs` |
| Lifecycle extension | One public `output()` accessor so screen helpers/Renderer write through the same stream | `src/terminal/lifecycle.rs` (extend) |
| Structure map | Reflect the `main` wiring + lifecycle extension | `.agent/project-structure.md` |
| README structure bullets | Mention the completed wiring | `README.md` |

### 0.2 Out of scope (do NOT touch, do NOT create)

- `src/terminal/screens.rs` or any new module file — SPEC §3 explicitly says "No new module files are introduced"; screens live as private helpers in `src/main.rs`. This resolves the "where does the screen code live" question: inside `main.rs`.
- Anything in Groups A/B files beyond the ONE accessor: `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/game_loop.rs`, `src/terminal.rs` — FROZEN, zero modifications (SPEC §14; note that SPEC §14 names `input.rs` for an exception, but that exception is §4's lifecycle change only — `input.rs` is NOT touched in this group).
- `src/game.rs`, `src/game/**` (all 8 files), `src/lib.rs`, `tests/**` (all 6 files), `Cargo.toml`, `.gitignore` — FROZEN.
- Test authoring (Group D, TODO Task 10); flow validation (Group D, TODO Task 9). No new test file, no test module.
- `Dockerfile`, `compose.yaml`, `Cargo.lock`, `dist/` (Phase 2). No new dependencies, crates, or feature flags.
- Restart, pause, menus, colors, difficulty, config, persistence — forbidden by the TODO "Implementation Constraints" (SPEC §11 compliance table).
- Marking TODO tasks 5/6 as `[DONE]` — happens at sub-step 4.6, NOT in 4.2.

### 0.3 Files NOT to modify

`src/lib.rs`, `src/game.rs`, `src/game/**` (all 8 files), `src/terminal/renderer.rs`, `src/terminal/input.rs`, `src/terminal/game_loop.rs`, `src/terminal.rs`, `tests/**` (all 6 files), `Cargo.toml`, `.gitignore`, `.agent/project-info/*` (phase close-out happens later; see §7).

`docs/terminal-ui.md` and `README.md` are touched ONLY where this plan explicitly says so: `README.md` gets the two structure-bullet edits in Step C5 (sub-step 4.2); `docs/terminal-ui.md` and broader README edits are the CONTENT GUIDANCE in §7 and are executed only by docs-specialist at sub-step 4.4.

---

## 1. Pre-Analysis — Decisions Encoded (no implementer judgment required)

### 1.1 Current state (verified in-tree)

- Group A primitives committed and FROZEN:
  - `src/terminal/lifecycle.rs` (60 lines): `TerminalHandle<W: Write>` private `output: W` field; `pub fn enable(output: W) -> io::Result<TerminalHandle<W>>` (raw mode → alt screen → hide cursor, best-effort rollback D7); `pub fn disable(&mut self) -> io::Result<()>` (restore commands + raw-mode off, error precedence D8); `Drop` guard (`let _ = self.disable();`).
  - `src/terminal/renderer.rs` (108 lines): `Renderer<W: Write>` with `pub fn new(output: W) -> Renderer<W>` and `pub fn render(&mut self, state: &GameState) -> io::Result<()>`. NOTE: `Renderer` OWNS its `output: W`; passing `&mut Stdout` creates a `Renderer<&mut Stdout>` that borrows the handle's stream — this is exactly the SPEC §6/§8.1 mechanism.
  - `src/terminal/input.rs` (81 lines): `map_key_event_to_direction`, private `is_key_press`, `drain_arrow_event`, private `pressed_arrow_key`/`is_arrow_key_press`/`is_arrow_key`, plus Group B's `drain_arrow_directions`/`read_arrow_direction`.
- Group B loop committed and FROZEN: `src/terminal/game_loop.rs` (63 lines): `pub fn run_playing_loop<W: Write>(state: &mut GameState, renderer: &mut Renderer<W>) -> io::Result<()>` (precondition `status() == Playing`; final losing frame rendered by the last `tick` before the `while` exits) and `pub fn tick(...) -> io::Result<GameStatus>`.
- `src/main.rs` is exactly `fn main() {}` (1 line) — replaced in this group.
- Domain API verified in-tree (`src/game/state.rs`, `src/game/setup.rs`): `GameState::{new(GameStateSetup), snake(), food(), score() -> i32, status(), change_direction, advance_one_step(), start_playing(), enter_game_over()}`, `GameStatus::{WaitingToStart, Playing, GameOver}` (derives `PartialEq`, `Copy`), `WIDTH = 40`, `HEIGHT = 25`. The construction entry point is `crate::game::setup::{initial_setup(), GameStateSetup}`.
- Environment: crossterm `0.29` already in `Cargo.toml`; `event::poll(Duration)` is by-value (D-verified); `KeyEventKind::Press` is the filter for presses (on Windows console only Press/Release occur, D/K201 note); `package name = "snake"` so `main.rs` reaches the library as `snake::...` (same as `tests/` precedent).
- No compile/test execution possible: verification is static only.

### 1.2 Line-budget check (rule `max-lines-per-file` ≤200)

- `main.rs` after rewrite ≈ 60 total (~50 code) — far within budget.
- `lifecycle.rs` after adding ~4 lines: ≈ 68 — within budget.
- **Decision: NO new files** (SPEC §3); both edits are in-place.

### 1.3 Behavioral decisions (continuing the Group A/B D-numbering; each answer is binding)

| # | Question | Decision |
|---|---|---|
| D26 | Wait-helper structure vs SPEC §9 literal snippet | The literal snapshot (`loop` → `if let Event::Key` → `if kind == Press`) nests **3** blocks and breaks the max-depth-2 rule. Implement the SAME behavior restructured into: `loop { let event = event::read()?; if is_any_key_press(event) { return Ok(()); } }` with a private predicate `fn is_any_key_press(event: Event) -> bool` (match-based, single-section). Behavior identical to SPEC §9: blocks on `event::read()`, returns `Ok(())` only on a key event with `KeyEventKind::Press`, discards releases/repeats/resize/mouse/other events, propagates `io::Error`. Precedent: Group A's review fix restructured a spec sketch into named predicates to satisfy the single-section rule. Bounded, documented deviation for reviewers (see §9). |
| D27 | Screen text storage | Private `const`s in `main.rs`: `LINE_BREAK: &str = "\r\n"` (mirrors Group A D1), `START_MESSAGE: &str = "Press any key to start"`, `GAME_OVER_TITLE: &str = "GAME OVER"`, `EXIT_PROMPT: &str = "Press any key to exit"`, `SCORE_PREFIX: &str = "Score: "`. Shared helper `write_line(output, content)` writes `content + \r\n` (2 params, ≤2 rule). Blank lines are `write_line(output, "")`. |
| D28 | Score formatting | `fn score_line(score: i32) -> String { format!("{SCORE_PREFIX}{score}") }` — output is exactly `Score: N`. A second private `SCORE_PREFIX` const lives in `main.rs` mirroring `renderer.rs` (both private → no coupling; do NOT make either public or move to a shared place). |
| D29 | `output()` accessor | Exact SPEC §4 snippet, placed in the public `impl<W: Write> TerminalHandle<W>` block AFTER `disable` (the enable/disable pair stays adjacent; all three public members grouped: `enable`, `output`, `disable` order does not matter — the decided order is `enable`, `disable`, then `output` between them? NO — decided: append `output` after `disable`, before the private `write_restore_commands`). Doc comment `///` on it; no behavior change to enable/disable/Drop. Public because `main.rs` legitimately needs it (prefer-private rule exception, SPEC §5 acceptance). |
| D30 | Cleanup guarantee | `terminal` is declared at the top of `main` and stays in scope through every return path; Rust drops it on both `Ok(())` and `Err` returns (and on panic unwind), so the `Drop` guard (D8) always restores the terminal. NO explicit `disable()` call in `main` (SPEC §8: "Return Ok(()); TerminalHandle Drop restores the terminal"). |
| D31 | Output stream identity | `TerminalHandle::enable(stdout())` — `stdout()` from `std::io`, typed `TerminalHandle<Stdout>`. One stream for everything: screens (via `output()`), Renderer (via `Renderer<&mut Stdout>`), and Drop restore. No second stream, no `stderr`. |
| D32 | Start key consumption | The key that ends `wait_for_any_key_press` is consumed by `event::read()` and is NOT replayed into the game: no code needed, but document — the snake starts in `Direction::Right` (initial setup) regardless of which key was pressed. |
| D33 | Loop preconditions | `state.start_playing()` is called immediately before creating the `Renderer` and entering `run_playing_loop`, satisfying its `status() == Playing` precondition (SPEC §6). There is no status check in `main` — the ordering guarantees it. |
| D34 | Error screen | None. Any `io::Error` propagates via `?`; Rust prints the default error message AFTER the Drop guard has restored the terminal (SPEC §10). No retries, no custom messages. |
| D35 | Borrow ordering (SPEC §8.2) | (a) `terminal` declared before `renderer`; (b) `Renderer::new(terminal.output())` yields `Renderer<&mut Stdout>` — the renderer holds a mutable borrow of the handle's stream; (c) `renderer`'s borrow ends after `run_playing_loop` (NLL), so `show_game_over_screen(terminal.output(), ...)` may re-borrow; (d) nothing else borrows `terminal` between these points. The SPEC §8.1 shape must be kept literally in this order. |
| D36 | Command macros | Only `queue!` + explicit `flush()` — NEVER `execute!` (SPEC §14). Screen writes use plain `write!` through `&mut W`. |
| D37 | Doc-comment style | Group A/B precedent: `//!` module docs + `///` on public items in library files; in `main.rs` the helpers are private file-local (`//!` module doc + `///` acceptable on the two screen helpers for flow clarity, none on `write_line`/`score_line`/`is_any_key_press`). |

### 1.4 crossterm 0.29 API surface used (verified against docs.rs 0.29.0)

- `crossterm::event::read() -> io::Result<Event>` — blocks until an event is available (raw mode already enabled by `TerminalHandle::enable`).
- `crossterm::event::{Event, KeyEventKind}` — `Event::Key(KeyEvent)`, `KeyEvent.kind: KeyEventKind` with `Press | Repeat | Release` variants. (NOTE: this helper ONLY needs `Event` + `KeyEventKind`; `KeyEvent` itself is not named — no import needed for it.)
- `crossterm::terminal::{Clear, ClearType}` — `Clear(ClearType::All)` wipes the alternate screen; `ClearType` variant `All` exists. (Same module that already provides `EnterAlternateScreen` in `lifecycle.rs`.)
- `crossterm::cursor::MoveTo(col: u16, row: u16)` — `MoveTo(0, 0)` = home; already used by the renderer.
- `crossterm::queue!` — writes commands into a `Write` without flushing.
- **No new dependency; `Cargo.toml` untouched.**

### 1.5 Domain API contract (verified in-tree, DO NOT redesign)

The binary may call ONLY: `snake::game::setup::{initial_setup, GameStateSetup}`, `snake::game::state::{GameState, (new)} , .start_playing(), .score()` — plus, through the frozen loop/renderer/handle, `change_direction`/`advance_one_step`/`status` transitively. Direct calls to `change_direction`, `advance_one_step`, collision/food/growth internals in `main.rs` are FORBIDDEN (SPEC §11; verified by grep audit C6).

---

## 2. Target State — File-by-File Structure

Line budgets (rule ≤200 lines/file, ideal ≤125 code): `main.rs` ≤ 80 total (~55 code), `lifecycle.rs` ≤ 70 total. If an implementation draft exceeds a budget, the implementer STOPs and reports — do not split files without approval.

### 2.1 `src/terminal/lifecycle.rs` (extend — the ONLY lifecycle change)

| Item | Specification |
|---|---|
| Imports | UNCHANGED — existing imports already cover the new method (`Write` in scope; no crossterm additions). |
| New public method | `pub fn output(&mut self) -> &mut W { &mut self.output }` — with doc comment `/// Borrow the wrapped output for screen writing.` (SPEC §4 exact body). |
| Where | Inside the existing `impl<W: Write> TerminalHandle<W>` block, immediately AFTER the `disable` method's closing brace and BEFORE `write_restore_commands`. |
| Frozen content | `enable`, `disable`, `Drop`, `write_setup_commands`, `report_first_error` — zero modifications (D6/D7/D8 intact; SPEC §4: "does not change enable/disable/Drop behavior"). |

### 2.2 `src/main.rs` (rewrite — TODO Tasks 5, 6 + wiring)

| Item | Specification |
|---|---|
| Module doc (`//!`) | One-two lines: binary entry point; wires initial setup, terminal lifecycle, the two screens, the playing loop, and exit cleanup. |
| Imports (exact) | `use std::io::{self, stdout, Write};` · `use crossterm::{cursor::MoveTo, event::{self, Event, KeyEventKind}, queue, terminal::{Clear, ClearType}};` · `use snake::game::setup::initial_setup;` · `use snake::game::state::GameState;` · `use snake::terminal::game_loop::run_playing_loop;` · `use snake::terminal::lifecycle::TerminalHandle;` · `use snake::terminal::renderer::Renderer;` |
| Constants (private) | `const LINE_BREAK: &str = "\r\n";` · `const START_MESSAGE: &str = "Press any key to start";` · `const GAME_OVER_TITLE: &str = "GAME OVER";` · `const EXIT_PROMPT: &str = "Press any key to exit";` · `const SCORE_PREFIX: &str = "Score: ";` |
| `fn main() -> io::Result<()>` | EXACT SPEC §8.1 body (§3.1 below): `GameState::new(initial_setup())` → `TerminalHandle::enable(stdout())?` → `show_start_screen(terminal.output())?` → `wait_for_any_key_press()?` → `state.start_playing()` → `let mut renderer = Renderer::new(terminal.output());` → `run_playing_loop(&mut state, &mut renderer)?` → `show_game_over_screen(terminal.output(), state.score())?` → `wait_for_any_key_press()?` → `Ok(())`. |
| Screen helper 1 (private) | `fn show_start_screen(output: &mut impl Write) -> io::Result<()>` — `queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;` → `write_line(output, START_MESSAGE)?;` → `output.flush()` (SPEC §5.1: one left-aligned line at home; no score, no board). |
| Screen helper 2 (private) | `fn show_game_over_screen(output: &mut impl Write, score: i32) -> io::Result<()>` — `queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;` then in EXACT order: `write_line(output, GAME_OVER_TITLE)?;` → `write_line(output, "")?;` → `write_line(output, &score_line(score))?;` → `write_line(output, "")?;` → `write_line(output, EXIT_PROMPT)?;` → `output.flush()` (SPEC §7.2: title, blank, score, blank, prompt; no board redraw). |
| LSM helper (private) | `fn write_line(output: &mut impl Write, content: &str) -> io::Result<()>` — `write!(output, "{content}{LINE_BREAK}")`. |
| Score formatter (private) | `fn score_line(score: i32) -> String` — `format!("{SCORE_PREFIX}{score}")` (D27/D28). |
| Key-wait helper (private) | `fn wait_for_any_key_press() -> io::Result<()>` — per SPEC §9 behavior, D26 structure (§3.2). |
| Press predicate (private) | `fn is_any_key_press(event: Event) -> bool` — `match event { Event::Key(key_event) => key_event.kind == KeyEventKind::Press, _ => false }`. |
| Visibility | Everything except `main` is PRIVATE (prefer-private rule; SPEC §14 "Keep screen helpers private"). |
| Ordering convention | Functions in this order: `main` → `show_start_screen` → `wait_for_any_key_press` → `show_game_over_screen` → `write_line` → `score_line` → `is_any_key_press`; constants above `main`. |

---

## 3. Code Sketches for Tricky Parts (encodes all decisions; local variable names may vary only if recognizable)

### 3.1 `main.rs` — the orchestration + screens (SPEC §8.1 shape kept verbatim)

```rust
//! Binary entry point: wires initial setup, terminal lifecycle, the start and
//! game-over screens, the playing loop, and exit cleanup into one flow.

use std::io::{self, stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
};
use snake::game::setup::initial_setup;
use snake::game::state::GameState;
use snake::terminal::game_loop::run_playing_loop;
use snake::terminal::lifecycle::TerminalHandle;
use snake::terminal::renderer::Renderer;

const LINE_BREAK: &str = "\r\n";
const START_MESSAGE: &str = "Press any key to start";
const GAME_OVER_TITLE: &str = "GAME OVER";
const EXIT_PROMPT: &str = "Press any key to exit";
const SCORE_PREFIX: &str = "Score: ";

/// Demonstrate the complete gameplay flow: start-screen wait, playing loop,
/// game-over screen, exit wait.
fn main() -> io::Result<()> {
    let mut state = GameState::new(initial_setup());
    let mut terminal = TerminalHandle::enable(stdout())?;

    show_start_screen(terminal.output())?;
    wait_for_any_key_press()?;
    state.start_playing();

    let mut renderer = Renderer::new(terminal.output());
    run_playing_loop(&mut state, &mut renderer)?;

    show_game_over_screen(terminal.output(), state.score())?;
    wait_for_any_key_press()?;

    Ok(())
}

/// Show the start prompt and block until any key is pressed, then begin play.
fn show_start_screen(output: &mut impl Write) -> io::Result<()> {
    queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;
    write_line(output, START_MESSAGE)?;
    output.flush()
}

/// Show the final score and block until any key is pressed, then exit.
fn show_game_over_screen(output: &mut impl Write, score: i32) -> io::Result<()> {
    queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;
    write_line(output, GAME_OVER_TITLE)?;
    write_line(output, "")?;
    write_line(output, &score_line(score))?;
    write_line(output, "")?;
    write_line(output, EXIT_PROMPT)?;
    output.flush()
}

fn write_line(output: &mut impl Write, content: &str) -> io::Result<()> {
    write!(output, "{content}{LINE_BREAK}")
}

fn score_line(score: i32) -> String {
    format!("{SCORE_PREFIX}{score}")
}

/// Block until the next key press (any key); ignore releases, repeats, and
/// non-key events such as resize.
fn wait_for_any_key_press() -> io::Result<()> {
    loop {
        let event = event::read()?;
        if is_any_key_press(event) {
            return Ok(());
        }
    }
}

fn is_any_key_press(event: Event) -> bool {
    match event {
        Event::Key(key_event) => key_event.kind == KeyEventKind::Press,
        _ => false,
    }
}
```

Notes on this sketch:

- Doc comments on `main` and the two screens are flow-level `///` comments (allowed, minimal, per D37); none on the other helpers.
- `show_start_screen(terminal.output())`: the argument expression is already `&mut Stdout`, exactly matching the `&mut impl Write` parameter — no extra borrow syntax needed. Same for the game-over call.
- `is_any_key_press` consumes the `Event` by value (it is dropped; no reuse) and matches in one section — the kind comparison is inside the match arm, not in a boolean chain (single-section rule).
- Rule checks: `main` body 11 lines; both screens ≤ 8 body lines; every `if` single-section; nesting depth ≤ 1 inside every function here; all fns ≤ 2 params.

### 3.2 `lifecycle.rs` — the inserted accessor (exact placement)

```rust
    /// Restore the terminal: cursor visible, main screen buffer, raw mode off.
    pub fn disable(&mut self) -> io::Result<()> {
        let restore_result = self.write_restore_commands();
        let raw_mode_result = terminal::disable_raw_mode();
        report_first_error(restore_result, raw_mode_result)
    }

    /// Borrow the wrapped output for screen writing.
    pub fn output(&mut self) -> &mut W {
        &mut self.output
    }
```

- Only these 6 lines are added to the file; everything else stays byte-identical (Group A frozen).
- `&mut self.output` borrows the field for the returned lifetime; callers cannot use the stream again until the borrow ends — which is exactly the SPEC §8.2 borrow-order contract (D35).

---

## 4. Step-by-Step Implementation (execute in order; every step ends in a verification)

Legend: numbered steps are atomic and verifiable. NO build/test commands exist (no toolchain). Sub-step labels map to Critical Workflow 4.2 execution for this group.

### Step C0 — Preconditions check (no file changes)

1. Run `git status` and `git branch --show-current`.
2. Verify: branch is `feat/phase1b-terminal-game`; working tree has ONLY the untracked file `.kilo/plans/20261001-phase1b-terminal-game-groupc-frontend-spec.md` (plus this plan file); `src/main.rs` is exactly `fn main() {}`; `Cargo.toml` version is `0.2.0`.
3. STOP and ask the caller if any condition fails.
4. NO commit in this step.

### Step C1 — Commit the Group C workflow plan documents

1. Stage ONLY the plan documents: `git add .kilo/plans/20261001-phase1b-terminal-game-groupc-frontend-spec.md .kilo/plans/20261001-phase1b-terminal-game-groupc.md`.
2. Commit: `git commit -m "docs: add group C screens spec and implementation plan"`.
3. Verify `git status` shows a clean tree before continuing.

### Step C2 — Lifecycle accessor (`src/terminal/lifecycle.rs`)

1. Edit `src/terminal/lifecycle.rs`: insert the `output` method exactly per §3.2 (after `disable`, before `write_restore_commands`). NO other edits anywhere in the file.
2. Self-check BEFORE staging: method signature exactly `pub fn output(&mut self) -> &mut W`; body exactly `&mut self.output`; `///` doc comment; `enable`/`disable`/`Drop`/private helpers untouched (`git diff` shows ONLY the insertion).
3. Static rule check: 3 public members now (`enable`, `disable`, `output`); file ≤ 70 lines total.
4. Stage + commit: `git add src/terminal/lifecycle.rs` then `git commit -m "feat: add terminal handle output accessor"`.

### Step C3 — `main.rs` wiring (TODO Tasks 5, 6)

1. Rewrite `src/main.rs` implementing §2.2 exactly, with the code shape of §3.1.
2. Self-check against SPEC §§3–10 BEFORE staging:
   - Wiring order is EXACTLY: setup → enable → start screen → any-key wait → `start_playing` → Renderer → `run_playing_loop` → game-over screen with `state.score()` → any-key wait → `Ok(())` (SPEC §8 list 1–10, one-to-one with the statements).
   - Start screen writes ONLY `Press any key to start` + CRLF after `Clear(ClearType::All)` + `MoveTo(0, 0)`; flush.
   - Game-over screen writes ONLY the five CRLF lines (title / blank / `Score: <score>` / blank / exit prompt) after `Clear(ClearType::All)` + `MoveTo(0, 0)`; flush.
   - `start_playing()` called exactly ONCE in the file, after the first key wait, before the loop.
   - Both waits use the same helper; only `KeyEventKind::Press` returns; blocking reads only (NO `event::poll` in `main.rs`).
   - No explicit `disable()` call — Drop handles restoration (D30).
3. Static rule check: imports exactly per §2.2 (nothing extra — no `KeyEvent`, no `Duration`, no `poll`, no `enter_game_over`, no `GameStatus`); file ≤ 80 lines; `main` returns `io::Result<()>`.
4. Stage + commit: `git add src/main.rs` then `git commit -m "feat: wire start and game over screens into main"`.

(DO NOT commit C2 and C3 together — separate review units.)

### Step C4 — Project structure map (structure maintenance workflow)

1. Edit `.agent/project-structure.md` preserving its format and sections:
   - Update the `src/main.rs` line to: `- src/main.rs - binary entry point; wires initial setup, terminal enable, the start screen, the playing loop, the game-over screen, and exit cleanup (screens and key-wait as private in-file helpers)`
   - Update the `src/terminal/lifecycle.rs` line to: `- src/terminal/lifecycle.rs - raw mode, alternate screen and cursor visibility with Drop guard cleanup plus an output accessor`
   - Update the `# Folders in src/` note line for `src/terminal/` stays UNCHANGED (no new files); leave every other line untouched.
2. Static check: `git status` shows only `.agent/project-structure.md` modified.
3. Stage + commit: `git add .agent/project-structure.md` then `git commit -m "docs: add main wiring to project structure"`.

### Step C5 — README structure bullets (analysis-only; full README rewrite happens at 4.4)

1. Edit `README.md` "Project Structure" bullet list ONLY (two bullets):
   - Replace the `src/main.rs` bullet with: `- \`src/main.rs\`: binary entry point; wires initial setup, terminal enable, the start screen, the playing loop, the game-over screen, and exit cleanup.`
   - Replace the terminal-layer bullet's trailing clause so it reads: `- \`src/terminal.rs\`, \`src/terminal/renderer.rs\`, \`src/terminal/input.rs\`, \`src/terminal/game_loop.rs\`, \`src/terminal/lifecycle.rs\`: terminal layer primitives (board rendering, arrow-key input, timed playing loop, terminal lifecycle).`
2. Nothing else in README changes in 4.2 (no section rewrites — 4.4 owns those).
3. Stage + commit: `git add README.md` then `git commit -m "docs: mention main wiring in readme structure"`.

### Step C6 — Static verification (no compiler available; run in order, fix anything failing)

1. `git log --oneline -12` — verify the 5 commits C1–C5 exist, oldest to newest matching §6 order.
2. `git diff --stat C1^ HEAD`— verify ONLY these files changed relative to the Group B end state (`c4661fb`): `src/terminal/lifecycle.rs`, `src/main.rs`, `.agent/project-structure.md`, `README.md` (+ the two Group C plan `.md` files from C1).
3. Frozen-files audit: `git diff c4661fb HEAD -- src/terminal/renderer.rs src/terminal/input.rs src/terminal/game_loop.rs src/terminal.rs src/lib.rs src/game/ tests/ Cargo.toml .gitignore` → EMPTY (no changes).
4. Line-count audit: `main.rs` ≤ 200 (expect ~60), `lifecycle.rs` ≤ 200 (expect ~68).
5. Naming audit: every new item name appears in this plan or the SPEC (`output`, `show_start_screen`, `show_game_over_screen`, `wait_for_any_key_press`, `write_line`, `score_line`, `is_any_key_press`, `START_MESSAGE`, `GAME_OVER_TITLE`, `EXIT_PROMPT`, `SCORE_PREFIX`, `LINE_BREAK`).
6. Literal-text audit: grep in `src/main.rs` for each exact string — `Press any key to start`, `GAME OVER`, `Score: ` (as `SCORE_PREFIX`), `Press any key to exit` — all present as constants; and `\\n` occurrences in the file are all part of `\\r\\n` (no bare `"\n"` D1 violation).
7. Forbidden-content audit (grep across `src/main.rs`):
   - `println!|eprintln!|dbg!|execute!|unsafe` → zero hits.
   - `is_outside_board|collides_with_body|choose_food_position|SCORE_INCREMENT|is_playing|enter_game_over|change_direction|advance_one_step` → zero hits (no duplicated domain logic, SPEC §11).
   - `start_playing` → EXACTLY ONE occurrence.
   - `event::poll|Duration` → zero hits (waits are blocking `read`s only).
   - `poll` in `lifecycle.rs` diff → zero new hits (accessor introduces none).
8. Borrow-order audit (read the final `main` once): `terminal` on top; single first `output()` borrow consumed by `show_start_screen`; `renderer` consumes the second `output()` borrow and is last used by `run_playing_loop`; the game-over screen re-borrows `terminal.output()` after that; no other `terminal.` uses. No `&mut` borrow conflict statically visible.
9. Contract text audit (SPEC §7.2 exact order/blank-line shape): title → CRLF → CRLF → `Score: <n>` → CRLF → CRLF → exit prompt → CRLF; verify the `write_line(output, "")?` pairs sit between title/score and score/prompt (two blank lines total, 5 CRLF-terminated lines rendered).
10. D-decision spot-audit: D26 (loop + `is_any_key_press` structure, no 3-level nesting, NO `if let` inside the loop), D27 (`"\r\n"` const + `write_line`), D28 (`Score: ` formatting), D29 (accessor after `disable`, zero other lifecycle edits), D30 (no explicit `disable()` call in main), D33 (`start_playing` immediately before Renderer creation), D35 (borrow order), D36 (`queue!` only, no `execute!`).
11. SPEC §12 acceptance checklist: walk all ten criteria statically; all must be satisfiable/true from the code + docs (record results in the step summary returned to the caller).
12. Any FAIL: fix with the smallest possible edit, re-run C6 fully, new commit `fix: align group C screens wiring with implementation plan`; never expand scope during fixes.

---

## 5. Mapping to TODO Tasks (traceability)

| Plan artifact | TODO task | TODO requirement(s) covered | SPEC section |
|---|---|---|---|
| `show_start_screen` + `wait_for_any_key_press` (start site) | Task 5 — Implement the Start Screen | displays `Press any key to start`; snake does NOT move while waiting; any key press transitions into the playing state; visuals simple | SPEC §5 (all) |
| `state.start_playing()` call site | Task 5 — State transition | `WaitingToStart → Playing` edge; the ONLY production caller of `GameState::start_playing()` | SPEC §5.3 |
| `show_game_over_screen` + exit wait | Task 6 — Implement Game Over Flow | loop already stops via Group B (status leaves `Playing`); `GAME OVER` + final score + `Press any key to exit`; one key press exits; no restart | SPEC §7 (all) |
| `main.rs` full orchestration + `io::Result` propagation | Tasks 5, 6 + wiring | complete flow setup → start → play → collision → game over → key → exit; error cleanup guarantee | SPEC §8, §10 |
| `TerminalHandle::output()` (SPEC §4) | wiring enabler | screens + Renderer share the handle's stream; no lifecycle behavior change | SPEC §4, §6 |
| Steps C4, C5 | Implementation Constraints | simplicity; no restart/menus/pause/persistence/config; no new module files; nothing duplicated | SPEC §3, §11 |
| All steps | Out of Scope | NO Docker/Windows/packaging/tests work started in this group | global plan step list |

Explicitly NOT covered here (deferred): Task 9 (flow validation) and Task 10 (tests) — Group D.

---

## 6. Commit Summary (expected end state of 4.2)

```text
* docs: mention main wiring in readme structure
* docs: add main wiring to project structure
* feat: wire start and game over screens into main
* feat: add terminal handle output accessor
* docs: add group C screens spec and implementation plan
```

(old to new; C1 stages both the Group C front-end spec and this implementation plan.) No push; no merge; no TODO `[DONE]` marking (4.6 does it).

---

## 7. Documentation Guidance for Sub-step 4.4 (docs-specialist; content frozen here; do not execute in 4.2 beyond C5)

1. **`README.md` `## Terminal UI (Phase 1B)` section**:
   - Update the intro paragraph: the remaining interactive wiring is DONE — the start and game-over screens and the `main` hookup now connect the loop; the section content should reflect that the flow `setup → enable → "Press any key to start" → key → start_playing` → loop → `GAME OVER`/`Score: N`/`Press any key to exit` → key → exit is implemented. Keep the existing tick/headless-`tick` explanation.
   - Add one bullet after the Lifecycle bullet:
     `- **Screens & wiring** (\`src/main.rs\`): after \`TerminalHandle::enable(stdout())\`, \`main\` clears the alternate screen, shows \`Press any key to start\`, blocks for any key press, calls \`start_playing()\`, runs \`run_playing_loop\`, then on game over shows \`GAME OVER\` / \`Score: N\` / \`Press any key to exit\`, waits for one more key press, and exits — with \`TerminalHandle\`'s Drop guard restoring the terminal on both normal and error paths.`
   - Update the Lifecycle bullet to mention the new `output()` accessor (one clause: "plus an `output()` accessor that shares the stream with the renderer and screens").
2. **`README.md` `## About this Project`**: one sentence noting the terminal gameplay flow (start screen → playing loop → game over → exit) is wired in `main` (still Docker-build pending).
3. **`docs/terminal-ui.md`**:
   - **Status**: change to "Implemented now (Phase 1B, Groups A, B, and C): the renderer, arrow-key input mapping, lifecycle guard (incl. the `output()` accessor), the game loop, AND the start / game-over screens with the `main.rs` wiring. Test execution arrives with the Docker build phase (Phase 2)."
   - **File Map**: add row `\`src/main.rs\` | Binary entry: start screen → any-key wait → playing loop → game-over screen → exit; owns the two private screen helpers and the key-wait helper.`
   - **Public API — Lifecycle**: add `- TerminalHandle::output(&mut self) -> &mut W — borrows the wrapped stream; start/game-over screens and the main.rs Renderer write through it.`
   - **Replace** the section "How the Next Group Connects It (planned, not yet implemented)" with "How the Flow Connects (implemented)" describing: `GameState::new(initial_setup())` → `TerminalHandle::enable(stdout())` → start screen (`Clear(All)` + `MoveTo(0, 0)` + one CRLF line) → `wait_for_any_key_press` (blocking `event::read`, only `KeyEventKind::Press` returns) → `state.start_playing()` → `Renderer::new(terminal.output())` → `run_playing_loop` → game-over screen (five CRLF lines, final `state.score()`) → `wait_for_any_key_press` → `Ok(())` → Drop cleanup; error paths cleaned by Drop.
   - **How to Validate Manually Later**: add two checklist bullets — "Start screen shows exactly `Press any key to start`; the snake is motionless until the key press." and "Game-over screen shows `GAME OVER`, a blank line, `Score: N`, a blank line, `Press any key to exit`; one key press exits with the terminal restored."
4. **`.agent/project-structure.md`**: already updated in Step C4 — at 4.4 do NOT re-edit it.
5. **`.agent/project-info/*`**: phase close-out belongs to the final Group D / step 6 close, NOT to 4.4 of this group.

---

## 8. Front-end Spec Acceptance Cross-Reference (for 4.5a reviewer)

| SPEC §12 criterion | Where satisfied |
|---|---|
| `main.rs` no longer empty; §8 orchestration present | §2.2, §3.1 |
| Start screen exactly `Press any key to start`; snake motionless before key | §3.1 (`show_start_screen`; loop not entered until after wait + `start_playing`) |
| First key → `Playing` via `start_playing()` | §3.1 (only `start_playing` call, after wait) |
| Playing loop 120 ms with existing input/render logic (untouched) | C6 audit 3 (game_loop.rs frozen) |
| Game-over screen: `GAME OVER` / blank / `Score: N` / blank / `Press any key to exit` | §3.1 (`show_game_over_screen`, exact five-line order) |
| One key press after game over exits cleanly | §3.1 (second `wait_for_any_key_press`, then `Ok(())`) |
| Terminal restored on normal AND error paths | D30 + frozen Drop guard (C6 audit 3: lifecycle Drop untouched) |
| No new dependencies | C6 audit 3 (Cargo.toml frozen) |
| No gameplay rules duplicated in `main.rs`/screens | §1.5, C6 audit 7 |
| ≤200-line files; ≤50-line fn bodies; single-section conditions; private-by-default; no commented-out code | §2 budgets, §3 rule checks, C6 audit 10 |

---

## 9. Notes for Reviewers (sub-steps 4.3 / 4.5a / 4.5b)

- Review units are commits C2–C3 (one per code change); C4/C5 are docs-only.
- **Approved, bounded deviation to verify (do NOT flag as a plan violation): D26** — `wait_for_any_key_press` does NOT follow the SPEC §9 literal three-block-nested snippet; it is restructured as `loop` + `is_any_key_press(Event) -> bool` with IDENTICAL behavior (blocks on `event::read()`; first `KeyEventKind::Press` key event returns `Ok(())`; releases/repeats/resize/mouse/other events discarded; `io::Error` propagates). Reason: max-depth-2 rule; precedent = Group A's review fix that extracted a named predicate instead of keeping a spec sketch's shape.
- Approved equivalence to verify: `Renderer::new(terminal.output())` creates `Renderer<&mut Stdout>` — the renderer owns a mutable borrow, not the stream; the game-over screen re-borrows after the loop under NLL. If a reviewer wants an owned `Renderer<Stdout>`, that is a plan deviation (rejected: two stdout streams could interleave; SPEC §6 mandates the shared borrow).
- The last GameOver frame IS rendered by the frozen Group B loop before the game-over screen clears it — that clearing is deliberate (SPEC §7.2: no board on the exit screen). Do not "optimize" the Clear away.
- `main` has no unwrap/expect/panic path; all errors propagate with `?`. Default Rust error printing is the agreed error UX (D34) — reviewers must not demand retry loops or error screens.
- If `crossterm = "0.29"` resolves to an API-different version at future build time, treat THAT as a build-time blocker, not a plan deviation (record in the adherence report).
- No compiler exists here: a reviewer must not demand `cargo build`/`cargo check` output; static checks of §10 (C6) are the verification contract.
