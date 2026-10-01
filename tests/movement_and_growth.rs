use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::snake::Snake;
use snake::game::state::GameState;

const NORMAL_STEPS_TO_ADVANCE: i32 = 5;

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn playing_game() -> GameState {
    let mut game = fresh_game();
    game.start_playing();
    game
}

#[test]
fn step_moves_head_one_cell_in_the_current_direction() {
    let mut game = playing_game();
    let head_before = game.snake().head();
    game.advance_one_step();
    assert_eq!(game.snake().head(), head_before + Direction::Right.offset());
}

#[test]
fn normal_step_preserves_snake_length() {
    let mut game = playing_game();
    for _ in 0..NORMAL_STEPS_TO_ADVANCE {
        game.advance_one_step();
    }
    assert_eq!(game.snake().length(), 3);
}

#[test]
fn body_follows_the_head() {
    let mut game = playing_game();
    let segments_before = game.snake().segments().to_vec();
    game.advance_one_step();
    assert_eq!(game.snake().segments()[1], segments_before[0]);
    assert_eq!(game.snake().segments()[2], segments_before[1]);
}

#[test]
fn advance_does_nothing_before_playing() {
    let mut game = fresh_game();
    let head_before = game.snake().head();
    let length_before = game.snake().length();
    game.advance_one_step();
    assert_eq!(game.snake().head(), head_before);
    assert_eq!(game.snake().length(), length_before);
}

#[test]
fn growth_step_adds_exactly_one_segment() {
    let body = vec![Position { x: 0, y: 0 }, Position { x: -1, y: 0 }];
    let mut snake = Snake::new(body);
    snake.advance(Position { x: 1, y: 0 }, false);
    assert_eq!(snake.length(), 3);
}

#[test]
fn growth_step_keeps_the_previous_tail_tip() {
    let body = vec![Position { x: 0, y: 0 }, Position { x: -1, y: 0 }];
    let mut snake = Snake::new(body);
    snake.advance(Position { x: 1, y: 0 }, false);
    assert_eq!(snake.segments().last(), Some(&Position { x: -1, y: 0 }));
}

#[test]
fn normal_step_removes_the_tail() {
    let body = vec![Position { x: 0, y: 0 }, Position { x: -1, y: 0 }];
    let mut snake = Snake::new(body);
    snake.advance(Position { x: 1, y: 0 }, true);
    assert_eq!(snake.length(), 2);
    assert_eq!(snake.segments().last(), Some(&Position { x: 0, y: 0 }));
}
