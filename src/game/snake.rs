use crate::game::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snake {
    segments: Vec<Position>,
}

impl Snake {
    pub fn new(segments: Vec<Position>) -> Snake {
        debug_assert!(!segments.is_empty(), "snake must have at least one segment");
        Snake { segments }
    }

    pub fn head(&self) -> Position {
        self.segments[0]
    }

    pub fn segments(&self) -> &[Position] {
        &self.segments
    }

    pub fn length(&self) -> usize {
        self.segments.len()
    }
}
