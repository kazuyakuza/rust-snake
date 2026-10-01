//! An ordered chain of positions: `segments[0]` is the head, followed by body
//! segments toward the tail. Movement and growth (`advance`) preserve this
//! head-first ordering.

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

    /// Insert `next_head` at the front; drop the last segment when
    /// `should_remove_tail`, which preserves the length, or retain it so the
    /// snake grows by exactly one segment.
    pub fn advance(&mut self, next_head: Position, should_remove_tail: bool) {
        self.segments.insert(0, next_head);
        if should_remove_tail {
            self.segments.pop();
        }
    }
}
