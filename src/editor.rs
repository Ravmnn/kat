use std::io::{self, stdout, Read};

use crossterm::event::KeyModifiers;
use crossterm::{
    cursor::MoveTo,
    event::{KeyCode, KeyEvent},
    queue,
    style::Print,
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub pos: Point,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn new() -> Self {
        Self::from(Point::new(), 0, 0)
    }

    pub fn from(pos: Point, width: usize, height: usize) -> Self {
        Rectangle { pos, width, height }
    }
}

// TODO: add CTRL modifier behavior

pub struct Editor {
    cursor: Point,
    lines: Vec<String>,
    viewport: Rectangle,
    key_modifiers: KeyModifiers,
    should_exit: bool,
}

impl Editor {
    const LINE_NUMBER_AREA_LENGTH_IN_CELLS: u16 = 7;
    const LINE_NUMBER_EXTRA_SPACES: u16 = 3;

    const CURSOR_COLUMN_START_OFFSET: u16 =
        Self::LINE_NUMBER_AREA_LENGTH_IN_CELLS + Self::LINE_NUMBER_EXTRA_SPACES;

    const VIEWPORT_BOUND_MARGIN: u16 = 2;

    const MINIMUM_TERMINAL_WIDTH: u16 = Self::CURSOR_COLUMN_START_OFFSET + 5;
    const MINIMUM_TERMINAL_HEIGHT: u16 = Self::VIEWPORT_BOUND_MARGIN * 2 + 1;

    const TERMINAL_TOO_SMALL_MESSAGE: &'static str = "Terminal too small!!";

    pub fn is_terminal_size_too_small() -> io::Result<bool> {
        let (current_terminal_width, current_terminal_height) = terminal::size()?;

        Ok(current_terminal_width < Self::MINIMUM_TERMINAL_WIDTH
            || current_terminal_height < Self::MINIMUM_TERMINAL_HEIGHT)
    }

    pub fn new() -> Editor {
        let mut editor = Editor {
            cursor: Point::new(),
            lines: vec![String::new()],
            viewport: Rectangle::new(),
            key_modifiers: KeyModifiers::empty(),
            should_exit: false,
        };

        let mut buffer = String::new();

        let _ = std::fs::File::open("./test.txt")
            .unwrap()
            .read_to_string(&mut buffer);

        editor.lines = buffer.split("\n").map(String::from).collect();

        editor
    }

    pub fn cursor(&self) -> &Point {
        &self.cursor
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    pub fn viewport(&self) -> &Rectangle {
        &self.viewport
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.key_modifiers.contains(KeyModifiers::CONTROL)
    }

    fn are_characters_at_cursor_word(&self, left_char: bool, right_char: bool) -> bool {
        let (ch_left, ch_right) = self.get_characters_at_cursor();
        let left_valid = if left_char {
            Self::is_character_word(ch_left.unwrap_or('\0'))
        } else {
            true
        };
        let right_valid = if right_char {
            Self::is_character_word(ch_right.unwrap_or('\0'))
        } else {
            true
        };

        left_valid && right_valid
    }

    fn are_both_characters_at_cursor_word(&self) -> bool {
        self.are_characters_at_cursor_word(true, true)
    }

    fn is_left_character_at_cursor_word(&self) -> bool {
        self.are_characters_at_cursor_word(true, false)
    }

    fn is_right_character_at_cursor_word(&self) -> bool {
        self.are_characters_at_cursor_word(false, true)
    }

    fn should_repeat(&self, left: bool, right: bool) -> bool {
        self.is_ctrl_pressed() && self.are_characters_at_cursor_word(left, right)
    }

    fn is_character_word(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    pub fn get_line_at(&self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }

    pub fn get_line_at_mut(&mut self, index: usize) -> Option<&mut String> {
        self.lines.get_mut(index)
    }

    pub fn get_line_at_cursor(&self) -> &String {
        self.get_line_at(self.cursor.row as usize).unwrap()
    }

    pub fn get_line_at_cursor_mut(&mut self) -> &mut String {
        self.get_line_at_mut(self.cursor.row as usize).unwrap()
    }

    pub fn get_character_at(&self, row: usize, col: usize) -> Option<char> {
        self.get_line_at(row)?.chars().nth(col)
    }

    pub fn get_character_at_cursor_line(&self, col: usize) -> Option<char> {
        self.get_line_at_cursor().chars().nth(col)
    }

    pub fn get_characters_at_cursor(&self) -> (Option<char>, Option<char>) {
        (
            if self.cursor.col <= 0 {
                None
            } else {
                self.get_character_at_cursor_line(self.cursor.col as usize - 1)
            },
            self.get_character_at_cursor_line(self.cursor.col as usize),
        )
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
        true
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.cursor.row >= self.max_rows() as isize {
            return false;
        }

        self.cursor.row += 1;
        true
    }

    pub fn move_cursor_forward_once(&mut self) -> bool {
        if self.cursor.col < self.max_cols() as isize {
            self.cursor.col += 1;
            return true;
        } else if self.move_cursor_down() {
            self.cursor.col = 0;
            return true;
        }

        false
    }

    pub fn move_cursor_forward(&mut self) -> bool {
        if self.move_cursor_forward_once() && self.should_repeat(true, true) {
            self.move_cursor_forward();
            return true;
        }

        false
    }

    pub fn move_cursor_backward_once(&mut self) -> bool {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            return true;
        } else if self.move_cursor_up() {
            self.cursor.col = self.max_cols() as isize;
            return true;
        }

        false
    }

    pub fn move_cursor_backward(&mut self) -> bool {
        if self.move_cursor_backward_once() && self.should_repeat(true, true) {
            self.move_cursor_backward();
            return true;
        }

        false
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

    pub fn get_viewport_cursor_position(&self) -> Point {
        Point::from(
            self.cursor.row - self.viewport.pos.row,
            self.cursor.col - self.viewport.pos.col + Self::CURSOR_COLUMN_START_OFFSET as isize,
        )
    }

    pub fn move_viewport_to_up(&mut self) -> bool {
        if self.viewport.pos.row < 1 {
            return false;
        }

        self.viewport.pos.row -= 1;
        true
    }

    pub fn move_viewport_to_down(&mut self) -> bool {
        if self.viewport.pos.row >= self.max_rows() as isize {
            return false;
        }

        self.viewport.pos.row += 1;
        true
    }

    pub fn move_viewport_to_left(&mut self) -> bool {
        if self.viewport.pos.col < 1 {
            return false;
        }

        self.viewport.pos.col -= 1;
        true
    }

    pub fn move_viewport_to_right(&mut self) -> bool {
        if self.viewport.pos.col + self.viewport.width as isize
            >= self.max_cols() as isize + Self::CURSOR_COLUMN_START_OFFSET as isize + 1
        {
            return false;
        }

        self.viewport.pos.col += 1;
        true
    }

    fn update_viewport_position(&mut self) {
        let viewport_cursor_position = self.get_viewport_cursor_position();

        let mut should_recurse = false;
        let margin = Self::VIEWPORT_BOUND_MARGIN as isize;
        let column_offset = Self::CURSOR_COLUMN_START_OFFSET as isize;

        if viewport_cursor_position.row >= self.viewport.height as isize - margin {
            should_recurse = self.move_viewport_to_down();
        } else if viewport_cursor_position.row < margin {
            should_recurse = self.move_viewport_to_up();
        }

        if viewport_cursor_position.col > self.viewport.width as isize - margin {
            should_recurse = self.move_viewport_to_right();
        } else if viewport_cursor_position.col < margin + column_offset {
            should_recurse = self.move_viewport_to_left();
        }

        if should_recurse {
            self.update_viewport_position();
        }
    }

    fn update_viewport_size(&mut self) {
        let (terminal_width, terminal_height) = terminal::size().unwrap();

        self.viewport.width = terminal_width as usize;
        self.viewport.height = terminal_height as usize;
    }

    pub fn process_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        if Self::is_terminal_size_too_small()? {
            return Ok(());
        }

        self.key_modifiers = key.modifiers;

        match key.code {
            KeyCode::Left => _ = self.move_cursor_backward(),
            KeyCode::Up => _ = self.move_cursor_up(),
            KeyCode::Right => _ = self.move_cursor_forward(),
            KeyCode::Down => _ = self.move_cursor_down(),

            KeyCode::Enter => self.enter(),
            KeyCode::Backspace => _ = self.backspace_once(),

            KeyCode::Char(ch) => {
                // CTRL + Backspace is interpreted wrong:
                // https://github.com/crossterm-rs/crossterm/issues/504

                if ch == 'h' && self.is_ctrl_pressed() {
                    self.backspace();
                } else {
                    self.insert_char(ch);
                }
            }

            KeyCode::Esc => self.should_exit = true,

            _ => {}
        };

        Ok(())
    }

    pub fn update(&mut self) -> io::Result<()> {
        if Self::is_terminal_size_too_small()? {
            return Ok(());
        }

        self.clamp_cursor_position();

        self.update_viewport_size();
        self.update_viewport_position();

        Ok(())
    }

    pub fn align_terminal_cursor_position(&self) -> io::Result<()> {
        let viewport_cursor_position = self.get_viewport_cursor_position();

        queue!(
            stdout(),
            MoveTo(
                viewport_cursor_position.col as u16,
                viewport_cursor_position.row as u16
            )
        )?;

        Ok(())
    }

    pub fn enter(&mut self) {
        let point = self.cursor;

        self.split_line_down(&point);

        self.move_cursor_down();
        self.move_cursor_to_start_of_line();
    }

    fn split_line_down(&mut self, point: &Point) {
        let newline = point.row as usize + 1;

        if newline > self.max_rows() {
            self.lines.push(String::new())
        }

        let split_line = self.get_line_at_cursor_mut().split_off(point.col as usize);

        self.lines.insert(newline, split_line);
    }

    pub fn backspace_once(&mut self) -> bool {
        if self.cursor.row == 0 && self.cursor.col == 0 {
            return false;
        }

        if self.cursor.col == 0 {
            self.move_cursor_backward_once();
            self.merge_below_line(self.cursor.row as usize);
            return false;
        }

        self.remove_character_at_cursor();
        self.move_cursor_backward_once();
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.backspace_once() && self.should_repeat(true, false) {
            self.backspace();
            return true;
        }

        false
    }

    fn remove_character_at_cursor(&mut self) {
        let cursor_col = self.cursor.col as usize;

        self.get_line_at_cursor_mut().remove(cursor_col - 1);
    }

    fn merge_below_line(&mut self, index: usize) {
        let line_buffer = self.lines.remove(index + 1);

        self.get_line_at_mut(index)
            .unwrap()
            .push_str(line_buffer.as_str());
    }

    pub fn insert_char(&mut self, char: char) {
        let col_index = self.cursor.col as usize;

        self.get_line_at_cursor_mut().insert(col_index, char);

        self.move_cursor_forward_once();
    }

    pub fn print(&self) -> io::Result<()> {
        Self::clear_all_screen()?;

        if Self::is_terminal_size_too_small()? {
            stdout().queue(Print(Self::TERMINAL_TOO_SMALL_MESSAGE))?;
        } else {
            stdout().queue(Print(self.get_formatted_lines()))?;
        }

        Ok(())
    }

    fn get_formatted_lines(&self) -> String {
        let range = self.viewport.pos.row..self.viewport.pos.row + self.viewport.height as isize;
        let mut lines = String::new();

        for (terminal_row, line_index) in (0..).zip(range) {
            if terminal_row + self.viewport.pos.row as usize >= self.max_rows() {
                break;
            }

            let line_start = self.viewport.pos.col as usize;
            let line_end =
                line_start + self.viewport.width - Self::CURSOR_COLUMN_START_OFFSET as usize;

            let line = self.format_line(line_index as usize, line_start, line_end);

            lines += format!("{}{}", MoveTo(0, terminal_row as u16), line).as_str();
        }

        lines
    }

    fn clear_all_screen() -> io::Result<()> {
        queue!(
            stdout(),
            MoveTo(0, 0),
            Clear(ClearType::All),
            Clear(ClearType::Purge),
        )
    }

    fn format_line(&self, index: usize, start: usize, mut end: usize) -> String {
        let line = &self.lines[index];
        let is_valid = start < line.len() && !line.is_empty();

        if is_valid {
            end = end.clamp(0, line.len());
        }

        let final_line = if is_valid { &line[start..end] } else { "" };

        format!(
            "{:>line_width$}{}{}",
            index + 1,
            " ".repeat(Self::LINE_NUMBER_EXTRA_SPACES as usize),
            final_line,
            line_width = Self::LINE_NUMBER_AREA_LENGTH_IN_CELLS as usize,
        )
    }
}
