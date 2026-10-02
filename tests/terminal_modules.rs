use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyEventState};

use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, HEIGHT};
use snake::terminal::game_loop::tick;
use snake::terminal::renderer::Renderer;
use snake::terminal::input::map_key_event_to_direction;

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn arrow_press(key_code: KeyCode) -> KeyEvent {
    KeyEvent { code: key_code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }
}

fn arrow_release(key_code: KeyCode) -> KeyEvent {
    KeyEvent { code: key_code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, state: KeyEventState::NONE }
}

fn count_occurrences(buffer: &[u8], text: &str) -> usize {
    buffer
        .windows(text.len())
        .filter(|byte_window| *byte_window == text.as_bytes())
        .count()
}

#[test]
fn arrow_presses_map_to_the_four_directions() {
    let arrow_cases = [
        (KeyCode::Up, Direction::Up),
        (KeyCode::Down, Direction::Down),
        (KeyCode::Left, Direction::Left),
        (KeyCode::Right, Direction::Right),
    ];
    for (key_code, expected) in arrow_cases {
        let mapped = map_key_event_to_direction(&arrow_press(key_code));
        assert_eq!(mapped, Some(expected));
    }
}

#[test]
fn non_arrow_press_maps_to_none() {
    let mapped = map_key_event_to_direction(&arrow_press(KeyCode::Char('a')));
    assert_eq!(mapped, None);
}

#[test]
fn key_release_maps_to_none() {
    let mapped = map_key_event_to_direction(&arrow_release(KeyCode::Left));
    assert_eq!(mapped, None);
}

#[test]
fn tick_applies_directions_before_one_step_and_rejects_reversal() {
    let mut state = fresh_game();
    state.start_playing();

    let mut buffer = Vec::new();
    let tick_status;
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick_status = tick(&mut state, &[Direction::Up, Direction::Down], &mut renderer)
            .expect("tick succeeds");
    }

    assert_eq!(state.current_direction(), Direction::Up);
    assert_eq!(state.snake().head(), Position { x: 10, y: 11 });
    assert_eq!(tick_status, GameStatus::Playing);
}

#[test]
fn tick_advances_exactly_one_step_per_call() {
    let mut state = fresh_game();
    state.start_playing();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick(&mut state, &[], &mut renderer).expect("first tick succeeds");
        assert_eq!(state.snake().head(), Position { x: 11, y: 12 });
        tick(&mut state, &[], &mut renderer).expect("second tick succeeds");
    }

    assert_eq!(state.snake().head(), Position { x: 12, y: 12 });
}

#[test]
fn tick_returns_game_over_when_the_head_exits_the_boundary() {
    let mut state = fresh_game();
    state.start_playing();

    let mut buffer = Vec::new();
    let mut tick_status;
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick_status =
            tick(&mut state, &[Direction::Down], &mut renderer).expect("tick steers down");
        let ticks_until_bottom_wall = (HEIGHT - 1) - state.snake().head().y;
        for _ in 0..=ticks_until_bottom_wall {
            tick_status =
                tick(&mut state, &[], &mut renderer).expect("tick succeeds");
        }
    }

    assert_eq!(tick_status, GameStatus::GameOver);
    assert_eq!(state.snake().head().y, HEIGHT - 1);
}

#[test]
fn tick_is_a_no_move_before_playing() {
    let mut state = fresh_game();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick(&mut state, &[], &mut renderer).expect("tick succeeds");
    }

    assert_eq!(state.status(), GameStatus::WaitingToStart);
    assert_eq!(state.snake().head(), Position { x: 10, y: 12 });
}

#[test]
fn tick_returns_the_new_status_playing() {
    let mut state = fresh_game();
    state.start_playing();

    let mut buffer = Vec::new();
    let tick_status;
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick_status = tick(&mut state, &[], &mut renderer).expect("tick succeeds");
    }

    assert_eq!(tick_status, GameStatus::Playing);
}

#[test]
fn render_writes_the_full_frame_into_the_buffer() {
    let state = fresh_game();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        renderer.render(&state).expect("render succeeds");
    }

    assert_eq!(count_occurrences(&buffer, "+"), 4);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
    assert_eq!(count_occurrences(&buffer, "●"), 1);
    assert_eq!(count_occurrences(&buffer, "■"), 2);
    assert_eq!(count_occurrences(&buffer, "◆"), 1);
}

#[test]
fn tick_renders_one_consistent_frame_of_glyphs() {
    let mut state = fresh_game();
    state.start_playing();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        tick(&mut state, &[], &mut renderer).expect("tick succeeds");
    }

    assert_eq!(count_occurrences(&buffer, "●"), 1);
    assert_eq!(count_occurrences(&buffer, "■"), 2);
    assert_eq!(count_occurrences(&buffer, "◆"), 1);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 1);
}

#[test]
fn two_renders_reuse_the_frame_without_scrolling() {
    let state = fresh_game();

    let mut buffer = Vec::new();
    {
        let mut renderer = Renderer::new(&mut buffer);
        renderer.render(&state).expect("first render succeeds");
        renderer.render(&state).expect("second render succeeds");
    }

    assert_eq!(count_occurrences(&buffer, "+"), 8);
    assert_eq!(count_occurrences(&buffer, "Score: 0"), 2);
    assert_eq!(count_occurrences(&buffer, "●"), 2);
}
