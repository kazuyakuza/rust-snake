# Implementation Plan — Phase 1B Group B: Game Loop, Wiring, Rendering Refinement (TODO Tasks 3, 4, 8)

- **Sub-step**: 4.1b of Group B (Tasks 3 + 4 + 8) — produced by the Architector.
- **TODO source**: `.agent/todos/20261001/20261001-todo-3.md` (Pattern C; ONLY tasks 3, 4, 8 + "Implementation Constraints" + "Out of Scope" are in scope for this group).
- **Front-end input (mandatory follow)**: `.kilo/plans/20261001-phase1b-terminal-game-groupb-frontend-spec.md` (cited below as `SPEC §n`; it is the authoritative behavioral contract for this group).
- **Global plan**: `.kilo/plans/20261001-phase1b-terminal-game.md`.
- **Group A baseline**: `.kilo/plans/20261001-phase1b-terminal-game-groupa.md`; the committed primitives are the frozen start state this plan extends.
- **Branch**: `feat/phase1b-terminal-game`. It already exists; the version is already `0.2.0` (Group A finished: commits `b92969a`…`6c3740e`). NO branch creation/switching here. NO version bump here. NO push. NO merge.
- **Implementer level**: JUNIOR, hard-blocked from scope/architecture decisions; follow this plan exactly; STOP and ask the caller if anything is ambiguous or impossible.
- **Verification mode**: NO local Rust toolchain, NO Docker. There is **no compile/test execution** in this group. Verification = static review only (order in §10). Authored tests execute in Phase 2 (Docker). Do not author any test in this group (Group D owns tests).

---

## 0. Scope Guardrails (binding)

### 0.1 In scope (this group, sub-step 4.2 implementation)

| TODO task (20261001-todo-3.md) | Deliverable | File(s) |
|---|---|---|
| Task 3 — Implement the Game Loop | Fixed 120 ms playing loop + headless one-tick function | `src/terminal/game_loop.rs` (NEW) |
| Task 4 — Connect Movement, Input, and Game State | Chronological arrow-direction drain wired to `change_direction`; `advance_one_step()` once per tick | `src/terminal/input.rs` (extend), `src/terminal/game_loop.rs` |
| Task 8 — Refine Terminal Rendering Behavior | Loop-side render discipline (one render/tick, final GameOver frame, no stray output, no extra flushes) | `src/terminal/game_loop.rs` (+ docs statements) |
| Module wiring | Library visibility of the new module | `src/terminal.rs` (extend) |
| Structure map | Reflect the new file | `.agent/project-structure.md` |
| README structure bullet | Mention `game_loop.rs` | `README.md` |

### 0.2 Out of scope (do NOT touch, do NOT create)

- `src/main.rs` — **stays `fn main() {}` in Group B**. Decision (encoded, from SPEC §4.4 + §16): the playing loop is entered only when `state.status() == GameStatus::Playing`; the start-screen wait + `start_playing()` are owned by Group C. A loop whose only specified entry is `Playing` has no safe, complete call path until the Group C screens exist; half-wiring `main.rs` now (starting the loop without a `start_playing()` call) would enter through an unspecified status and contradict SPEC §4.4. Therefore `main.rs` is NOT modified in this group; full flow wiring (start screen → loop → game-over flow) is Group C.
- Start-screen text / wait loop / `start_playing()` call site (Group C, TODO Task 5).
- Game-over screen text, final-score presentation, exit-on-key wait (Group C, TODO Task 6).
- `src/game/**` — the domain API is frozen; not one line changes (SPEC §16).
- `src/terminal/renderer.rs`, `src/terminal/lifecycle.rs` — frozen; no behavior change (SPEC §8.2: "No changes to `renderer.rs` are required").
- Test authoring (Group D, TODO Task 10); flow validation (Group D, TODO Task 9).
- `Dockerfile`, `compose.yaml`, `Cargo.lock`, `dist/` (Phase 2). No new dependencies, crates, or feature flags.
- Restart, pause, colors, difficulty, config — forbidden by the TODO "Implementation Constraints".

### 0.3 Files NOT to modify

`src/main.rs`, `src/lib.rs`, `src/game.rs`, `src/game/**` (all 8 files), `src/terminal/renderer.rs`, `src/terminal/lifecycle.rs`, `tests/**` (all 6 files), `.agent/project-info/*` (close-out happens later), `.gitignore`, `Cargo.toml`.

`docs/terminal-ui.md` and `README.md` are touched ONLY where this plan explicitly says so: `README.md` gets the single structure-bullet edit in Step B6 (sub-step 4.2); `docs/terminal-ui.md` and broader README edits are the CONTENT GUIDANCE in §7 and are executed only by docs-specialist at sub-step 4.4.

---

## 1. Pre-Analysis — Decisions Encoded (no implementer judgment required)

### 1.1 Current state (verified in-tree)

- Group A primitives are committed and FROZEN:
  - `src/terminal/renderer.rs` (108 lines): `Renderer<W: Write>` private `output` field; `pub fn new(output: W) -> Renderer<W>`; `pub fn render(&mut self, state: &GameState) -> io::Result<()>` (MoveTo(0,0) → full-frame overwrite → single `flush()`).
  - `src/terminal/input.rs` (60 lines): `pub fn map_key_event_to_direction(&KeyEvent) -> Option<Direction>` (press-filter inside, `_ => None` falls through), private `is_key_press`, `pub fn drain_arrow_event() -> io::Result<Option<KeyEvent>>` (last-arrow-wins), private `pressed_arrow_key(Event) -> Option<KeyEvent>` / `is_arrow_key_press(&KeyEvent) -> bool` / `is_arrow_key(&KeyEvent) -> bool`.
  - `src/terminal/lifecycle.rs` (60 lines): `TerminalHandle<W: Write>` with `pub fn enable(W)`, `pub fn disable()`, `Drop` guard.
  - `src/terminal.rs`: declares `pub mod input; pub mod lifecycle; pub mod renderer;` (alphabetical).
  - `src/lib.rs`: `pub mod game; pub mod terminal;` — no change needed (the new module hangs off `terminal`).
- Domain API verified in-tree (`src/game/state.rs`, `src/game/setup.rs`): `GameState::{new(GameStateSetup), snake(), food(), score(), status(), current_direction(), change_direction(Direction) -> bool, advance_one_step(), start_playing(), enter_game_over()}`, `GameStatus::{WaitingToStart, Playing, GameOver}` (derives `PartialEq`, `Copy`), `WIDTH = 40`, `HEIGHT = 25`. The construction entry point for later consumers is `crate::game::setup::{initial_setup(), GameStateSetup}` — Group B does NOT construct a `GameState` anywhere (Group C/D callers do).
- `src/main.rs` is `fn main() {}` (1 line) — unchanged here (see §0.2).
- No compile/test execution possible: verification is static only.

### 1.2 Line-budget check for the `input.rs` extension (rule `max-lines-per-file` ≤200)

- `input.rs` currently 60 lines. The extension adds ~18 lines → ~78 total. WELL within the 200-line budget. **Decision: extend `input.rs` in place; do NOT create a new input file** (SPEC §6 mandates the helper lives in `input.rs`; no second module for it).

### 1.3 Behavioral decisions (D13+ continue Group A's D-numbering; each answer is binding)

| # | Question | Decision |
|---|---|---|
| D13 | Drain semantics for the loop | NEW `drain_arrow_directions() -> io::Result<Vec<Direction>>` returns ALL buffered arrow presses in chronological (read) order; existing `drain_arrow_event()` stays UNTOUCHED (SPEC §4.2, §6). |
| D14 | Where the `while` lives | `run_playing_loop` owns the timing/status `while`; `tick` is pure/headless (no sleep, no real-input read) and returns the NEW `GameStatus` (SPEC §5). |
| D15 | In-loop GameOver handling | `tick` renders the GameOver frame (losing snake position visible), then `run_playing_loop`'s `while` re-check fails and the function returns `Ok(())`. No explicit `break`/`return` inside the loop body beyond falling out of the `while` (SPEC §4.5, §4.6). Note: `sleep_remaining` is skipped on the final (GameOver) tick only because the `while` exits after `tick` returns — accept that the last tick does NOT sleep (SPEC's own §4.6 sketch has the same shape). |
| D16 | `WaitingToStart` entry | The loop is intended ONLY when `Playing` (SPEC §4.4). No pre-check beyond the `while` condition (documented precondition). If invoked while `WaitingToStart`, `advance_one_step()` is a domain no-op and the loop would render + sleep indefinitely — this is unreachable from the Group C contract and is deliberately NOT special-cased in Group B (SPEC §4.4: "behavior is unspecified for Group B"). |
| D17 | Render/flush discipline (Task 8) | Loop-side: NO `println!`/`eprintln!`/extra flushes anywhere in `game_loop.rs`; `Renderer::render` is the only output flush point; render is called exactly once per tick, AFTER `advance_one_step()` (SPEC §8.2 items 1–4). |
| D18 | `read_arrow_direction` composition | Follows the SPEC §6 literal snippet exactly: `event::read().ok()?` → match `Event::Key` (non-key events → `None`) → `map_key_event_to_direction(&key_event)`. The existing private `pressed_arrow_key` stays in use by `drain_arrow_event` ONLY; the new helper does not call it (no dead code: both are still referenced). "Arrow key" remains single-sourced through `map_key_event_to_direction`, which encodes press-filter + arrow mapping. Approved wording note for 4.5b; zero behavior difference. |
| D18b | Push condition in `drain_arrow_directions` | Push only when `read_arrow_direction()` returns `Some`, exactly as the SPEC §6 snippet: `if let Some(direction) = read_arrow_direction() { directions.push(direction); }`. Non-arrow and release/repeat events are consumed and dropped; the drain NEVER blocks (poll `Duration::ZERO`). |
| D19 | Tick work exceeds 120 ms | `sleep_remaining` no-ops when `elapsed >= TICK_DURATION` (guard written as `if elapsed < TICK_DURATION { ... }`); the loop just proceeds to the next tick; NO multi-tick catch-up (SPEC §4.1). |
| D20 | Constant visibility | `TICK_DURATION` is a private module-level `const` in `game_loop.rs` (prefer-private rule; observers verify timing behaviorally, not by reading the const). |
| D21 | `tick` param-count exception | SPEC §5 freezes `tick(state, directions, renderer)` — 3 params (exceeds the ≤2 rule). The rules allow documented exceptions; SPEC §10 + the Group D test hooks are written against this exact signature, and the params form one coherent unit (domain state + tick input + IO adapter). **The SPEC signature is authoritative**; do NOT wrap the params in a struct. Record as a bounded, documented exception for review. |
| D22 | `apply_directions` / `sleep_remaining` / `read_arrow_direction` visibility | All private (module-internal helpers only). |
| D23 | Sleep mechanics | `std::thread::sleep(TICK_DURATION - elapsed)` is the only wait; no spin, no `event::poll(remaining)` long waits (poll remains `Duration::ZERO` inside the drains). |
| D24 | `GameStatus` import in `game_loop.rs` | Needed for the `while` condition and `tick`'s return value. "Renderer never branches on status" (D9/SPEC §4.7) is untouched: the LOOP branches on status, the renderer does not. |
| D25 | Doc-comment style | Group A precedent: `//!` module docs + `///` on public items only; no comments on private items; no commented-out code. |

### 1.4 API surface verification (crossterm 0.29 + std, no new dependency)

- `crossterm::event::poll(timeout: Duration) -> io::Result<bool>` — **by value** (Group A confirmed docs.rs 0.29.0; never `&Duration`). `Ok(true)` guarantees the next `event::read()` does not block.
- `crossterm::event::{Event, KeyEvent}` — already imported in `input.rs`; no new crossterm import needed there.
- Std additions used by `game_loop.rs`: `std::io::{self, Write}`, `std::thread`, `std::time::{Duration, Instant}`.
- **No new dependency added; `Cargo.toml` untouched.**

### 1.5 Domain API contract (verified in-tree, DO NOT redesign)

- `GameState::status() -> GameStatus` (Copy) drives the loop condition and `tick`'s return.
- `GameState::change_direction(Direction) -> bool` — reversal rejection is domain-owned; the terminal layer **MUST NOT duplicate it** and ignores the returned `bool` (SPEC §4.3 step 3, §7.2).
- `GameState::advance_one_step()` — exactly once per tick; resolves movement, boundary/self collision, food consumption, scoring, growth, respawn, and the `Playing` gate (SPEC §7.1).
- `GameState::start_playing()` — **NOT called anywhere in Group B** (Group C owns the start-screen call site, SPEC §7.1).
- Read accessors (`snake()`, `food()`, `score()`, `status()`) are consumed only via `Renderer::render`.
- The terminal layer mutates `GameState` ONLY via `change_direction` and `advance_one_step` (SPEC §9); terminal modules may depend only on `game::state` and `game::direction` (+ `game::position` transitively in the existing renderer).

---

## 2. Target State — File-by-File Structure

Line budgets (rule ≤200 lines/file, ideal ≤125 code): `input.rs` after edit ≤ 80 total (~70 code), `game_loop.rs` ≤ 70 total (~55 code), `terminal.rs` ≤ 12. If an implementation draft exceeds a budget, implementer STOPs and reports — do not split files without approval.

### 2.1 `src/terminal.rs` (modify — module declaration)

Final exact content (update the module doc lines; declare `game_loop` first, keeping alphabetical order):

```rust
//! Terminal UI layer: rendering, arrow-key input, timed playing loop, and terminal
//! lifecycle. Gameplay rules remain in `game`; this layer only renders state, maps
//! keys, ticks the loop, and toggles raw mode / alternate screen / cursor visibility.

pub mod game_loop;
pub mod input;
pub mod lifecycle;
pub mod renderer;
```

### 2.2 `src/terminal/input.rs` (extend — TODO Task 4 helper; existing content stays unchanged)

| Item | Specification |
|---|---|
| Imports | UNCHANGED — the existing `use std::io::{self};`, `use std::time::Duration;`, `use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};`, `use crate::game::direction::Direction;` already cover everything the new functions need (`Vec` is in prelude; `Option` too). |
| New public fn | `pub fn drain_arrow_directions() -> io::Result<Vec<Direction>>` — drains ALL pending events with `while event::poll(Duration::ZERO)?`, pushes every direction produced by `read_arrow_direction()`, returns them in chronological order; never blocks (SPEC §6). |
| New private fn | `fn read_arrow_direction() -> Option<Direction>` — `event::read().ok()?` → match `Event::Key(key_event)` else `None` → `map_key_event_to_direction(&key_event)` (D18; see snippet). |
| Where | Appended at the end of the file, after `is_arrow_key`; doc-comment `///` on the public fn, none on the private one. |
| Frozen Group A content | `map_key_event_to_direction`, `is_key_press`, `drain_arrow_event`, `pressed_arrow_key`, `is_arrow_key_press`, `is_arrow_key` — zero modifications. |

### 2.3 `src/terminal/game_loop.rs` (NEW — TODO Tasks 3, 4, 8)

| Item | Specification |
|---|---|
| Imports | `use std::io::{self, Write};` · `use std::thread;` · `use std::time::{Duration, Instant};` · `use crate::game::direction::Direction;` · `use crate::game::state::{GameState, GameStatus};` · `use crate::terminal::input::drain_arrow_directions;` · `use crate::terminal::renderer::Renderer;` |
| Constant (private) | `const TICK_DURATION: Duration = Duration::from_millis(120);` |
| Public loop fn | `pub fn run_playing_loop<W: Write>(state: &mut GameState, renderer: &mut Renderer<W>) -> io::Result<()>` — `while state.status() == GameStatus::Playing { tick_start → drain → tick(...) → sleep_remaining }`; returns `Ok(())` when the status is no longer `Playing` (i.e., `GameOver` reached; SPEC §4.6). |
| Public headless tick fn | `pub fn tick<W: Write>(state: &mut GameState, directions: &[Direction], renderer: &mut Renderer<W>) -> io::Result<GameStatus>` — `apply_directions` → `state.advance_one_step()` → `renderer.render(state)?` → `Ok(state.status())`. No sleep, no terminal read (SPEC §5). |
| Private helper 1 | `fn apply_directions(state: &mut GameState, directions: &[Direction])` — `for &direction in directions { state.change_direction(direction); }` (return `bool` ignored, D-decision §1.5). |
| Private helper 2 | `fn sleep_remaining(tick_start: Instant)` — `let elapsed = tick_start.elapsed();` then `if elapsed < TICK_DURATION { thread::sleep(TICK_DURATION - elapsed); }` (SPEC §4.1; single-section `if`). |
| No other items | No struct, no extra public API, no `GameStatus::GameOver` literal match, no `start_playing`, no render-adjacent output. |
| Doc comments | `//!` module docs (loop + timing contract) + `///` on the two public fns (precondition: status `Playing`; final-frame-on-GameOver note), per D25. |

---

## 3. Code Sketches for Tricky Parts (encodes all decisions; local variable names may vary only if listed names stay recognizable)

### 3.1 `input.rs` — the two appended functions (Task 4)

```rust
/// Drain all currently available input events without waiting, and return the
/// arrow-key directions seen, in chronological order. Never blocks.
pub fn drain_arrow_directions() -> io::Result<Vec<Direction>> {
    let mut directions = Vec::new();
    while event::poll(Duration::ZERO)? {
        if let Some(direction) = read_arrow_direction() {
            directions.push(direction);
        }
    }
    Ok(directions)
}

fn read_arrow_direction() -> Option<Direction> {
    let event = event::read().ok()?;
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    map_key_event_to_direction(&key_event)
}
```

- D18 note: the new `read_arrow_direction` takes the SPEC §6 literal form (match `Event::Key`, then `map_key_event_to_direction`); the existing private `pressed_arrow_key` stays untouched and continues to serve `drain_arrow_event` only. "What counts as an arrow press" remains single-sourced inside `map_key_event_to_direction` (press-filter + arrow code match). No dead code and no behavior difference.
- Chronology: `event::poll` + `event::read` return events in arrival order, so the `Vec` is chronological by construction.
- Blocking: zero. `poll(Duration::ZERO)` returns `Ok(false)` immediately when the queue is empty.

### 3.2 `game_loop.rs` — full module shape (Tasks 3, 4, 8)

```rust
//! Fixed 120 ms playing loop: drain arrow directions, apply them via the
//! domain's `change_direction`, advance one step, render, and sleep the
//! remainder of the tick. Runs only while the status is `Playing`; it renders
//! the final frame when the status becomes `GameOver` and then returns.

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use crate::game::direction::Direction;
use crate::game::state::{GameState, GameStatus};
use crate::terminal::input::drain_arrow_directions;
use crate::terminal::renderer::Renderer;

const TICK_DURATION: Duration = Duration::from_millis(120);

/// Run the fixed 120 ms playing loop until the game reaches GameOver.
///
/// Precondition: `state.status()` is `Playing`. Each iteration polls input,
/// applies direction changes, advances the domain state, renders it once, and
/// sleeps the remainder of the tick. The loop renders the final frame (the
/// losing position) and returns when the status is no longer `Playing`.
pub fn run_playing_loop<W: Write>(
    state: &mut GameState,
    renderer: &mut Renderer<W>,
) -> io::Result<()> {
    while state.status() == GameStatus::Playing {
        let tick_start = Instant::now();
        let directions = drain_arrow_directions()?;
        tick(state, &directions, renderer)?;
        sleep_remaining(tick_start);
    }
    Ok(())
}

/// Execute exactly one playing tick with the supplied input directions.
///
/// Applies each direction in order, advances the domain once, renders the
/// resulting state, and returns the new status. Headless: never sleeps and
/// never reads the real terminal, so tests can drive `Vec<u8>` renderers.
pub fn tick<W: Write>(
    state: &mut GameState,
    directions: &[Direction],
    renderer: &mut Renderer<W>,
) -> io::Result<GameStatus> {
    apply_directions(state, directions);
    state.advance_one_step();
    renderer.render(state)?;
    Ok(state.status())
}

fn apply_directions(state: &mut GameState, directions: &[Direction]) {
    for &direction in directions {
        state.change_direction(direction);
    }
}

fn sleep_remaining(tick_start: Instant) {
    let elapsed = tick_start.elapsed();
    if elapsed < TICK_DURATION {
        thread::sleep(TICK_DURATION - elapsed);
    }
}
```

Rule checks (must hold in the final code):

- `run_playing_loop` body ≤ 10 lines (`while` + 4 statements); nesting depth = 1 (the `while`).
- `tick` body ≤ 4 lines, no branching; nesting depth = 1.
- `sleep_remaining`: single-section `if` (the `elapsed < TICK_DURATION` guard hides the subtraction edge from the call site — no `TICK_DURATION - elapsed` underflow path exists because it is inside the guard).
- `apply_directions`: `bool` from `change_direction` deliberately discarded (no `let _ =` — plain call, SPEC §4.3 step 3).
- Every `if` / `while` condition is a single section (no `&&` / `||` anywhere in the module).
- ≤ 2 non-self params per fn except `tick` (three) and `run_playing_loop` (two — fine) per documented exception D21.
- In `run_playing_loop`, D14/D15 shape: `tick` returns `GameStatus`; the loop re-checks via the `while` condition. The returned status of `tick` inside the loop is deliberately ignored (plain `tick(...)?;` statement) — the status check happens at the top of the next iteration. Group D may still use `tick`'s return value headlessly.

---

## 4. Step-by-Step Implementation (execute in order; every step ends in a verification)

Legend: numbered steps are atomic and verifiable. NO build/test commands exist (no toolchain). Sub-step labels map to Critical Workflow 4.2 execution for this group.

### Step B0 — Preconditions check (no file changes)

1. Run `git status` and `git branch --show-current`.
2. Verify: branch is `feat/phase1b-terminal-game`; `Cargo.toml` version is `0.2.0`; working tree has ONLY the untracked file `.kilo/plans/20261001-phase1b-terminal-game-groupb-frontend-spec.md` (plus this plan file).
3. STOP and ask the caller if any condition fails.
4. NO commit in this step.

### Step B1 — Commit the Group B workflow plan documents

1. Stage ONLY the plan documents: `git add .kilo/plans/20261001-phase1b-terminal-game-groupb-frontend-spec.md .kilo/plans/20261001-phase1b-terminal-game-groupb.md`.
2. Commit: `git commit -m "docs: add group B game loop spec and implementation plan"`.
3. Verify `git status` shows a clean tree before continuing.

### Step B2 — Module wiring (`src/terminal.rs`)

1. Edit `src/terminal.rs` to the exact content of §2.1 (adds `pub mod game_loop;`, updates the `//!` module docs).
2. Static check: `git diff --stat` shows exactly `src/terminal.rs` modified; `src/terminal/game_loop.rs` does not exist yet (a full compile would fail until B3 — acceptable; do NOT create placeholder files, do NOT run any build command).
3. Stage + commit: `git add src/terminal.rs` then `git commit -m "feat: declare game loop module in terminal root"`.

### Step B3 — Input extension (TODO Task 4 helper)

1. Edit `src/terminal/input.rs`: append the two functions of §3.1 verbatim at the file end (after `is_arrow_key`). NO other edits anywhere in the file (Group A functions frozen).
2. Self-check BEFORE staging: `drain_arrow_directions` uses `while event::poll(Duration::ZERO)?` (by-value Duration); pushes only `Some` directions; returns `Ok(directions)`; never blocks; `read_arrow_direction` returns `None` for non-key events and keys that are not arrows; `Direction` import already present; NO new imports required.
3. Static rule check: only ONE new `pub` fn (`drain_arrow_directions`); `read_arrow_direction` private; helpers ≤ 2 params; nesting depth of `drain_arrow_directions` = 2 (`while` + `if let`) — max allowed; file total ≤ 80 lines.
4. Stage + commit: `git add src/terminal/input.rs` then `git commit -m "feat: drain all buffered arrow directions chronologically"`.

### Step B4 — Game loop module (TODO Tasks 3, 4, 8)

1. Create `src/terminal/game_loop.rs` implementing §2.3 exactly, with the code shape of §3.2.
2. Self-check against SPEC §4/§5/§7/§8 BEFORE staging:
   - `TICK_DURATION` = `Duration::from_millis(120)`, private.
   - Loop condition exactly `state.status() == GameStatus::Playing` (single section).
   - Per-tick order: `Instant::now()` → `drain_arrow_directions()?` → `tick(...)?` → `sleep_remaining(tick_start)`.
   - `tick` order: `apply_directions` → `advance_one_step()` → `render` → return `state.status()`.
   - NO `println!`/`eprintln!`/`dbg!`, NO extra `flush`, NO `break`, NO `GameStatus::GameOver` literal, NO `start_playing`.
   - Domain interactions ONLY: `change_direction` (bool ignored), `advance_one_step()`, `status()`; NO transition/collision/food reads.
3. Static rule check: imports exactly per §2.3 table (nothing extra — e.g., no `Setup`, no `TerminalHandle`); two `pub` fns only; file ≤ 70 lines; `//!` + `///` comments match D25.
4. Stage + commit: `git add src/terminal/game_loop.rs` then `git commit -m "feat: run fixed 120 ms playing loop with headless tick"`.

### Step B5 — Project structure map (structure maintenance workflow)

1. Edit `.agent/project-structure.md` preserving its format and sections:
   - In `# Folders in src/`, update the `src/terminal/` line to: `- src/terminal/ - terminal UI module tree (game_loop, input, lifecycle, renderer)`
   - In `# Rust project files`, between the `src/terminal.rs` line and the `src/terminal/renderer.rs` line, insert: `- src/terminal/game_loop.rs - fixed 120 ms playing loop: drains arrow directions, advances the domain, renders, and sleeps the tick remainder`
   - Update the `src/terminal/input.rs` line to: `- src/terminal/input.rs - arrow key press to game Direction mapping plus non blocking drains of the last event and of all buffered directions`
   - Leave every other line untouched.
2. Static check: `git status` shows only `.agent/project-structure.md` modified.
3. Stage + commit: `git add .agent/project-structure.md` then `git commit -m "docs: add game loop module to project structure"`.

### Step B6 — README structure bullet (analysis-only; full README rewrite happens at 4.4)

1. Edit `README.md` "Project Structure" bullet list ONLY: update the terminal bullet to read:
   `- \`src/terminal.rs\`, \`src/terminal/renderer.rs\`, \`src/terminal/input.rs\`, \`src/terminal/game_loop.rs\`, \`src/terminal/lifecycle.rs\`: terminal layer primitives (board rendering, arrow-key input, timed playing loop, terminal lifecycle) — start/game-over screens and \`main\` wiring arrive in the next group.`
2. Nothing else in README changes in 4.2 (no section rewrites — 4.4 owns those).
3. Stage + commit: `git add README.md` then `git commit -m "docs: mention game loop in readme structure"`.

### Step B7 — Static verification (no compiler available; run in order, fix anything failing)

1. `git log --oneline -8` — verify the 6 commits B1–B6 exist, oldest to newest matching §6 order.
2. `git diff --stat HEAD~6 HEAD` — verify ONLY these files changed: `.kilo/plans/20261001-phase1b-terminal-game-groupb-frontend-spec.md`, `.kilo/plans/20261001-phase1b-terminal-game-groupb.md`, `src/terminal.rs`, `src/terminal/input.rs`, `src/terminal/game_loop.rs`, `.agent/project-structure.md`, `README.md`.
3. Line-count audit: `input.rs` ≤ 200 (expect ~78), `game_loop.rs` ≤ 200 (expect ~65), `terminal.rs` ≤ 12.
4. Naming audit: every new item name appears in this plan or the SPEC (`drain_arrow_directions`, `read_arrow_direction`, `run_playing_loop`, `tick`, `apply_directions`, `sleep_remaining`, `TICK_DURATION`).
5. Forbidden-content audit (grep across `src/terminal/`):
   - `println!|eprintln!|dbg!` → zero hits (Task 8 no-stray-output).
   - `flush` in `game_loop.rs` → zero hits (only `renderer.rs` flushes).
   - `start_playing|enter_game_over|opposite\(\)|is_outside_board|collides_with_body|choose_food_position|SCORE_INCREMENT|is_inside_board` in `game_loop.rs` → zero hits (no duplicated/reachable domain-internal rules; SPEC §7.2, §9).
   - `unsafe` → zero hits; `Clear` → zero hits.
   - `change_direction|advance_one_step|status\(\)` appear ONLY in `game_loop.rs` (not in `input.rs`/`renderer.rs`/`lifecycle.rs`).
6. Frozen-files audit: `git diff HEAD~6 HEAD -- src/terminal/renderer.rs src/terminal/lifecycle.rs src/main.rs src/lib.rs src/game/ tests/ Cargo.toml` → EMPTY (no changes).
7. Module-graph audit: `src/terminal.rs` declares `game_loop` before `input` (alphabetical); `game_loop.rs` imports only paths from §2.3; no file imports `crate::terminal::game_loop` inside `src/` yet (main.rs untouched — Group C wires it).
8. Rule-compliance matrix (record PASS/FAIL per file): max 200 lines/file · ≤125 ideal code lines · fn bodies ≤50 lines · ≤2 non-self params (except documented `tick`) · ≤2 nesting depth · single-section boolean conditions · private members by default · self-documenting names · no commented-out code · minimal comments.
9. D-decision spot-audit: D13 (all directions kept, not last-wins), D15 (no explicit break), D18b (`if let Some` push shape), D19 (guard `elapsed < TICK_DURATION`), D20 (`const` private), D23/poll-by-value (`poll(Duration::ZERO)` with no borrow).
10. Any FAIL: fix with the smallest possible edit, re-run B7 fully, new commit `fix: align group B loop with implementation plan`; never expand scope during fixes.

---

## 5. Mapping to TODO Tasks (traceability)

| Plan artifact | TODO task | TODO requirement(s) covered | SPEC section |
|---|---|---|---|
| `src/terminal/game_loop.rs` `run_playing_loop` + `sleep_remaining` + `TICK_DURATION` | Task 3 — Implement the Game Loop | fixed ~120 ms interval; loop processes input → updates state → detects collision/food (via `advance_one_step`) → renders → waits remainder; input poll never blocks movement (`Duration::ZERO`); timing/polling = implementation detail | SPEC §4 (all) |
| `src/terminal/input.rs` `drain_arrow_directions` / `read_arrow_direction`; `game_loop.rs` `apply_directions` | Task 4 — Connect Movement, Input, and Game State | arrow keys → direction via existing mapping; direction applied through `change_direction` (reversal NOT duplicated); movement/collision/scoring/growth/loss handled ONLY by domain `advance_one_step`; terminal layer has no gameplay rules | SPEC §6, §7 |
| `game_loop.rs` render-once-per-tick + final GameOver frame + no stray output/flush; docs statements (§7 below) | Task 8 — Refine Terminal Rendering Behavior | repeated updates stay clean/readable; no scroll (full-frame renderer already does MoveTo+overwrite); board visually stable; no unnecessary output | SPEC §8 |
| Steps B2, B5, B6 | Implementation Constraints | keep terminal layer simple; no engine/framework added; input/rendering/timing only; domain stays independent | SPEC §12–§14 |
| All steps | Out of Scope | NO Docker/Windows/packaging work started in this group | global plan step list |

Explicitly NOT covered here (deferred): Task 5 (start screen), Task 6 (game-over flow), Task 9 (flow validation), Task 10 (tests) — Groups C/D.

---

## 6. Commit Summary (expected end state of 4.2)

```text
* docs: mention game loop in readme structure
* docs: add game loop module to project structure
* feat: run fixed 120 ms playing loop with headless tick
* feat: drain all buffered arrow directions chronologically
* feat: declare game loop module in terminal root
* docs: add group B game loop spec and implementation plan
```

(old to new; B1 stages both the Group B front-end spec and this implementation plan.) No push; no merge; main.rs stays `fn main() {}`.

---

## 7. Documentation Guidance for Sub-step 4.4 (docs-specialist; content frozen here; do not execute in 4.2 beyond B6)

1. **`README.md` `## Terminal UI (Phase 1B)` section**:
   - Update the intro sentence: primitives AND the playing loop are implemented; start/game-over screens and the `main` hookup remain the next group.
   - Add a **Game loop** bullet between Input and Lifecycle: `- **Game loop** (\`src/terminal/game_loop.rs\`): \`run_playing_loop\` ticks at a fixed 120 ms while the status is Playing — each tick drains buffered arrow presses (chronological), applies them through the domain's change_direction, calls advance_one_step once, renders exactly once, and sleeps the remainder of the tick. Entering with a status other than Playing is outside its contract (the start screen arrives with the next group). On GameOver the final frame renders in the losing position, then the loop returns.`
   - Update the Input bullet: `drain_arrow_directions` (all buffered arrow presses, chronological) exists alongside `drain_arrow_event` (last arrow press) — both non blocking (`event::poll(Duration::ZERO)`).
2. **`docs/terminal-ui.md`**:
   - **Status** section: replace the "Not yet present" wording to reflect — implemented through Group B: game loop (`run_playing_loop`, `tick`), chronological directions drain; still not present: start/game-over screens, `main.rs` wiring (Groups C), tests execution (Docker phase, Phase 2).
   - **File Map** table: add row `src/terminal/game_loop.rs | Fixed 120 ms playing loop and headless one-tick function.`
   - **Public API**: add Game loop block after the Input block:
     - `run_playing_loop(&mut GameState, &mut Renderer<W>) -> io::Result<()>` — precondition `status() == Playing`; per tick: drain → apply → advance → render → sleep remainder; returns after rendering the final frame once status leaves `Playing`.
     - `tick(&mut GameState, &[Direction], &mut Renderer<W>) -> io::Result<GameStatus>` — one headless transition, no sleep/terminal read; Group-D test hook (drives a `Vec<u8>` renderer).
     - `drain_arrow_directions() -> io::Result<Vec<Direction>>` — all buffered arrow presses chronologically, never blocks.
   - **"How the Next Group Wires It"** section: keep the plan, but adjust step 2 wording to the implemented reality — each tick: `drain_arrow_directions()` → `run_playing_loop` applies `change_direction` → `advance_one_step()` → `render` (title the section "How the Next Group Connects It (planned, not yet implemented)" if clearer; content otherwise unchanged).
3. **`.agent/project-structure.md`**: already updated in Step B5 — at 4.4 do NOT re-edit it.
4. **`.agent/project-info/*`**: phase close-out belongs to the final Group D / step 6 close, NOT to 4.4 of this group.

---

## 8. Front-end Spec Acceptance Cross-Reference (for 4.5a reviewer)

| SPEC §15 criterion | Where satisfied |
|---|---|
| `game_loop.rs` declares `TICK_DURATION = 120 ms`, `run_playing_loop`, `tick` | §2.3, §3.2 |
| Loop iterates while `status == Playing`, polls, applies, advances, renders, sleeps remainder | §3.2 (`run_playing_loop`) |
| `tick` performs one headless transition and returns the new `GameStatus` | §3.2 (`tick`) |
| `drain_arrow_directions` returns all buffered arrow directions, chronological, non-blocking | §2.2, §3.1 |
| Direction changes only via `change_direction`; no reversal logic in terminal layer | §1.5, §3.2 (`apply_directions`), B7 audit 5 |
| Movement/collision/scoring/growth/respawn only via `advance_one_step` | §1.5, §3.2 (`tick`) |
| Render exactly once per tick + final frame before exiting on GameOver | §3.2 (render inside `tick`; `while` exits after final render), D15 |
| No extra terminal output (no `println!`/`eprintln!`/flushes/scrolling) | B7 audit 5 |
| Group A frozen contracts intact; only `input.rs` extended with `drain_arrow_directions` | §0.3, B7 audit 6 |
| All new files ≤ 200 lines | §2 budgets, B7 audit 3 |

---

## 9. Notes for Reviewers (sub-steps 4.3 / 4.5a / 4.5b)

- Review units are commits B2–B4 (one per code change); B5/B6 are docs-only.
- Approved, bounded deviations to verify (do not flag as plan violations): (a) `read_arrow_direction` composition follows the SPEC §6 literal match-form (D18), `pressed_arrow_key` untouched; (b) `tick` has 3 params — SPEC-frozen signature wins over the ≤2-params rule (D21 documented exception); (c) the final GameOver tick does NOT sleep before loop exit (SPEC §4.6 sketch has the identical shape; D15).
- `run_playing_loop` ignores `tick`'s returned status in-loop; the `while` re-check is the single status authority (D14/D15). If a reviewer prefers `if status == GameOver { break }`, that is a plan deviation — no `break` is planned.
- Any `io::Error` from `poll`/`read`/`render` must propagate via `?`; no retry loops, no error screens (SPEC does not require them; Group C/D keep behavior).
- If `crossterm = "0.29"` resolves to an API-different version at future build time, treat THAT as a build-time blocker, not a plan deviation (record in the adherence report).
- No compiler exists here: a reviewer must not demand `cargo build`/`cargo check` output; static checks of §10 (B7) are the verification contract.
