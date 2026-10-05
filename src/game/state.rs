//! Central game state: board dimensions and playable bounds, the status state
//! machine, and the per-tick move/consume/collide driver.
//!
//! `WIDTH`/`HEIGHT` are cell counts (not terminal pixels); a coordinate is inside
//! the board when it lies in the inclusive range `MIN_AVAILABLE_COORDINATE..=WIDTH - 1`
//! (`x`) or `..=HEIGHT - 1` (`y`), with no wrap-around. `GameState` owns the snake,
//! food, direction, score, and `GameStatus`; `advance_one_step` resolves a single
//! move: boundary and self collisions, food consumption, and status transitions.

use crate::game::collision;
use crate::game::direction::Direction;
use crate::game::food::Food;
use crate::game::food_placement;
use crate::game::position::Position;
use crate::game::setup::GameStateSetup;
use crate::game::snake::Snake;

pub const WIDTH: i32 = 80;
pub const HEIGHT: i32 = 80;

const INITIAL_SCORE: i32 = 0;
pub const MIN_AVAILABLE_COORDINATE: i32 = 0;
const SCORE_INCREMENT: i32 = 1;

fn is_within_bounds(value: i32, max_inclusive: i32) -> bool {
    value >= MIN_AVAILABLE_COORDINATE && value <= max_inclusive
}

pub fn is_inside_board(position: Position) -> bool {
    is_within_bounds(position.x, WIDTH - 1) && is_within_bounds(position.y, HEIGHT - 1)
}

fn is_immediate_reversal(current: Direction, candidate: Direction) -> bool {
    candidate == current.opposite()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    WaitingToStart,
    Playing,
    GameOver,
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

    /// Steer the snake, rejecting an immediate reversal into itself. Returns
    /// `true` when the direction changed, or `false` when `new_direction` is
    /// directly opposite the current one (the direction is then left unchanged).
    pub fn change_direction(&mut self, new_direction: Direction) -> bool {
        if is_immediate_reversal(self.current_direction, new_direction) {
            return false;
        }
        self.current_direction = new_direction;
        true
    }

    /// Advance one playing tick: move onto the next cell, or end the game on a
    /// boundary or self collision. Does nothing unless the game is playing.
    pub fn advance_one_step(&mut self) {
        if !self.is_playing() {
            return;
        }
        let next_head = self.snake.head() + self.current_direction.offset();
        if collision::is_outside_board(next_head) {
            self.enter_game_over();
            return;
        }
        if collision::collides_with_body(next_head, self.snake.segments()) {
            self.enter_game_over();
            return;
        }
        let will_consume = self.food.occupies(next_head);
        self.snake.advance(next_head, !will_consume);
        if will_consume {
            self.score += SCORE_INCREMENT;
            self.respawn_food();
        }
    }

    fn is_playing(&self) -> bool {
        self.status == GameStatus::Playing
    }

    /// Begin play: the `WaitingToStart -> Playing` edge. The domain never starts
    /// itself; the later terminal phase calls this after the start key is pressed.
    pub fn start_playing(&mut self) {
        self.status = GameStatus::Playing;
    }

    /// End the game: the `-> GameOver` edge, reached when `advance_one_step`
    /// detects a boundary exit or a body collision.
    pub fn enter_game_over(&mut self) {
        self.status = GameStatus::GameOver;
    }

    /// Put the food on a fresh free cell. If the board is full, the current food
    /// stays put: board-full is not a loss condition, so nothing here ends the game.
    fn respawn_food(&mut self) {
        let occupied = self.snake.segments().to_vec();
        if let Some(new_food_position) = food_placement::choose_food_position(&occupied) {
            self.food = Food::new(new_food_position);
        }
    }
}
