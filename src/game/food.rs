use crate::game::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Food {
    position: Position,
}

impl Food {
    pub fn new(position: Position) -> Food {
        Food { position }
    }

    pub fn position(&self) -> Position {
        self.position
    }
}
