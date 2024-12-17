use std::io::stdout;
use std::io::{self, Read};

use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor::MoveTo, queue, style::Print};

use crate::input::Character;

#[derive(Debug)]
pub struct Point {
    pub row: isize,
    pub col: isize,
}

impl Point {
    pub fn new() -> Self {
        Self::from(0, 0)
    }

    pub fn from(row: isize, col: isize) -> Self {
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

        return Point::from(terminal_height as isize, terminal_width as isize);
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
        if self.cursor.row >= self.max_rows() as isize {
            return false;
        }

        self.cursor.row += 1;
        return true;
    }

    pub fn move_cursor_forward(&mut self) -> bool {
        if self.cursor.col < self.max_cols() as isize {
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
            self.cursor.col = self.max_cols() as isize; // reset to what?
            return true;
        }

        return false;
    }

    pub fn move_cursor_to_start_of_line(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_cursor_to_end_of_line(&mut self) {
        self.cursor.col = self.max_cols() as isize;
    }

    fn clamp_cursor_position(&mut self) {
        self.cursor.row = self.cursor.row.clamp(0, self.max_rows() as isize - 1);
        self.cursor.col = self.cursor.col.clamp(0, self.max_cols() as isize);
    }

    pub fn cursor_position_to_screen(&self) -> (isize, isize) {
        (
            self.cursor.row - self.viewport_offset.row,
            self.cursor.col - self.viewport_offset.col + Self::COLUMN_START as isize,
        )
    }

    pub fn is_cursor_x_inside_viewport(&self) -> bool {
        let x = self.cursor_position_to_screen().1;
        return x >= self.viewport_offset.col && x <= self.viewport_size().col;
    }

    pub fn is_cursor_y_inside_viewport(&self) -> bool {
        let y = self.cursor_position_to_screen().0;
        return y >= self.viewport_offset.row && y <= self.viewport_size().row;
    }

    pub fn is_cursor_inside_viewport(&self) -> bool {
        self.is_cursor_x_inside_viewport() && self.is_cursor_y_inside_viewport()
    }

    pub fn move_viewport_to_up(&mut self) -> bool {
        if self.viewport_offset.row < 1 {
            return false;
        }

        self.viewport_offset.row -= 1;
        return true;
    }

    pub fn move_viewport_to_down(&mut self) -> bool {
        if self.viewport_offset.row >= self.max_rows() as isize {
            return false;
        }

        self.viewport_offset.row += 1;
        return true;
    }

    pub fn move_viewport_to_left(&mut self) -> bool {
        if self.viewport_offset.col < 1 {
            return false;
        }

        self.viewport_offset.col -= 1;
        return true;
    }

    pub fn move_viewport_to_right(&mut self) -> bool {
        if self.viewport_offset.col + self.viewport_size().col
            >= self.max_cols() as isize + Self::COLUMN_START as isize + 1
        {
            return false;
        }

        self.viewport_offset.col += 1;
        return true;
    }

    fn update_viewport_offset(&mut self) {
        let (row, col) = self.cursor_position_to_screen();
        let viewport_size = self.viewport_size();

        let mut should_recurse = false;

        if row >= viewport_size.row - 2 {
            should_recurse = self.move_viewport_to_down();
        } else if row <= 1 {
            should_recurse = self.move_viewport_to_up();
        }

        if col >= viewport_size.col - 2 {
            should_recurse = self.move_viewport_to_right();
        } else if col <= 1 + Self::COLUMN_START as isize {
            should_recurse = self.move_viewport_to_left();
        }

        if should_recurse {
            self.update_viewport_offset();
        }
    }

    pub fn process_character(&mut self, character: Character) {
        match character {
            Character::Normal(ch) => self.insert_byte(ch),
            _ => self.process_special_character(character),
        }

        self.clamp_cursor_position();
        self.update_viewport_offset();
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
            row: self.cursor.row + 1,
            ..self.cursor
        };

        self.split_line_down(&point);

        self.move_cursor_down();
        self.move_cursor_to_start_of_line();
    }

    fn split_line_down(&mut self, point: &Point) {
        if point.row > self.max_rows() as isize {
            self.lines.push(String::new())
        }

        let splitted_line = self
            .get_line_at_cursor_mut()
            .unwrap()
            .split_off(point.col as usize);

        self.lines.insert(point.row as usize, splitted_line);
    }

    pub fn backspace(&mut self) {
        if self.cursor.row == 0 && self.cursor.col == 0 {
            return;
        }

        if self.cursor.col == 0 {
            self.move_cursor_backward();
            self.merge_below_line(self.cursor.row as usize);
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
            if terminal_line as usize + viewport_offset.row as usize >= self.max_rows() {
                break;
            }

            let line_start = viewport_offset.col as usize;
            let line_end = line_start + viewport_size.col as usize - Self::COLUMN_START as usize;

            queue!(
                io::stdout(),
                MoveTo(0, terminal_line),
                Print(self.line_to_string(line_index as usize, line_start, line_end))
            )
            .unwrap();

            terminal_line += 1;
        }

        return Result::Ok(());
    }

    fn line_to_string(&self, index: usize, start: usize, mut end: usize) -> String {
        let line = &self.lines[index];

        if end >= line.len() {
            end = line.len() - 1;
        }

        format!(
            "{:>line_width$}  {}",
            index + 1,
            if start > line.len() {
                ""
            } else {
                &line[start..=end]
            },
            line_width = Self::LINE_NUMBER_ALIGNMENT as usize,
        )
    }
}
