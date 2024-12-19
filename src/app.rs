use std::io::{self, stdout};

use crossterm::{
    terminal::{self, DisableLineWrap, EnableLineWrap},
    ExecutableCommand,
};

pub fn init() -> Result<(), io::Error> {
    terminal::enable_raw_mode()?;
    stdout().execute(DisableLineWrap)?;

    Ok(())
}

pub fn deinit() -> Result<(), io::Error> {
    terminal::disable_raw_mode()?;
    stdout().execute(EnableLineWrap)?;

    Ok(())
}
