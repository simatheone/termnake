use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use std::time::Duration;

use crate::config::{
    EASY_MIN_TICK, EASY_START_TICK, HARD_MIN_TICK, HARD_START_TICK, MEDIUM_MIN_TICK,
    MEDIUM_START_TICK,
};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use std::io::{self, Write, stdout};

const MENU: [(&str, Option<Difficulty>); 4] = [
    ("Easy", Some(Difficulty::Easy)),
    ("Medium", Some(Difficulty::Medium)),
    ("Hard", Some(Difficulty::Hard)),
    ("Exit", None),
];

pub struct DifficultyTickLevel {
    pub start_tick: Duration,
    pub min_tick: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn tick_range(&self) -> DifficultyTickLevel {
        match self {
            Difficulty::Easy => DifficultyTickLevel {
                start_tick: EASY_START_TICK,
                min_tick: EASY_MIN_TICK,
            },
            Difficulty::Medium => DifficultyTickLevel {
                start_tick: MEDIUM_START_TICK,
                min_tick: MEDIUM_MIN_TICK,
            },
            Difficulty::Hard => DifficultyTickLevel {
                start_tick: HARD_START_TICK,
                min_tick: HARD_MIN_TICK,
            },
        }
    }
}

fn build_screen(selected: usize) {
    for (idx, (lvl_text, _)) in MENU.iter().enumerate() {
        if idx == MENU.len() - 1 {
            print!("\r\n");
        }

        if selected == idx {
            print!("\r\n> {}", lvl_text);
        } else {
            print!("\r\n  {}", lvl_text);
        }
    }
}

pub fn menu() -> io::Result<Option<Difficulty>> {
    let mut selected: usize = 0;

    loop {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        print!("\r\nВыберите уровень сложности:");

        build_screen(selected);

        stdout().flush()?;

        let Event::Key(key) = read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if key.code == KeyCode::Char('w') || key.code == KeyCode::Up {
            selected = (selected + MENU.len() - 1) % MENU.len()
        } else if key.code == KeyCode::Char('s') || key.code == KeyCode::Down {
            selected = (selected + 1) % MENU.len()
        } else if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            return Ok(None);
        } else if key.code == KeyCode::Enter {
            return Ok(MENU[selected].1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_ticks() {
        let mut prev_lvl_start_tick: Option<Duration> = None;

        for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let tick_range = difficulty.tick_range();
            assert!(tick_range.start_tick > tick_range.min_tick);

            if let Some(prev) = prev_lvl_start_tick {
                assert!(prev > tick_range.start_tick);
            }

            prev_lvl_start_tick = Some(tick_range.start_tick);
        }
    }
}
