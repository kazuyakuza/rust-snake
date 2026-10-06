# Testing Guide

Integration tests live in `tests/` and exercise the public `snake` API. There is
no local Rust toolchain on the host (see [BUILD.md](BUILD.md)); tests run inside
the pinned Docker image (`rust:1.98.1-slim-bookworm`). `CARGO_TARGET_DIR=/tmp/target`
keeps the generated `target/` inside the container so the host checkout stays clean.

## Running a single suite

From the Alpine VM, mount the repo and run one integration target:

```text
docker run --rm -v /rust-snake:/project -w /project -e CARGO_TARGET_DIR=/tmp/target rust:1.98.1-slim-bookworm cargo test --test direction_swap_reversal
```

Replace `--test direction_swap_reversal` with another target name, or drop the
flag to run the whole suite.

## Rapid-direction-swap regression suite

`tests/direction_swap_reversal.rs` pins the rule that several arrow presses
drained inside **one** tick must never steer the head onto its own body. It
reproduces the circling-death bug and guards the related input behaviors. The
suite drives the initial 3-segment snake (head `(10, 12)`, heading `Right`, score
`0`) through the domain layer (`GameState::change_direction` +
`advance_one_step`) and, for the loop cases, through `terminal::game_loop::tick`
with an in-memory `Vec<u8>` renderer (`HeadlessLoop`). Helpers:
`playing_game`, `apply_and_step`, `assert_snake_survived`.

The suite is currently split 5 / 5: five tests are bug repros that **fail** until
the direction-handling fix lands, and five are regression pins that pass now and
must stay green.

| Test | Layer | Status |
| --- | --- | --- |
| `two_key_burst_within_one_tick_must_not_step_onto_the_neck` | domain | Fails until fix |
| `three_key_burst_ending_in_reversal_must_not_step_onto_the_neck` | domain | Fails until fix |
| `two_tick_interleaved_burst_must_not_reenter_the_body` | domain | Fails until fix |
| `loop_tick_survives_a_two_key_burst_in_one_tick` | loop | Fails until fix |
| `loop_tick_survives_a_three_key_burst_ending_in_reversal` | loop | Fails until fix |
| `three_key_burst_not_ending_in_reversal_stays_alive` | domain | Passes (pin) |
| `one_turn_per_tick_circles_back_to_the_start_cell` | domain | Passes (pin) |
| `right_angle_turn_between_ticks_still_turns` | domain | Passes (pin) |
| `double_key_turn_within_one_tick_toward_free_cells_still_turns` | domain | Passes (pin) |
| `loop_tick_survives_continuous_circling_for_two_revolutions` | loop | Passes (pin) |

Two pins define the boundary the fix must not cross:
`double_key_turn_within_one_tick_toward_free_cells_still_turns` pins the
`[Right, Down]` one-tick turn (a legitimate double-turn toward a free cell, so
the final direction is `Down`), while the failing `..._three_key_burst_ending_in_reversal_...`
tests pin that a burst ending opposite the last-moved direction must be rejected.
`one_turn_per_tick_circles_back_to_the_start_cell` and
`loop_tick_survives_continuous_circling_for_two_revolutions` assert a one-turn-per-tick
circle returns the head to `(10, 12)` without any self-overlap.
