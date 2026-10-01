use crate::game::position::Position;
use crate::game::state::{HEIGHT, WIDTH, MIN_AVAILABLE_COORDINATE};
use rand::Rng;
use rand::rngs::ThreadRng;

const TOTAL_BOARD_CELLS: usize = (WIDTH * HEIGHT) as usize;

/// Choose a random free cell for the food, retrying on snake-occupied picks.
/// Returns `None` when every cell is occupied: the explicit signal that no
/// respawn is possible, with no loop iteration entered at all.
pub fn choose_food_position(occupied: &[Position]) -> Option<Position> {
    if is_board_full(occupied) {
        return None;
    }
    let mut board_rng = rand::thread_rng();
    loop {
        let candidate = random_board_position(&mut board_rng);
        if !occupied.contains(&candidate) {
            return Some(candidate);
        }
    }
}

fn is_board_full(occupied: &[Position]) -> bool {
    occupied.len() >= TOTAL_BOARD_CELLS
}

fn random_board_position(board_rng: &mut ThreadRng) -> Position {
    Position {
        x: board_rng.gen_range(MIN_AVAILABLE_COORDINATE..=WIDTH - 1),
        y: board_rng.gen_range(MIN_AVAILABLE_COORDINATE..=HEIGHT - 1),
    }
}
