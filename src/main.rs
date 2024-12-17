mod editor;
mod input;

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{self, DisableLineWrap};

use editor::Editor;
use input::read_character_from_stdin;

fn main() {
    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), DisableLineWrap).unwrap();

    let mut editor = Editor::new();

    loop {
        match read_character_from_stdin() {
            Some(character) => editor.process_character(character),
            _ => break,
        }

        editor.print().expect("Couldn't print editor");

        let (row, col) = editor.cursor_position_to_screen();

        // print!("{:?}", screen_cursor);

        execute!(io::stdout(), MoveTo(col as u16, row as u16)).unwrap();

        io::stdout().flush().unwrap();
    }

    terminal::disable_raw_mode().unwrap();
}
