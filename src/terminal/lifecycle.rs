//! Terminal lifecycle guard: raw mode, alternate screen, and cursor
//! visibility, restored on explicit disable or on drop.

use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

/// Guard that owns the terminal output stream and restores the terminal state:
/// raw mode off, main screen buffer back, cursor visible.
pub struct TerminalHandle<W: Write> {
    output: W,
}

impl<W: Write> TerminalHandle<W> {
    /// Enable raw mode, switch to the alternate screen, and hide the cursor.
    pub fn enable(mut output: W) -> io::Result<TerminalHandle<W>> {
        terminal::enable_raw_mode()?;
        if let Err(setup_error) = write_setup_commands(&mut output) {
            let _ = terminal::disable_raw_mode();
            return Err(setup_error);
        }
        Ok(TerminalHandle { output })
    }

    /// Restore the terminal: cursor visible, main screen buffer, raw mode off.
    pub fn disable(&mut self) -> io::Result<()> {
        let restore_result = self.write_restore_commands();
        let raw_mode_result = terminal::disable_raw_mode();
        report_first_error(restore_result, raw_mode_result)
    }

    /// Borrow the wrapped output for screen writing.
    pub fn output(&mut self) -> &mut W {
        &mut self.output
    }

    fn write_restore_commands(&mut self) -> io::Result<()> {
        queue!(self.output, cursor::Show, LeaveAlternateScreen)?;
        self.output.flush()
    }
}

impl<W: Write> Drop for TerminalHandle<W> {
    fn drop(&mut self) {
        let _ = self.disable();
    }
}

fn write_setup_commands(output: &mut impl Write) -> io::Result<()> {
    queue!(output, EnterAlternateScreen, cursor::Hide)?;
    output.flush()
}

fn report_first_error(
    restore_result: io::Result<()>,
    raw_mode_result: io::Result<()>,
) -> io::Result<()> {
    match restore_result {
        Err(error) => Err(error),
        Ok(()) => raw_mode_result,
    }
}
