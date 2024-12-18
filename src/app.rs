use std::io;

use crossterm::{
    execute,
    terminal::{self, DisableLineWrap},
};

pub fn init() -> Result<(), io::Error> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), DisableLineWrap)?;

    Ok(())
}

pub fn deinit() -> Result<(), io::Error> {
    terminal::disable_raw_mode()?;

    Ok(())
}
