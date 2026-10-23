use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::poll;
use crossterm::event::{Event, KeyCode, read};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::collections::VecDeque;
use std::io;
use std::io::{Write, stdout};
use std::time::{Duration, Instant};

const WIDTH: u16 = 15;
const HEIGHT: u16 = 10;
// Проверка констант, чтобы хвост не спавнился за рамками поля
const _: () = assert!(WIDTH >= 6 && HEIGHT >= 3, "Поле слишком маленькое");
const START_TICK: Duration = Duration::from_millis(200);
const MIN_TICK: Duration = Duration::from_millis(70);
const SPEEDUP_PER_FOOD: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

impl Point {
    #[must_use]
    fn step(self, direction: Direction) -> Point {
        match direction {
            Direction::Up => Point {
                y: self.y - 1,
                ..self
            },
            Direction::Down => Point {
                y: self.y + 1,
                ..self
            },

            Direction::Left => Point {
                x: self.x - 1,
                ..self
            },
            Direction::Right => Point {
                x: self.x + 1,
                ..self
            },
        }
    }

    fn is_wall(self) -> bool {
        self.y == 0 || self.y == HEIGHT - 1 || self.x == 0 || self.x == WIDTH - 1
    }
}

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
    last_step: Direction,
}

impl Snake {
    fn new() -> Self {
        Snake {
            body: VecDeque::from([
                Point {
                    x: WIDTH / 2,
                    y: HEIGHT / 2,
                },
                Point {
                    x: WIDTH / 2 - 1,
                    y: HEIGHT / 2,
                },
                Point {
                    x: WIDTH / 2 - 2,
                    y: HEIGHT / 2,
                },
            ]),
            direction: Direction::Right,
            last_step: Direction::Right,
        }
    }

    fn head(&self) -> &Point {
        self.body.front().expect("Snake should have a head")
    }

    fn make_move(&mut self, food: &Point) -> SnakeStatus {
        let new_head_pos: Point = self.head().step(self.direction);
        let eats = new_head_pos == *food;

        if new_head_pos.is_wall() {
            return SnakeStatus::WallHit;
        }

        if !eats {
            self.body.pop_back();
        }

        if self.body.contains(&new_head_pos) {
            return SnakeStatus::BodyHit;
        }

        self.body.push_front(new_head_pos);
        self.last_step = self.direction; // Исключает разворот на 180 град. за время 1 тика

        if eats {
            SnakeStatus::AteFood
        } else {
            SnakeStatus::Alive
        }
    }
}

#[derive(Debug, PartialEq)]
enum SnakeStatus {
    Alive,
    AteFood,
    BodyHit,
    WallHit,
}

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

struct TerminalGuard;

impl TerminalGuard {
    /// Конструктор, включающий альтернативный экран, raw-режим терминала и скрывающий курсор
    fn new() -> io::Result<TerminalGuard> {
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
                } else if game.snake.body.contains(&curr_point) {
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
    (WIDTH as usize - 2) * (HEIGHT as usize - 2) > snake.body.len()
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

        if !snake.body.contains(&food_point) {
            return Some(food_point);
        }
    }
}

fn play_game() -> io::Result<u32> {
    let mut game = Game::new();

    'game: loop {
        // Очищает экран и переводит курсор в левый верхний угол
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        print!("\r\nКак играть: w/a/s/d либо ↑/←/↓/→ — ход, q — выход.");

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

                if key.code == KeyCode::Char('q') {
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
    print!("\r\nr - для перезапуска игры; q - для выхода из игры");

    stdout().flush()?;

    loop {
        let Event::Key(key) = read()? else {
            continue;
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposite() {
        let direction_up = Direction::Up;
        let direction_down = Direction::Down;
        let direction_left = Direction::Left;
        let direction_right = Direction::Right;

        assert_eq!(direction_up.opposite(), Direction::Down);
        assert_eq!(direction_down.opposite(), Direction::Up);
        assert_eq!(direction_left.opposite(), Direction::Right);
        assert_eq!(direction_right.opposite(), Direction::Left);
    }

    #[test]
    fn test_snake_does_not_grow_without_food_hit() {
        let mut snake = Snake::new();
        let initial_snake_len = snake.body.len();

        let food = Point { x: 1, y: 1 };

        snake.make_move(&food);

        assert_eq!(initial_snake_len, snake.body.len())
    }

    #[test]
    fn test_snake_grows_on_food_hit() {
        let mut snake = Snake::new();
        let initial_snake_len = snake.body.len();

        let food = Point {
            x: WIDTH / 2 + 1,
            y: HEIGHT / 2,
        };

        let snake_status = snake.make_move(&food);

        assert_eq!(initial_snake_len + 1, snake.body.len());
        assert_eq!(snake_status, SnakeStatus::AteFood);
    }

    #[test]
    fn test_four_length_snake_cant_hit_itself() {
        let mut snake: Snake = Snake {
            body: VecDeque::from([
                Point { x: 5, y: 5 },
                Point { x: 6, y: 5 },
                Point { x: 6, y: 4 },
                Point { x: 5, y: 4 },
            ]),
            direction: Direction::Up,
            last_step: Direction::Left,
        };
        let food = Point { x: 1, y: 1 };

        let tail_position: Point = *snake.body.back().unwrap();
        let snake_status = snake.make_move(&food);

        assert_eq!(snake_status, SnakeStatus::Alive);
        assert_eq!(snake.head(), &tail_position);
    }

    #[test]
    fn test_snake_wall_hit_return_wallhit_status() {
        let mut snake: Snake = Snake {
            body: VecDeque::from([
                Point { x: 13, y: 1 },
                Point { x: 12, y: 1 },
                Point { x: 11, y: 1 },
            ]),
            direction: Direction::Right,
            last_step: Direction::Right,
        };
        let food = Point { x: 1, y: 1 };

        let snake_status = snake.make_move(&food);

        assert_eq!(snake_status, SnakeStatus::WallHit);
    }
}
