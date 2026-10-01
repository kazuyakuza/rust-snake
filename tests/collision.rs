use snake::game::collision::{collides_with_body, is_outside_board};
use snake::game::direction::Direction;
use snake::game::position::Position;
use snake::game::setup::initial_setup;
use snake::game::state::{GameStatus, GameState, HEIGHT, WIDTH, is_inside_board};

fn fresh_game() -> GameState {
    GameState::new(initial_setup())
}

fn playing_game() -> GameState {
    let mut game = fresh_game();
    game.start_playing();
    game
}

#[test]
fn cells_inside_the_board_are_not_outside() {
    let minimum_corner = Position { x: 0, y: 0 };
    let maximum_corner = Position { x: WIDTH - 1, y: HEIGHT - 1 };
    assert!(!is_outside_board(minimum_corner));
    assert!(!is_outside_board(maximum_corner));
    assert!(is_inside_board(minimum_corner));
    assert!(is_inside_board(maximum_corner));
}

#[test]
fn boundary_edges_of_every_axis_are_outside() {
    assert!(is_outside_board(Position { x: WIDTH, y: 0 }));
    assert!(is_outside_board(Position { x: 0, y: HEIGHT }));
    assert!(is_outside_board(Position { x: -1, y: 0 }));
    assert!(is_outside_board(Position { x: 0, y: -1 }));
}

#[test]
fn hitting_the_right_wall_ends_the_game_without_wraparound() {
    let mut game = playing_game();
    let steps_until_wall = WIDTH - game.snake().head().x;
    for _ in 0..steps_until_wall {
        game.advance_one_step();
    }
    assert_eq!(game.status(), GameStatus::GameOver);
    assert_eq!(game.snake().head().x, WIDTH - 1);
}

#[test]
fn hitting_the_bottom_wall_ends_the_game_without_wraparound() {
    let mut game = playing_game();
    game.change_direction(Direction::Down);
    let steps_until_floor = HEIGHT - game.snake().head().y;
    for _ in 0..steps_until_floor {
        game.advance_one_step();
    }
    assert_eq!(game.status(), GameStatus::GameOver);
    assert_eq!(game.snake().head().y, HEIGHT - 1);
}

#[test]
fn head_landing_on_a_body_segment_is_a_collision() {
    let segments = vec![
        Position { x: 5, y: 5 },
        Position { x: 4, y: 5 },
        Position { x: 3, y: 5 },
    ];
    let next_head = Position { x: 4, y: 5 };
    assert!(collides_with_body(next_head, &segments));
}

#[test]
fn tail_cell_is_excluded_because_it_vacates() {
    let segments = vec![
        Position { x: 5, y: 5 },
        Position { x: 4, y: 5 },
        Position { x: 3, y: 5 },
    ];
    let next_head = Position { x: 3, y: 5 };
    assert!(!collides_with_body(next_head, &segments));
}

#[test]
fn head_front_cell_of_a_straight_snake_is_not_a_collision() {
    let segments = vec![
        Position { x: 5, y: 5 },
        Position { x: 4, y: 5 },
        Position { x: 3, y: 5 },
    ];
    let next_head = Position { x: 6, y: 5 };
    assert!(!collides_with_body(next_head, &segments));
}

#[test]
fn empty_snake_cannot_collide() {
    assert!(!collides_with_body(Position { x: 0, y: 0 }, &[]));
}

#[test]
fn single_segment_snake_cannot_collide() {
    let segments = vec![Position { x: 7, y: 7 }];
    let next_head = Position { x: 7, y: 7 };
    assert!(!collides_with_body(next_head, &segments));
}
