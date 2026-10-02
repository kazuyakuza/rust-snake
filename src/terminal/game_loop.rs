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
