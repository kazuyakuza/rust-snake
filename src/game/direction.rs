use crate::game::position::Position;

const ZERO_GRID_STEP: i32 = 0;
const SINGLE_GRID_STEP: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// One-cell grid offset for this direction, in terminal-grid orientation:
    /// `y` grows downward, so `Up` decreases `y`.
    pub fn offset(self) -> Position {
        match self {
            Direction::Up => Position { x: ZERO_GRID_STEP, y: -SINGLE_GRID_STEP },
            Direction::Down => Position { x: ZERO_GRID_STEP, y: SINGLE_GRID_STEP },
            Direction::Left => Position { x: -SINGLE_GRID_STEP, y: ZERO_GRID_STEP },
            Direction::Right => Position { x: SINGLE_GRID_STEP, y: ZERO_GRID_STEP },
        }
    }
}
