//! Arrow-key input: pure key-to-direction mapping and non-blocking drain of
//! pending terminal events; reversal rejection stays in `GameState`.

use std::io::{self};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::game::direction::Direction;

/// Returns the game direction of an arrow-key press, or `None` for any
/// other key, release, or repeat event.
pub fn map_key_event_to_direction(event: &KeyEvent) -> Option<Direction> {
    if !is_key_press(event) {
        return None;
    }
    match event.code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

fn is_key_press(event: &KeyEvent) -> bool {
    event.kind == KeyEventKind::Press
}

/// Drain all currently available input events without waiting, and return the
/// last arrow-key press seen, if any. Never blocks: when no event is pending
/// the zero-duration poll ends the loop immediately.
pub fn drain_arrow_event() -> io::Result<Option<KeyEvent>> {
    let mut last_arrow_press = None;
    while event::poll(Duration::ZERO)? {
        if let Some(arrow_press) = pressed_arrow_key(event::read()?) {
            last_arrow_press = Some(arrow_press);
        }
    }
    Ok(last_arrow_press)
}

fn pressed_arrow_key(event: Event) -> Option<KeyEvent> {
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    if !is_arrow_key_press(&key_event) {
        return None;
    }
    Some(key_event)
}

fn is_arrow_key_press(event: &KeyEvent) -> bool {
    is_key_press(event) && is_arrow_key(event)
}

fn is_arrow_key(event: &KeyEvent) -> bool {
    map_key_event_to_direction(event).is_some()
}

/// Drain all currently available input events without waiting, and return the
/// arrow-key directions seen, in chronological order. Never blocks.
pub fn drain_arrow_directions() -> io::Result<Vec<Direction>> {
    let mut directions = Vec::new();
    while event::poll(Duration::ZERO)? {
        if let Some(direction) = read_arrow_direction() {
            directions.push(direction);
        }
    }
    Ok(directions)
}

fn read_arrow_direction() -> Option<Direction> {
    let event = event::read().ok()?;
    let key_event = match event {
        Event::Key(key_event) => key_event,
        _ => return None,
    };
    map_key_event_to_direction(&key_event)
}
