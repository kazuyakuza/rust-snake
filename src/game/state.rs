//! Board dimensions and the canonical playable bounds for the game grid.
//!
//! `WIDTH`/`HEIGHT` are cell counts (not terminal pixels). A coordinate is on the
//! board when it lies in the inclusive range `MIN_AVAILABLE_COORDINATE..=WIDTH - 1`
//! (`x`) or `..=HEIGHT - 1` (`y`); anything outside is a boundary collision, with no
//! wrap-around.

use crate::game::direction::Direction;
use crate::game::food::Food;
use crate::game::position::Position;
use crate::game::snake::Snake;

pub const WIDTH: i32 = 40;
pub const HEIGHT: i32 = 25;

const INITIAL_SCORE: i32 = 0;
const MIN_AVAILABLE_COORDINATE: i32 = 0;

fn is_within_bounds(value: i32, max_inclusive: i32) -> bool {
    value >= MIN_AVAILABLE_COORDINATE && value <= max_inclusive
}

pub fn is_inside_board(position: Position) -> bool {
    is_within_bounds(position.x, WIDTH - 1) && is_within_bounds(position.y, HEIGHT - 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    WaitingToStart,
    Playing,
    GameOver,
}

pub struct GameStateSetup {
    pub snake: Snake,
    pub food: Food,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub struct GameState {
    snake: Snake,
    food: Food,
    current_direction: Direction,
    score: i32,
    status: GameStatus,
}

impl GameState {
    pub fn new(setup: GameStateSetup) -> GameState {
        GameState {
            snake: setup.snake,
            food: setup.food,
            current_direction: setup.direction,
            score: INITIAL_SCORE,
            status: GameStatus::WaitingToStart,
        }
    }

    pub fn snake(&self) -> &Snake {
        &self.snake
    }

    pub fn food(&self) -> &Food {
        &self.food
    }

    pub fn current_direction(&self) -> Direction {
        self.current_direction
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }
}
