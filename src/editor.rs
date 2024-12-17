use std::io::stdout;
use std::io::{self, Read};

use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor::MoveTo, queue, style::Print};

use crate::input::Character;

#[derive(Debug)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl Point {
    pub fn new() -> Self {
        Self::from(0, 0)
    }

    pub fn from(row: usize, col: usize) -> Self {
        Point { row, col }
    }
}

pub struct Editor {
    cursor: Point,
    lines: Vec<String>,
    viewport_offset: Point,
}

impl Editor {
    pub const LINE_NUMBER_ALIGNMENT: u8 = 5; // line number (max of 5 characters)
    pub const COLUMN_START: u8 = Self::LINE_NUMBER_ALIGNMENT + 2; // line number alignment + "  " (2)

    pub fn new() -> Editor {
        let mut editor = Editor {
            lines: vec![String::new()],
            cursor: Point::new(),
            viewport_offset: Point::new(),
        };

        let mut buffer = String::new();

        std::fs::File::open("./test.txt")
            .unwrap()
            .read_to_string(&mut buffer);

        editor.lines = buffer.split("\n").map(String::from).collect();

        return editor;
    }

    pub fn viewport_offset(&self) -> &Point {
        &self.viewport_offset
    }

    pub fn viewport_size(&self) -> Point {
        let (terminal_width, terminal_height) = terminal::size().unwrap();

        return Point::from(terminal_height as usize, terminal_width as usize);
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

    pub fn cursor_position_to_screen(&self) -> Point {
        Point {
            col: self.cursor.col + Self::COLUMN_START as usize,
            row: self.cursor.row - self.viewport_offset.row,
        }
    }

    pub fn move_viewport_to_up(&mut self) {
        if self.viewport_offset.row >= 1 {
            self.viewport_offset.row -= 1;
        }
    }

    pub fn move_viewport_to_down(&mut self) {
        if self.viewport_offset.row < self.max_rows() {
            self.viewport_offset.row += 1;
        }
    }

    fn update_viewport_offset(&mut self) {
        let screen_cursor = self.cursor_position_to_screen();
        let viewport_size = self.viewport_size();

        if screen_cursor.row >= viewport_size.row - 2 {
            self.move_viewport_to_down();
        }

        if screen_cursor.row <= 1 {
            self.move_viewport_to_up();
        }
    }

    pub fn process_character(&mut self, character: Character) {
        match character {
            Character::Normal(ch) => self.insert_byte(ch),
            _ => self.process_special_character(character),
        }

        self.update_viewport_offset();
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
        let point = Point {
            row: self.cursor.row as usize + 1,
            ..self.cursor
        };

        self.split_line_down(&point);

        self.move_cursor_down();
        self.move_cursor_to_start_of_line();
    }

    fn split_line_down(&mut self, point: &Point) {
        if point.row > self.max_rows() {
            self.lines.push(String::new())
        }

        let splitted_line = self.get_line_at_cursor_mut().unwrap().split_off(point.col);

        self.lines.insert(point.row, splitted_line);
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

        let viewport_offset = self.viewport_offset();
        let viewport_size = self.viewport_size();

        let mut terminal_line: u16 = 0;

        for line_index in viewport_offset.row..viewport_offset.row + viewport_size.row {
            if terminal_line as usize + viewport_offset.row >= self.max_rows() {
                break;
            }

            queue!(
                io::stdout(),
                MoveTo(0, terminal_line),
                Print(self.line_to_string(line_index))
            )
            .unwrap();

            terminal_line += 1;
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
