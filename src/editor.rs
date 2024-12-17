use std::io;
use std::io::stdout;

use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor::MoveTo, queue, style::Print};

use crate::input::Character;

#[derive(Debug)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

pub struct Editor {
    lines: Vec<String>,
    cursor: Cursor,
}

impl Editor {
    pub const LINE_NUMBER_ALIGNMENT: u8 = 5; // line number (max of 5 characters)
    pub const COLUMN_START: u8 = Self::LINE_NUMBER_ALIGNMENT + 2; // line number alignment + "  " (2)

    pub fn new() -> Editor {
        Editor {
            lines: vec![String::new()],
            cursor: Cursor { row: 0, col: 0 },
        }
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    pub fn get_line_at(&self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }

    pub fn get_line_at_mut(&mut self, index: usize) -> Option<&mut String> {
        self.lines.get_mut(index)
    }

    pub fn get_line_at_cursor(&self) -> Option<&String> {
        self.get_line_at(self.cursor.row as usize)
    }

    pub fn get_line_at_cursor_mut(&mut self) -> Option<&mut String> {
        self.get_line_at_mut(self.cursor.row as usize)
    }

    pub fn max_rows(&self) -> usize {
        self.lines.len()
    }

    pub fn max_cols(&self) -> usize {
        self.get_line_at(self.cursor.row as usize).unwrap().len()
    }

    pub fn move_cursor_up(&mut self) -> bool {
        if self.cursor.row <= 0 {
            return false;
        }

        self.cursor.row -= 1;
        return true;
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.cursor.row >= self.max_rows() {
            return false;
        }

        self.cursor.row += 1;
        return true;
    }

    pub fn move_cursor_forward(&mut self) -> bool {
        if self.cursor.col < self.max_cols() {
            self.cursor.col += 1;
            return true;
        }

        if self.move_cursor_down() {
            self.cursor.col = 0;
            return true;
        }

        return false;
    }

    pub fn move_cursor_backward(&mut self) -> bool {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            return true;
        }

        if self.move_cursor_up() {
            self.cursor.col = self.max_cols(); // reset to what?
            return true;
        }

        return false;
    }

    pub fn move_cursor_to_start_of_line(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_cursor_to_end_of_line(&mut self) {
        self.cursor.col = self.max_cols();
    }

    fn clamp_cursor_position(&mut self) {
        self.cursor.row = self.cursor.row.clamp(0, self.max_rows() - 1);
        self.cursor.col = self.cursor.col.clamp(0, self.max_cols());
    }

    pub fn cursor_position_to_screen(&self) -> Cursor {
        Cursor {
            col: self.cursor.col + Self::COLUMN_START as usize,
            ..self.cursor
        }
    }

    pub fn process_character(&mut self, character: Character) {
        match character {
            Character::Normal(ch) => self.insert_byte(ch),
            _ => self.process_special_character(character),
        }

        self.clamp_cursor_position();
    }

    fn process_special_character(&mut self, character: Character) {
        match character {
            Character::ArrowLeft => {
                self.move_cursor_backward();
            }
            Character::ArrowTop => {
                self.move_cursor_up();
            }
            Character::ArrowRight => {
                self.move_cursor_forward();
            }
            Character::ArrowBottom => {
                self.move_cursor_down();
            }

            Character::Enter => {
                self.enter();
            }

            Character::Backspace => {
                self.backspace();
            }

            _ => {}
        };
    }

    pub fn enter(&mut self) {
        let cursor = Cursor {
            row: self.cursor.row as usize + 1,
            ..self.cursor
        };

        self.split_line_down(&cursor);

        self.move_cursor_down();
        self.move_cursor_to_start_of_line();
    }

    fn split_line_down(&mut self, cursor: &Cursor) {
        if cursor.row > self.max_rows() {
            self.lines.push(String::new())
        }

        let splitted_line = self.get_line_at_cursor_mut().unwrap().split_off(cursor.col);

        self.lines.insert(cursor.row, splitted_line);
    }

    pub fn backspace(&mut self) {
        if self.cursor.row == 0 && self.cursor.col == 0 {
            return;
        }

        if self.cursor.col == 0 {
            self.move_cursor_backward();
            self.merge_below_line(self.cursor.row);
            return;
        }

        self.remove_character_at_cursor();
    }

    fn remove_character_at_cursor(&mut self) {
        let cursor_col = self.cursor.col as usize;

        self.get_line_at_cursor_mut()
            .unwrap()
            .remove(cursor_col - 1);

        self.move_cursor_backward();
    }

    fn merge_below_line(&mut self, index: usize) {
        let line_buffer = self.lines.remove(index + 1);

        self.get_line_at_mut(index)
            .unwrap()
            .push_str(line_buffer.as_str());
    }

    pub fn insert_byte(&mut self, byte: u8) {
        let col_index = self.cursor.col as usize;

        self.get_line_at_cursor_mut()
            .unwrap()
            .insert(col_index, byte as char);

        self.move_cursor_forward();
    }

    pub fn print(&self) -> io::Result<()> {
        queue!(
            stdout(),
            MoveTo(0, 0),
            Clear(ClearType::All),
            Clear(ClearType::Purge),
        )?;

        for (i, _) in self.lines.iter().enumerate() {
            queue!(
                io::stdout(),
                MoveTo(0, i as u16),
                Print(self.line_to_string(i))
            )
            .unwrap();
        }

        return Result::Ok(());
    }

    fn line_to_string(&self, index: usize) -> String {
        format!(
            "{:>line_width$}  {}",
            index + 1,
            self.lines[index],
            line_width = Self::LINE_NUMBER_ALIGNMENT as usize,
        )
    }
}
