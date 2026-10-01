//! Death predicates for one movement step: boundary exit and body overlap.
//! Both are pure and terminal-free; `state.rs` applies them to a next head.

use crate::game::position::Position;
use crate::game::state::is_inside_board;

const TAIL_SEGMENT_COUNT: usize = 1;

pub fn is_outside_board(next_head: Position) -> bool {
    !is_inside_board(next_head)
}

/// `true` when `next_head` lands on any segment except the current tail: the
/// tail vacates its cell in the same step, and the only step where it would
/// not (food growth) cannot enter it, since food never overlaps the snake.
pub fn collides_with_body(next_head: Position, segments: &[Position]) -> bool {
    head_to_body_slice(segments).iter().any(|segment| *segment == next_head)
}

fn head_to_body_slice(segments: &[Position]) -> &[Position] {
    if segments.is_empty() {
        return segments;
    }
    &segments[..segments.len() - TAIL_SEGMENT_COUNT]
}
