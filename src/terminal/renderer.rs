//! Full-frame board renderer: ASCII borders, snake, food, and score line,
//! redrawn in place from an immutable `GameState`.

use std::io::{self, Write};

use crossterm::{cursor::MoveTo, queue};

use crate::game::position::Position;
use crate::game::state::{GameState, HEIGHT, WIDTH};

const EMPTY_SPAN: &'static str = "  ";
const BODY_SPAN: &'static str = "██";
const HEAD_SPAN: &'static str = "●●";
const FOOD_SPAN: &'static str = "◆◆";
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
        self.write_board_rows(state)?;
        self.write_border_row()?;
        self.write_score_line(state)?;
        self.output.flush()
    }

    fn write_border_row(&mut self) -> io::Result<()> {
        let border_text = rendered_border_row();
        self.write_line(&border_text)
    }

    fn write_board_rows(&mut self, state: &GameState) -> io::Result<()> {
        for row in 0..HEIGHT {
            let row_text = self.render_board_row(state, row);
            self.write_line(&row_text)?;
        }
        Ok(())
    }

    fn write_score_line(&mut self, state: &GameState) -> io::Result<()> {
        let score_text = format!("{SCORE_PREFIX}{}", state.score());
        self.write_line(&score_text)
    }

    fn write_line(&mut self, content: &str) -> io::Result<()> {
        write!(self.output, "{content}{LINE_BREAK}")
    }

    fn render_board_row(&self, state: &GameState, row: i32) -> String {
        let mut row_text = String::new();
        row_text.push(VERTICAL_GLYPH);
        for column in 0..WIDTH {
            row_text.push_str(self.cell_span(state, Position { x: column, y: row }));
        }
        row_text.push(VERTICAL_GLYPH);
        row_text
    }

    fn cell_span(&self, state: &GameState, cell: Position) -> &'static str {
        if self.is_snake_head(state, cell) {
            return HEAD_SPAN;
        }
        if self.is_snake_body(state, cell) {
            return BODY_SPAN;
        }
        if self.is_food(state, cell) {
            return FOOD_SPAN;
        }
        EMPTY_SPAN
    }

    fn is_snake_head(&self, state: &GameState, cell: Position) -> bool {
        state.snake().head() == cell
    }

    fn is_snake_body(&self, state: &GameState, cell: Position) -> bool {
        state.snake().segments().contains(&cell)
    }

    fn is_food(&self, state: &GameState, cell: Position) -> bool {
        state.food().occupies(cell)
    }
}

fn rendered_border_row() -> String {
    let mut border_row = String::new();
    border_row.push(CORNER_GLYPH);
    for _ in 0..(WIDTH * 2) {
        border_row.push(HORIZONTAL_GLYPH);
    }
    border_row.push(CORNER_GLYPH);
    border_row
}
