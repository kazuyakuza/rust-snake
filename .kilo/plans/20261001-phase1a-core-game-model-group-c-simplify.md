# Simplification Plan — Phase 1A, Group C (step 4.3, simplifier half)

**Source scope:** commits `c06e0f9`, `ee97aab`, `0a1b8d1`, `5f565d5` on `feat/phase1a-core-game-model`.
**Files reviewed:** `src/game/setup.rs`, `src/game/collision.rs`, `src/game/food_placement.rs`, `src/game/state.rs`, `src/game/food.rs`, `src/game.rs`.
**Binding decisions:** D1–D10 + §7 pre-decided out-of-fix-bounds list of `.kilo/plans/20261001-phase1a-core-game-model-group-c.md`. Nothing below contradicts them.

**Verdict:** Group C landed essentially verbatim to plan §2. Exactly ONE micro-simplification is warranted (S1). All other candidates were examined and rejected (either already optimal, or pre-decided out of fix bounds by the plan).

---

## 1. Change to apply

### S1. Drop the redundant type annotation in `respawn_food`

- File: `src/game/state.rs`, line 129 (inside `fn respawn_food`).
- Before (exact):

  ```rust
        let occupied: Vec<Position> = self.snake.segments().to_vec();
  ```

- After (exact):

  ```rust
        let occupied = self.snake.segments().to_vec();
  ```

- Why safe: `to_vec()` on `&[Position]` infers `Vec<Position>` unambiguously; behavior identical; the borrow-check `to_vec()` snapshot itself is pre-decided by plan §7 and is KEPT — only the incidental annotation is dropped. No other line of `respawn_food` changes.
- Side effect: the `Position` import in `state.rs` (line 12) is still used by `is_immediate_reversal`? — NO. Verify: after S1, `Position` remains used in `state.rs` at the `advance_one_step` line `let next_head = ...` only if typed; it is NOT typed there. Current genuine uses of the `Position` path in `state.rs` after S1: `pub fn is_inside_board(position: Position)` (line 27). Import stays; it is used. Do not remove it.
- Commit ownership: caller (workflow step) decides; suggested message if a separate commit is wanted: `refactor: drop redundant type annotation in respawn_food`. If the caller folds S1 into the existing Group C history, that decision is NOT made here.

---

## 2. Candidates examined and REJECTED (do not act on them)

1. **`is_outside_board` wrapper (collision.rs:9–11)** — verified it delegates exactly as plan D5 mandates (`!is_inside_board(next_head)`). Thin negation wrapper, bounds rule stays single-sourced in `state.rs`. Removing it would contradict D5. Keep.
2. **`head_to_body_slice` (collision.rs:20–25)** — clear, guarded, named constant. The `is_empty()` early return is REQUIRED (unguarded `len() - TAIL_SEGMENT_COUNT` underflows/pansics on an empty slice; Group D may pass `&[]`). A `saturating_sub` one-liner is shorter but plan D5 pins the helper "exactly as (no alternative forms)". Keep.
3. **`choose_food_position(occupied: &[Position]) -> Option<Position>` shape** — a plain slice, not a trait/generic injection; the simplest seam that satisfies plan D3's Group-D determinism requirement and avoids a `&GameState` back-dependency. Does NOT contradict the TODO's "no unnecessary abstractions". Keep.
4. **Two `enter_game_over(); return;` blocks in `advance_one_step`** — merging with `||` would violate the single-section boolean rule; plan D7 pins the exact per-tick order (brief §7). Keep.
5. **`pub` on `start_playing` / `enter_game_over` / `MIN_AVAILABLE_COORDINATE` / `is_inside_board` / `choose_food_position` / `occupies`** — exactly the six cross-module `pub` additions of plan D10; `start_playing`'s dead-code lint and `enter_game_over`'s current self-only caller are pre-decided (§7, §469 handoff). `is_playing`, `respawn_food`, `is_board_full`, `random_board_position`, `head_to_body_slice` verified PRIVATE. No needless pub found.
6. **`Vec::from([...])` in setup.rs:17 and struct-after-fn ordering** — Group B content moved byte-verbatim per D8 ("no edits to bodies or names"), 3-file split layout pre-decided in §7. Out of Group C bounds.
7. **Two-section conditions in `is_within_bounds`/`is_inside_board` (state.rs:24,28)** — Group A legacy, explicitly retained out-of-scope by plan §5.10. No Group C-added condition violates the rule.

---

## 3. Rule audit result (Group C files)

- Caps: largest file `state.rs` 134 total (<200); largest body `advance_one_step` 19 lines (<50). ✓
- Args: max 2 (`collides_with_body`). ✓ Depth: max 2 (`choose_food_position` loop→if). ✓
- Named constants: `SCORE_INCREMENT`, `TAIL_SEGMENT_COUNT`, `TOTAL_BOARD_CELLS` present; no new magic numbers. ✓
- No commented-out code; no unused imports (verified call-sites of every import in the 6 files). ✓
- No duplicated logic between `state.rs` and `collision.rs` beyond the mandated D5 delegation. ✓

## 4. Informational handoff — NOT changes for this step

- **state.rs sizing:** 134 total lines; 20 blank + 11 comment/doc + 7 import lines → **96 code lines** (103 including imports). Under the 125 code-line ideal.
- **Group D threat assessment:** inline `#[cfg(test)] mod tests` in `state.rs` for task-14 state-related tests (initial length/score, direction rules, movement, growth, consume/score, both collisions, transitions ≈ 10+ tests, realistically +120–180 lines) would breach the 200-line hard cap (134+120=254) long before usefulness. **Recommendation for the Group D planning step: place tests in separate files under `tests/` (integration style).** The Group C public surface is sufficient for that (`GameState` getters, `change_direction`, `advance_one_step`, `start_playing`, `enter_game_over`, `collision::*`, `food_placement::choose_food_position`, `Food::occupies`, `Snake`/`Position`/`Direction`/`setup::initial_setup` all `pub`); `tests/` files sit outside the `src/`-only line caps; this also justifies keeping `start_playing`/`enter_game_over` `pub`.
- **Step 4.4 (docs-specialist) note:** `state.rs` module doc (lines 1–6) still describes only "board dimensions and bounds" — stale since Group C added the state machine + tick driver. Plan §7 already assigns the `state.rs` description update to 4.4; not edited by the simplifier.
