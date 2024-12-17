mod editor;
mod input;

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::{execute, terminal};

use editor::Editor;
use input::read_character_from_stdin;

fn main() {
    terminal::enable_raw_mode().unwrap();

    let mut editor = Editor::new();

    loop {
        match read_character_from_stdin() {
            Some(character) => editor.process_character(character),
            _ => break,
        }

        editor.print().expect("Couldn't print editor");

        let screen_cursor = editor.cursor_position_to_screen();

        // print!("{:?}", screen_cursor);

        execute!(
            io::stdout(),
            MoveTo(screen_cursor.col as u16, screen_cursor.row as u16)
        )
        .unwrap();

        io::stdout().flush().unwrap();
    }

    terminal::disable_raw_mode().unwrap();
}
