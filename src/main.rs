//! Binary entry point: wires initial setup, terminal lifecycle, the start and
//! game-over screens, the playing loop, and exit cleanup into one flow.

use std::io::{self, stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
};
use snake::game::setup::initial_setup;
use snake::game::state::GameState;
use snake::terminal::game_loop::run_playing_loop;
use snake::terminal::lifecycle::TerminalHandle;
use snake::terminal::renderer::Renderer;

const LINE_BREAK: &str = "\r\n";
const START_MESSAGE: &str = "Press any key to start";
const GAME_OVER_TITLE: &str = "GAME OVER";
const EXIT_PROMPT: &str = "Press any key to exit";
const SCORE_PREFIX: &str = "Score: ";

/// Demonstrate the complete gameplay flow: start-screen wait, playing loop,
/// game-over screen, exit wait.
fn main() -> io::Result<()> {
    let mut state = GameState::new(initial_setup());
    let mut terminal = TerminalHandle::enable(stdout())?;

    show_start_screen(terminal.output())?;
    wait_for_any_key_press()?;
    state.start_playing();

    let mut renderer = Renderer::new(terminal.output());
    run_playing_loop(&mut state, &mut renderer)?;

    show_game_over_screen(terminal.output(), state.score())?;
    wait_for_any_key_press()?;

    Ok(())
}

/// Show the start prompt and block until any key is pressed, then begin play.
fn show_start_screen(output: &mut impl Write) -> io::Result<()> {
    queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;
    write_line(output, START_MESSAGE)?;
    output.flush()
}

/// Block until the next key press (any key); ignore releases, repeats, and
/// non-key events such as resize.
fn wait_for_any_key_press() -> io::Result<()> {
    loop {
        let event = event::read()?;
        if is_any_key_press(event) {
            return Ok(());
        }
    }
}

/// Show the final score and block until any key is pressed, then exit.
fn show_game_over_screen(output: &mut impl Write, score: i32) -> io::Result<()> {
    queue!(output, Clear(ClearType::All), MoveTo(0, 0))?;
    write_line(output, GAME_OVER_TITLE)?;
    write_line(output, "")?;
    write_line(output, &score_line(score))?;
    write_line(output, "")?;
    write_line(output, EXIT_PROMPT)?;
    output.flush()
}

fn write_line(output: &mut impl Write, content: &str) -> io::Result<()> {
    write!(output, "{content}{LINE_BREAK}")
}

fn score_line(score: i32) -> String {
    format!("{SCORE_PREFIX}{score}")
}

fn is_any_key_press(event: Event) -> bool {
    match event {
        Event::Key(key_event) => key_event.kind == KeyEventKind::Press,
        _ => false,
    }
}
