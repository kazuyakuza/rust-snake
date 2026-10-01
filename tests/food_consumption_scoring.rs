use snake::game::setup::initial_setup;
use snake::game::state::{GameState, is_inside_board};

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn playing_game() -> GameState {
    let mut game = fresh_game();
    game.start_playing();
    game
}

fn steps_to_reach_food() -> i32 {
    let setup = initial_setup();
    setup.food.position().x - setup.snake.head().x
}

fn game_that_has_eaten() -> GameState {
    let mut game = playing_game();
    let steps = steps_to_reach_food();
    for _ in 0..steps {
        game.advance_one_step();
    }
    game
}

#[test]
fn head_reaching_the_food_consumes_it() {
    let playing = playing_game();
    let food_cell = playing.food().position();
    let game = game_that_has_eaten();
    assert_eq!(game.snake().head(), food_cell);
}

#[test]
fn eating_food_increments_score_by_exactly_one() {
    assert_eq!(game_that_has_eaten().score(), 1);
}

#[test]
fn eating_food_grows_the_snake_by_exactly_one_segment() {
    assert_eq!(game_that_has_eaten().snake().length(), 4);
}

#[test]
fn new_food_after_consumption_is_valid() {
    let game = game_that_has_eaten();
    assert!(is_inside_board(game.food().position()));
    assert!(!game.snake().segments().contains(&game.food().position()));
}
