use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, read};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::collections::VecDeque;
use std::io;
use std::io::{Write, stdout};

const WIDTH: u16 = 15;
const HEIGHT: u16 = 10;
// Проверка констант, чтобы хвост не спавнился за рамками поля
const _: () = assert!(WIDTH >= 6 && HEIGHT >= 3, "Поле слишком маленькое");

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

        if eats {
            SnakeStatus::AteFood
        } else {
            SnakeStatus::Alive
        }
    }
}

#[derive(Debug)]
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

fn run() -> io::Result<()> {
    let snake = Snake::new();
    let Some(food) = random_food_cell(&snake) else {
        print!("\r\nИгра окончена");
        return Ok(());
    };

    let mut game = Game {
        snake,
        food,
        score: 0,
    };

    loop {
        // Очищает экран и переводит курсор в левый верхний угол
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        print!("\r\nКак играть: w/a/s/d либо стрелочки — ход, q — выход.");

        let game_field = build_field(&game);

        print!("\r\nСчет: {}", game.score);
        print!("\r\n{}", game_field);
        stdout().flush()?; // Сброс буффера для печати последний строки с границой поля

        let Event::Key(key) = read()? else {
            continue;
        };

        if key.code == KeyCode::Char('q') {
            break;
        } else if let Some(direction) = parse_direction(key.code)
            && game.snake.direction.opposite() != direction
        {
            game.snake.direction = direction;
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
                    break;
                }
            },
            SnakeStatus::BodyHit => {
                print!("\r\n\nЗмея ударилась в свое тело");
                break;
            }
            SnakeStatus::WallHit => {
                print!("\r\n\nЗмея ударилась в стену");
                break;
            }
        }
    }

    print!(
        "\r\nИгра окончена. Счет: {}.\r\nСпасибо за игру, всего хорошего!\r\n\n",
        game.score
    );

    Ok(())
}

fn main() -> io::Result<()> {
    execute!(stdout(), Hide)?; // Скрываем курсор на время игры

    enable_raw_mode()?;

    let game_result = run();

    disable_raw_mode()?;

    execute!(stdout(), Show)?; // Возвращаем видимость курсора

    game_result
}
