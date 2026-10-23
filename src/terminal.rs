use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io;
use std::io::stdout;

pub struct TerminalGuard;

impl TerminalGuard {
    /// Конструктор, включающий альтернативный экран, raw-режим терминала и скрывающий курсор
    pub fn new() -> io::Result<TerminalGuard> {
        enable_raw_mode()?;
        execute!(stdout(), Hide)?; // Скрываем курсор на время игры
        execute!(stdout(), EnterAlternateScreen)?;

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    /// Деструктор, выключающий альтернативный экран, raw-режим терминала и возвращающий курсор при выходе из
    /// области видимости
    fn drop(&mut self) {
        let _ = execute!(stdout(), LeaveAlternateScreen);
        let _ = execute!(stdout(), Show); // Возвращаем видимость курсора
        let _ = disable_raw_mode();
    }
}
