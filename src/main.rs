mod app;
mod editor;

use std::{
    io::{self, Error, Write},
    time::Duration,
};

use crossterm::event::{poll, read, Event::Key};

use app::{deinit, init};
use editor::Editor;

fn main() -> Result<(), Error> {
    init().expect("Couldn't init kat");

    let mut editor = Editor::new();

    while !editor.should_exit() {
        if poll(Duration::from_millis(10))? {
            if let Key(key) = read()? {
                editor.process_key_event(key)
            };
        }

        editor.update();
        editor.print().expect("Couldn't print editor");
        editor.align_terminal_cursor_position()?;

        io::stdout().flush()?;
    }

    deinit().expect("Couldn't close kat correctly");

    Ok(())
}
