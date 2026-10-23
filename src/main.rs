use crossterm::cursor::MoveTo;
use crossterm::event::poll;
use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use std::io;
use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use crate::config::{HEIGHT, WIDTH};
use crate::snake::{Direction, Point, Snake, SnakeStatus};
use crate::terminal::TerminalGuard;

// Проверка констант, чтобы хвост не спавнился за рамками поля
const _: () = assert!(WIDTH >= 6 && HEIGHT >= 3, "Поле слишком маленькое");
const START_TICK: Duration = Duration::from_millis(200);
const MIN_TICK: Duration = Duration::from_millis(70);
const SPEEDUP_PER_FOOD: u64 = 10;

mod config;
mod snake;
mod terminal;

struct Game {
    snake: Snake,
    food: Point,
    score: u32,
}

impl Game {
    fn new() -> Self {
        let snake = Snake::new();
        let food = random_food_cell(&snake)
            .expect("Food should be able to spawn at the beginning of the game");

        Game {
            snake,
            food,
            score: 0,
        }
    }

    /// Тик игры с учётом счёта, но не меньше MIN_TICK.
    fn tick(&self) -> Duration {
        let speedup = Duration::from_millis(u64::from(self.score) * SPEEDUP_PER_FOOD);
        let new_tick = START_TICK.saturating_sub(speedup);

        new_tick.max(MIN_TICK)
    }
}

fn build_field(game: &Game) -> String {
    // Символ переноса строки + возврата каретки "\r\n" - 2 байт
    let mut game_field = String::with_capacity(((WIDTH + 2) * HEIGHT) as usize);
    let head = game.snake.head();

    for line in 0..HEIGHT {
        if line == 0 || line == HEIGHT - 1 {
            game_field.push_str("\r\n");
            game_field.push_str(&"#".repeat(WIDTH as usize));
            continue;
        }

        for col in 0..WIDTH {
            if col == 0 {
                game_field.push_str("\r\n#");
            } else if col == WIDTH - 1 {
                game_field.push('#');
            } else {
                let curr_point = Point { y: line, x: col };

                if curr_point == *head {
                    game_field.push('@');
                } else if game.snake.contains(&curr_point) {
                    game_field.push('o');
                } else if curr_point == game.food {
                    game_field.push('*')
                } else {
                    game_field.push('.');
                }
            }
        }
    }

    game_field
}

fn parse_direction(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::Char('w') | KeyCode::Up => Some(Direction::Up),
        KeyCode::Char('s') | KeyCode::Down => Some(Direction::Down),
        KeyCode::Char('d') | KeyCode::Right => Some(Direction::Right),
        KeyCode::Char('a') | KeyCode::Left => Some(Direction::Left),
        _ => None,
    }
}

fn has_free_cells(snake: &Snake) -> bool {
    (WIDTH as usize - 2) * (HEIGHT as usize - 2) > snake.len()
}

fn random_food_cell(snake: &Snake) -> Option<Point> {
    if !has_free_cells(snake) {
        return None;
    }

    loop {
        let food_point = Point {
            x: rand::random_range(1..WIDTH - 1),
            y: rand::random_range(1..HEIGHT - 1),
        };

        if !snake.contains(&food_point) {
            return Some(food_point);
        }
    }
}

fn play_game() -> io::Result<u32> {
    let mut game = Game::new();

    'game: loop {
        // Очищает экран и переводит курсор в левый верхний угол
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        print!("\r\nКак играть: w/a/s/d либо ↑/←/↓/→ — ход, Esc/q — выход.");

        let game_field = build_field(&game);

        print!("\r\nСчет: {}", game.score);
        print!("\r\n{}", game_field);
        stdout().flush()?; // Сброс буффера для печати последний строки с границой поля

        let tick_start = Instant::now(); // Внутренний таймер для тика, чтобы нажатия клавишь не ускоряли змейку
        let game_tick = game.tick();

        loop {
            let tick_elapsed = game_tick.saturating_sub(tick_start.elapsed());
            if tick_elapsed == Duration::ZERO {
                break;
            }

            if poll(tick_elapsed)? {
                let Event::Key(key) = read()? else {
                    continue;
                };

                // Обработка для Windows, где нажатие считается как 2 команды: Press, Release (На MacOS только Press)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break 'game;
                } else if let Some(direction) = parse_direction(key.code)
                    && game.snake.last_step.opposite() != direction
                {
                    game.snake.direction = direction;
                }
            }
        }

        match game.snake.make_move(&game.food) {
            SnakeStatus::Alive => continue,
            SnakeStatus::AteFood => match random_food_cell(&game.snake) {
                Some(food) => {
                    game.food = food;
                    game.score += 1
                }
                None => {
                    game.score += 1;
                    print!(
                        "\r\nПобеда! Змейка заполнила все клетки. Свободных клеток не осталось."
                    );
                    break 'game;
                }
            },
            SnakeStatus::BodyHit => {
                print!("\r\n\nЗмея ударилась в свое тело");
                break 'game;
            }
            SnakeStatus::WallHit => {
                print!("\r\n\nЗмея ударилась в стену");
                break 'game;
            }
        }
    }

    print!(
        "\r\nИгра окончена. Счет: {}.\r\nСпасибо за игру, всего хорошего!\r\n\n",
        game.score
    );

    Ok(game.score)
}

fn ask_restart() -> io::Result<bool> {
    print!("\r\nr - для перезапуска игры; Esc/q - для выхода из игры");

    stdout().flush()?;

    loop {
        let Event::Key(key) = read()? else {
            continue;
        };

        // Обработка для Windows, где нажатие считается как 2 команды: Press, Release (На MacOS только Press)
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('r') => break Ok(true),
            KeyCode::Char('q') => break Ok(false),
            KeyCode::Esc => break Ok(false),
            _ => (),
        }
    }
}

fn game_cycle() -> io::Result<u32> {
    loop {
        let game_score = play_game()?;

        let has_restart_game = ask_restart()?;

        if !has_restart_game {
            break Ok(game_score);
        }
    }
}

fn main() -> io::Result<()> {
    // Guard  защищает терминал от поломки игры (в случае panic или except) в raw режиме терминала
    let guard = TerminalGuard::new()?;

    let game_score = game_cycle();

    drop(guard);

    println!(
        "\nИгра окончена. Счет: {}.\r\nСпасибо за игру, всего хорошего!\r\n\n",
        game_score?
    );

    Ok(())
}
