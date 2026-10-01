use snake::game::direction::Direction;
use snake::game::setup::initial_setup;
use snake::game::state::GameState;

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

#[test]
fn right_changes_to_up() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert_eq!(game.current_direction(), Direction::Up);
}

#[test]
fn right_changes_to_down() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Down));
    assert_eq!(game.current_direction(), Direction::Down);
}

#[test]
fn up_changes_to_left() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(game.change_direction(Direction::Left));
    assert_eq!(game.current_direction(), Direction::Left);
}

#[test]
fn up_changes_to_right() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(game.change_direction(Direction::Right));
    assert_eq!(game.current_direction(), Direction::Right);
}

#[test]
fn down_changes_to_left() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Down));
    assert!(game.change_direction(Direction::Left));
    assert_eq!(game.current_direction(), Direction::Left);
}

#[test]
fn down_changes_to_right() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Down));
    assert!(game.change_direction(Direction::Right));
    assert_eq!(game.current_direction(), Direction::Right);
}

#[test]
fn left_changes_to_up() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(game.change_direction(Direction::Left));
    assert!(game.change_direction(Direction::Up));
    assert_eq!(game.current_direction(), Direction::Up);
}

#[test]
fn left_changes_to_down() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(game.change_direction(Direction::Left));
    assert!(game.change_direction(Direction::Down));
    assert_eq!(game.current_direction(), Direction::Down);
}

#[test]
fn same_direction_change_is_accepted() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Right));
    assert_eq!(game.current_direction(), Direction::Right);
}

#[test]
fn right_rejects_left() {
    let mut game = fresh_game();
    assert!(!game.change_direction(Direction::Left));
    assert_eq!(game.current_direction(), Direction::Right);
}

#[test]
fn left_rejects_right() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(game.change_direction(Direction::Left));
    assert!(!game.change_direction(Direction::Right));
    assert_eq!(game.current_direction(), Direction::Left);
}

#[test]
fn up_rejects_down() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Up));
    assert!(!game.change_direction(Direction::Down));
    assert_eq!(game.current_direction(), Direction::Up);
}

#[test]
fn down_rejects_up() {
    let mut game = fresh_game();
    assert!(game.change_direction(Direction::Down));
    assert!(!game.change_direction(Direction::Up));
    assert_eq!(game.current_direction(), Direction::Down);
}
