use snake::game::food_placement::choose_food_position;
use snake::game::position::Position;
use snake::game::state::{HEIGHT, WIDTH, is_inside_board};

const FREE_BOARD_SAMPLE_COUNT: usize = 8;
const OCCUPIED_BOARD_SAMPLE_COUNT: usize = 20;

fn all_board_positions(except: Option<Position>) -> Vec<Position> {
    let mut occupied = Vec::new();
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            occupied.push(Position { x, y });
        }
    }
    occupied.retain(|position| Some(*position) != except);
    occupied
}

#[test]
fn free_board_yields_positions_inside_the_board() {
    for _ in 0..FREE_BOARD_SAMPLE_COUNT {
        let chosen_position = choose_food_position(&[])
            .expect("a free board always yields a position");
        assert!(is_inside_board(chosen_position));
    }
}

#[test]
fn chosen_positions_never_overlap_occupied_cells() {
    let occupied = vec![
        Position { x: 1, y: 1 },
        Position { x: 2, y: 2 },
        Position { x: 3, y: 3 },
    ];
    for _ in 0..OCCUPIED_BOARD_SAMPLE_COUNT {
        let chosen_position = choose_food_position(&occupied)
            .expect("free cells remain, so a position is always found");
        assert!(!occupied.contains(&chosen_position));
        assert!(is_inside_board(chosen_position));
    }
}

#[test]
fn some_is_returned_until_the_last_free_cell_is_reached() {
    let free = Position { x: WIDTH - 1, y: HEIGHT - 1 };
    let occupied = all_board_positions(Some(free));
    assert_eq!(choose_food_position(&occupied), Some(free));
}

#[test]
fn full_board_returns_none_explicitly() {
    let occupied = all_board_positions(None);
    assert_eq!(choose_food_position(&occupied), None);
}
