use snake::game::direction::Direction;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, is_inside_board};

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

#[test]
fn initial_snake_has_exactly_three_segments() {
    let game = fresh_game();
    assert_eq!(game.snake().length(), 3);
    assert_eq!(game.snake().segments().len(), 3);
}

#[test]
fn initial_snake_head_leads_two_body_segments() {
    let game = fresh_game();
    assert_eq!(game.snake().head(), game.snake().segments()[0]);
    assert_ne!(game.snake().segments()[1], game.snake().head());
    assert_ne!(game.snake().segments()[2], game.snake().head());
    assert_ne!(game.snake().segments()[1], game.snake().segments()[2]);
}

#[test]
fn initial_score_is_zero() {
    assert_eq!(fresh_game().score(), 0);
}

#[test]
fn initial_direction_is_right() {
    assert_eq!(fresh_game().current_direction(), Direction::Right);
}

#[test]
fn initial_status_is_waiting_to_start() {
    assert_eq!(fresh_game().status(), GameStatus::WaitingToStart);
}

#[test]
fn initial_snake_is_inside_the_board() {
    let game = fresh_game();
    for segment in game.snake().segments() {
        assert!(is_inside_board(*segment));
    }
}

#[test]
fn initial_food_is_valid() {
    let game = fresh_game();
    assert!(is_inside_board(game.food().position()));
    assert!(!game.snake().segments().contains(&game.food().position()));
}
