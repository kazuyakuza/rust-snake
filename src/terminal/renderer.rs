//! Full-frame board renderer: ASCII borders, half-block-packed board rows,
//! and the score line, redrawn in place from an immutable `GameState`.
//!
//! The board packs two logical rows into each terminal row: terminal row
//! `packed_row` shows logical rows `2 * packed_row` (upper half) and
//! `2 * packed_row + 1` (lower half) through half-block glyphs, so the
//! 80-row board renders as `HEIGHT / 2` packed rows. `HEIGHT` is even, so
//! every logical row lands in exactly one half of one packed row.

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Colors, Print, ResetColor, SetColors},
};

use crate::game::position::Position;
use crate::game::state::{GameState, HEIGHT, WIDTH};

const CORNER_GLYPH: char = '+';
const HORIZONTAL_GLYPH: char = '-';
const VERTICAL_GLYPH: char = '|';
const SCORE_PREFIX: &str = "Score: ";
const LINE_BREAK: &str = "\r\n";

/// Full-frame renderer that writes the board frame and score line to an owned output.
pub struct Renderer<W: Write> {
    output: W,
}

impl<W: Write> Renderer<W> {
    /// Create a renderer that writes its frames to `output`.
    pub fn new(output: W) -> Renderer<W> {
        Renderer { output }
    }

    /// Draw the complete frame for `state` at the home position, without scrolling.
    pub fn render(&mut self, state: &GameState) -> io::Result<()> {
        queue!(self.output, MoveTo(0, 0))?;
        self.write_border_row()?;
        self.write_packed_rows(state)?;
        self.write_border_row()?;
        self.write_score_line(state)?;
        queue!(self.output, ResetColor)?;
        self.output.flush()
    }

    fn write_border_row(&mut self) -> io::Result<()> {
        queue!(self.output, ResetColor)?;
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }

    fn write_packed_rows(&mut self, state: &GameState) -> io::Result<()> {
        for packed_row in 0..(HEIGHT / 2) {
            self.write_packed_row(state, packed_row)?;
        }
        Ok(())
    }

    fn write_packed_row(&mut self, state: &GameState, packed_row: i32) -> io::Result<()> {
        let row_cells = PackedRowCells::for_packed_row(packed_row);
        queue!(self.output, Print(VERTICAL_GLYPH))?;
        for column in 0..WIDTH {
            self.write_packed_cell(state, row_cells.cell_at(column))?;
        }
        queue!(self.output, ResetColor)?;
        queue!(self.output, Print(VERTICAL_GLYPH))?;
        queue!(self.output, Print(LINE_BREAK))?;
        Ok(())
    }

    fn write_packed_cell(&mut self, state: &GameState, cell: PackedCell) -> io::Result<()> {
        let (glyph, foreground, background) =
            half_block_glyph(cell_kind_at(state, cell.top), cell_kind_at(state, cell.bottom));
        queue!(self.output, SetColors(Colors::new(foreground, background)))?;
        queue!(self.output, Print(glyph))?;
        Ok(())
    }

    fn write_score_line(&mut self, state: &GameState) -> io::Result<()> {
        queue!(self.output, ResetColor)?;
        let score_text = format!("{SCORE_PREFIX}{}", state.score());
        self.write_line(&score_text)
    }

    fn write_line(&mut self, content: &str) -> io::Result<()> {
        write!(self.output, "{content}{LINE_BREAK}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Empty,
    Head,
    Body,
    Food,
}

fn cell_kind_at(state: &GameState, position: Position) -> CellKind {
    if state.snake().head() == position {
        return CellKind::Head;
    }
    if state.snake().segments().contains(&position) {
        return CellKind::Body;
    }
    if state.food().occupies(position) {
        return CellKind::Food;
    }
    CellKind::Empty
}

fn cell_color(kind: CellKind) -> Color {
    match kind {
        CellKind::Head => Color::Yellow,
        CellKind::Body => Color::Green,
        CellKind::Food => Color::Red,
        CellKind::Empty => Color::Reset,
    }
}

fn half_block_glyph(top: CellKind, bottom: CellKind) -> (char, Color, Color) {
    match (top, bottom) {
        (CellKind::Empty, CellKind::Empty) => (' ', Color::Reset, Color::Reset),
        (top_kind, CellKind::Empty) => ('▀', cell_color(top_kind), Color::Reset),
        (CellKind::Empty, bottom_kind) => ('▄', cell_color(bottom_kind), Color::Reset),
        (kind_a, kind_b) if kind_a == kind_b => ('█', cell_color(kind_a), Color::Reset),
        (top_kind, bottom_kind) => ('█', cell_color(top_kind), cell_color(bottom_kind)),
    }
}

struct PackedRowCells {
    top_y: i32,
    bottom_y: i32,
}

impl PackedRowCells {
    fn for_packed_row(packed_row: i32) -> PackedRowCells {
        PackedRowCells { top_y: 2 * packed_row, bottom_y: 2 * packed_row + 1 }
    }

    fn cell_at(&self, column: i32) -> PackedCell {
        PackedCell {
            top: Position { x: column, y: self.top_y },
            bottom: Position { x: column, y: self.bottom_y },
        }
    }
}

struct PackedCell {
    top: Position,
    bottom: Position,
}

fn rendered_border_row() -> String {
    let mut border_row = String::new();
    border_row.push(CORNER_GLYPH);
    for _ in 0..WIDTH {
        border_row.push(HORIZONTAL_GLYPH);
    }
    border_row.push(CORNER_GLYPH);
    border_row
}
