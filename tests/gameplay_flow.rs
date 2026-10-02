//! Headless walkthrough of the complete gameplay flow (`src/main.rs` wiring).
//!
//! ```text
//! Start (main)          -> GameState::new(initial_setup()), status WaitingToStart    -> covered: test 1
//! Press any key         -> state.start_playing() (domain edge; the key-wait itself
//!                          is binary-private and validated manually / Phase 2)       -> covered: test 1
//! Playing               -> per tick: drain -> change_direction (reversals rejected)
//!                          -> advance_one_step -> render                             -> covered: test 2 + tests/terminal_modules.rs tick tests
//! Collision             -> enter_game_over() via the domain predicates                -> covered: tests 3 (boundary), 4 (self)
//! Game over / Exit      -> GAME OVER + Score: N + wait key + TerminalHandle Drop      -> manual (docs/terminal-ui.md), Phase 2 run
//! ```
//!
//! Static trace of the `src/main.rs` wiring behind that chain: line 27 builds
//! the state (`WaitingToStart`) -> test 1; line 28 enables the terminal ->
//! manual (touches the real console); lines 30-31 start screen and key wait ->
//! manual; line 32 `start_playing()` -> test 1; lines 34-35 create the
//! renderer and run `run_playing_loop` -> `tick` is the headless stand-in for
//! the loop body (tick tests here and in `tests/terminal_modules.rs`; timing
//! and the real event drain stay excluded); lines 37-38 game-over screen and
//! exit wait -> manual; the dropped `TerminalHandle` restores the terminal ->
//! manual (Drop guard, real console).
//!
//! The screens and key-waits cannot be tested headlessly. The tests below
//! assert the domain and loop state transitions that the wiring triggers in
//! the identical order `main.rs` uses: setup -> start gate -> start_playing ->
//! ticks -> collision -> GameOver -> final score preserved.

use std::io;

use snake::game::direction::Direction;
use snake::game::food::Food;
use snake::game::position::Position;
use snake::game::setup::{GameStateSetup, initial_setup};
use snake::game::snake::Snake;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn hooked_snake_game() -> GameState {
    let hook_segments = vec![
        Position { x: 10, y: 10 },
        Position { x: 9, y: 10 },
        Position { x: 8, y: 10 },
        Position { x: 7, y: 10 },
        Position { x: 6, y: 10 },
    ];
    GameState::new(GameStateSetup {
        snake: Snake::new(hook_segments),
        food: Food::new(Position { x: 30, y: 20 }),
        direction: Direction::Right,
    })
}

fn count_occurrences(buffer: &[u8], text: &str) -> usize {
    buffer
        .windows(text.len())
        .filter(|byte_window| *byte_window == text.as_bytes())
        .count()
}

#[test]
fn flow_starts_only_after_a_key_press() {
    let mut game = fresh_game();
    assert_eq!(game.status(), GameStatus::WaitingToStart);

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick(&mut game, &[], &mut renderer).expect("tick succeeds while waiting");
    }
    assert_eq!(game.status(), GameStatus::WaitingToStart);
    assert_eq!(game.snake().head(), Position { x: 10, y: 12 });

    game.start_playing();
    assert_eq!(game.status(), GameStatus::Playing);
}

#[test]
fn flow_moves_eats_and_grows_during_ticks() {
    let mut game = fresh_game();
    game.start_playing();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        for _ in 0..10 {
            tick(&mut game, &[], &mut renderer).expect("tick succeeds while playing");
        }
    }

    assert_eq!(game.status(), GameStatus::Playing);
    assert_eq!(game.snake().head(), Position { x: 20, y: 12 });
    assert_eq!(game.score(), 1);
    assert_eq!(game.snake().length(), 4);
    assert_ne!(game.food().position(), Position { x: 20, y: 12 });
    assert_eq!(count_occurrences(&buffer, "Score: 1"), 1);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 9);
}

#[test]
fn flow_boundary_collision_returns_game_over() {
    let mut game = fresh_game();
    game.start_playing();

    let mut buffer = Vec::new();
    let mut tick_status;
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick_status = tick(&mut game, &[Direction::Down], &mut renderer)
            .expect("tick steers down");
        assert_eq!(game.current_direction(), Direction::Down);
        assert_eq!(game.snake().head(), Position { x: 10, y: 13 });

        let ticks_until_bottom_wall = (HEIGHT - 1) - game.snake().head().y;
        for _ in 0..=ticks_until_bottom_wall {
            tick_status =
                tick(&mut game, &[], &mut renderer).expect("tick succeeds");
        }
    }

    assert_eq!(tick_status, GameStatus::GameOver);
    assert_eq!(game.status(), GameStatus::GameOver);
    assert_eq!(game.snake().head().y, HEIGHT - 1);
    assert_eq!(game.score(), 0);
    assert!(count_occurrences(&buffer, "Score: 0") > 0);
}

#[test]
fn flow_self_collision_returns_game_over() {
    let mut game = hooked_snake_game();
    game.start_playing();
    assert_eq!(game.snake().length(), 5);

    let mut buffer = Vec::new();
    let fatal_tick_status;
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick(&mut game, &[], &mut renderer).expect("tick moves right");
        tick(&mut game, &[Direction::Up], &mut renderer).expect("tick steers up");
        tick(&mut game, &[Direction::Left], &mut renderer).expect("tick steers left");
        assert_eq!(game.status(), GameStatus::Playing);
        assert_eq!(game.snake().length(), 5);
        fatal_tick_status =
            tick(&mut game, &[Direction::Down], &mut renderer).expect("tick steers down");
    }

    assert_eq!(fatal_tick_status, GameStatus::GameOver);
    assert_eq!(game.status(), GameStatus::GameOver);
    assert_eq!(game.snake().head(), Position { x: 10, y: 9 });
    assert_eq!(game.snake().length(), 5);
    assert_eq!(game.score(), 0);
}
