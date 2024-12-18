mod app;
mod editor;

use std::{
    io::{self, Error, Write},
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event::Key},
    queue,
};

use app::{deinit, init};
use editor::Editor;

fn main() -> Result<(), Error> {
    init().expect("Couldn't init kat");

    let mut editor = Editor::new();

    while !editor.should_exit() {
        if poll(Duration::from_millis(50))? {
            match read()? {
                Key(key) => editor.process_key_event(key),
                _ => {}
            };
        }

        editor.print().expect("Couldn't print editor");

        let viewport_cursor_position = editor.get_viewport_cursor_position();

        queue!(
            io::stdout(),
            MoveTo(
                viewport_cursor_position.col as u16,
                viewport_cursor_position.row as u16
            )
        )?;

        io::stdout().flush()?;
    }

    deinit().expect("Couldn't close kat correctly");

    Ok(())
}
