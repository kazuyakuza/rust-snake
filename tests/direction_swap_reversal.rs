//! Regression tests for the rapid-direction-swap reversal bug: several
//! arrow presses drained inside ONE tick must never steer the head onto
//! its own body. The failing tests reproduce the bug against the unfixed
//! domain and loop layers and must turn green with the Task 2 fix.

use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;

const INITIAL_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_LENGTH: usize = 3;

fn playing_game() -> GameState {
    let mut game = GameState::new(initial_setup());
    game.start_playing();
    game
}

fn apply_and_step(game: &mut GameState, directions: &[Direction]) {
    for &direction in directions {
        game.change_direction(direction);
    }
    game.advance_one_step();
}

fn assert_snake_survived(game: &GameState, context: &str) {
    assert_ne!(
        game.status(),
        GameStatus::GameOver,
        "{context}: the snake must stay alive"
    );
    assert_eq!(game.snake().length(), INITIAL_SNAKE_LENGTH, "{context}: length unchanged");
    assert_eq!(game.score(), 0, "{context}: score unchanged");
    let segments = game.snake().segments();
    for (index, segment) in segments.iter().enumerate() {
        assert!(
            !segments[..index].contains(segment),
            "{context}: segment {index} overlaps an earlier segment"
        );
    }
}

#[test]
fn two_key_burst_within_one_tick_must_not_step_onto_the_neck() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up, Direction::Left]);
    assert_snake_survived(&game, "burst [Up, Left] inside one tick");
}

#[test]
fn three_key_burst_ending_in_reversal_must_not_step_onto_the_neck() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Right, Direction::Down, Direction::Left]);
    assert_snake_survived(&game, "burst [Right, Down, Left] inside one tick");
}

#[test]
fn two_tick_interleaved_burst_must_not_reenter_the_body() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up]);
    assert_snake_survived(&game, "after the [Up] tick");
    apply_and_step(&mut game, &[Direction::Left, Direction::Down]);
    assert_snake_survived(&game, "after the [Left, Down] burst tick");
}

#[test]
fn three_key_burst_not_ending_in_reversal_stays_alive() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Up, Direction::Left, Direction::Down]);
    assert_snake_survived(&game, "burst [Up, Left, Down] inside one tick");
}

#[test]
fn one_turn_per_tick_circles_back_to_the_start_cell() {
    let mut game = playing_game();
    let circling_turns = [
        [Direction::Up],
        [Direction::Left],
        [Direction::Down],
        [Direction::Right],
    ];
    for turn in &circling_turns {
        apply_and_step(&mut game, turn);
        assert_snake_survived(&game, "while circling one turn per tick");
    }
    assert_eq!(game.snake().head(), INITIAL_HEAD);
}

#[test]
fn right_angle_turn_between_ticks_still_turns() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[]);
    assert_eq!(game.snake().head(), INITIAL_HEAD + Direction::Right.offset());
    apply_and_step(&mut game, &[Direction::Down]);
    assert_eq!(
        game.snake().head(),
        INITIAL_HEAD + Direction::Right.offset() + Direction::Down.offset()
    );
    assert_eq!(game.current_direction(), Direction::Down);
    assert_snake_survived(&game, "after a plain turn between ticks");
}

#[test]
fn double_key_turn_within_one_tick_toward_free_cells_still_turns() {
    let mut game = playing_game();
    apply_and_step(&mut game, &[Direction::Right, Direction::Down]);
    assert_eq!(game.current_direction(), Direction::Down);
    assert_eq!(game.snake().head(), INITIAL_HEAD + Direction::Down.offset());
    assert_snake_survived(&game, "after the [Right, Down] one-tick turn");
}

struct HeadlessLoop {
    state: GameState,
    buffer: Vec<u8>,
}

impl HeadlessLoop {
    fn new() -> HeadlessLoop {
        let mut state = playing_game();
        HeadlessLoop { state, buffer: Vec::new() }
    }

    fn tick(&mut self, directions: &[Direction]) -> GameStatus {
        let mut renderer = Renderer::new(&mut self.buffer);
        tick(&mut self.state, directions, &mut renderer).expect("headless tick succeeds")
    }
}

#[test]
fn loop_tick_survives_a_two_key_burst_in_one_tick() {
    let mut game = HeadlessLoop::new();
    let status = game.tick(&[Direction::Up, Direction::Left]);
    assert_ne!(
        status,
        GameStatus::GameOver,
        "loop tick [Up, Left]: should not lose while swapping quickly in one tick"
    );
    assert_snake_survived(&game.state, "loop tick burst [Up, Left]");
}

#[test]
fn loop_tick_survives_a_three_key_burst_ending_in_reversal() {
    let mut game = HeadlessLoop::new();
    let status = game.tick(&[Direction::Right, Direction::Down, Direction::Left]);
    assert_ne!(
        status,
        GameStatus::GameOver,
        "loop tick [Right, Down, Left]: should not lose while swapping quickly in one tick"
    );
    assert_snake_survived(&game.state, "loop tick burst [Right, Down, Left]");
}

#[test]
fn loop_tick_survives_continuous_circling_for_two_revolutions() {
    let mut game = HeadlessLoop::new();
    let circling_turns = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];
    for &turn in &circling_turns {
        let status = game.tick(&[turn]);
        assert_ne!(
            status,
            GameStatus::GameOver,
            "loop tick circling turn {turn:?}: should not lose while circling"
        );
        assert_snake_survived(&game.state, "loop tick circling");
    }
    assert_eq!(game.state.snake().head(), INITIAL_HEAD);
}
