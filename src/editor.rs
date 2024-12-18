use std::io::{self, stdout, Read};

use crossterm::{
    cursor::MoveTo,
    event::{KeyCode, KeyEvent},
    queue,
    style::Print,
    terminal::{self, Clear, ClearType},
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

pub struct Editor {
    cursor: Point,
    lines: Vec<String>,
    viewport: Rectangle,
    should_exit: bool,
}

impl Editor {
    const LINE_NUMBER_ALIGNMENT: u8 = 7; // line number (max of 7 characters)
    const CURSOR_COLUMN_START_OFFSET: u8 = Self::LINE_NUMBER_ALIGNMENT + 3; // line number alignment + "   " (3)

    const VIEWPORT_BOUND_MARGIN: u8 = 2;

    pub fn new() -> Editor {
        let mut editor = Editor {
            cursor: Point::new(),
            lines: vec![String::new()],
            viewport: Rectangle::new(),
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
        true
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.cursor.row >= self.max_rows() as isize {
            return false;
        }

        self.cursor.row += 1;
        true
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

        false
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

    pub fn process_key_event(&mut self, character: KeyEvent) {
        match character.code {
            KeyCode::Left => _ = self.move_cursor_backward(),
            KeyCode::Up => _ = self.move_cursor_up(),
            KeyCode::Right => _ = self.move_cursor_forward(),
            KeyCode::Down => _ = self.move_cursor_down(),

            KeyCode::Enter => self.enter(),
            KeyCode::Backspace => self.backspace(),

            KeyCode::Char(ch) => self.insert_byte(ch as u8),

            KeyCode::Esc => self.should_exit = true,

            _ => {}
        };
    }

    pub fn update(&mut self) {
        self.clamp_cursor_position();

        self.update_viewport_size();
        self.update_viewport_position();
    }

    pub fn align_terminal_cursor_position(&self) -> io::Result<()> {
        let viewport_cursor_position = self.get_viewport_cursor_position();

        queue!(
            io::stdout(),
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

        let splitted_line = self
            .get_line_at_cursor_mut()
            .unwrap()
            .split_off(point.col as usize);

        self.lines.insert(newline, splitted_line);
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
        self.move_cursor_backward();
    }

    fn remove_character_at_cursor(&mut self) {
        let cursor_col = self.cursor.col as usize;

        self.get_line_at_cursor_mut()
            .unwrap()
            .remove(cursor_col - 1);
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
        let range = self.viewport.pos.row..self.viewport.pos.row + self.viewport.height as isize;

        let mut lines = String::new();

        Self::clear_all_screen()?;

        for (terminal_row, line_index) in (0..).zip(range) {
            if terminal_row + self.viewport.pos.row as usize >= self.max_rows() {
                break;
            }

            let line_start = self.viewport.pos.col as usize;
            let line_end =
                line_start + self.viewport.width - Self::CURSOR_COLUMN_START_OFFSET as usize - 1;

            // TODO: the above calculation can overflow (unsigned int) if the terminal window is too small.
            // add a minimum size to the window to fix or think in another solution.
            //
            let line = self.format_line(line_index as usize, line_start, line_end);

            lines += format!("{}{}", MoveTo(0, terminal_row as u16), line).as_str();
        }

        queue!(stdout(), Print(lines))?;

        Ok(())
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
            end = end.clamp(0, line.len() - 1);
        }

        let final_line = if is_valid { &line[start..=end] } else { "" };

        format!(
            "{:>line_width$}   {}", // TODO: automatize: calculate amount of spaces based on a constant
            index + 1,
            final_line,
            line_width = Self::LINE_NUMBER_ALIGNMENT as usize,
        )
    }
}
