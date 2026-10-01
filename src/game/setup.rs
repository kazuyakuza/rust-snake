use crate::game::direction::Direction;
use crate::game::food::Food;
use crate::game::position::Position;
use crate::game::snake::Snake;

// Head-to-tail start: `HEAD` leads and moves `Right`, `BODY_AHEAD` sits adjacent
// to it, and `BODY_BEHIND` is the tail tip. The initial food shares row 12 but
// stays clear of every snake cell and inside the playable area.
const INITIAL_SNAKE_HEAD: Position = Position { x: 10, y: 12 };
const INITIAL_SNAKE_BODY_AHEAD: Position = Position { x: 9, y: 12 };
const INITIAL_SNAKE_BODY_BEHIND: Position = Position { x: 8, y: 12 };
const INITIAL_DIRECTION: Direction = Direction::Right;
const INITIAL_FOOD_POSITION: Position = Position { x: 20, y: 12 };

pub fn initial_setup() -> GameStateSetup {
    GameStateSetup {
        snake: Snake::new(Vec::from([INITIAL_SNAKE_HEAD, INITIAL_SNAKE_BODY_AHEAD, INITIAL_SNAKE_BODY_BEHIND])),
        food: Food::new(INITIAL_FOOD_POSITION),
        direction: INITIAL_DIRECTION,
    }
}

pub struct GameStateSetup {
    pub snake: Snake,
    pub food: Food,
    pub direction: Direction,
}
