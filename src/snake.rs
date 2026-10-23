use std::collections::VecDeque;

use crate::config::{HEIGHT, WIDTH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    #[must_use]
    pub fn step(self, direction: Direction) -> Point {
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

    pub fn is_wall(self) -> bool {
        self.y == 0 || self.y == HEIGHT - 1 || self.x == 0 || self.x == WIDTH - 1
    }
}

pub struct Snake {
    body: VecDeque<Point>,
    pub direction: Direction,
    pub last_step: Direction,
}

impl Snake {
    pub fn new() -> Self {
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

    pub fn head(&self) -> &Point {
        self.body.front().expect("Snake should have a head")
    }

    pub fn make_move(&mut self, food: &Point) -> SnakeStatus {
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

    pub fn contains(&self, point: &Point) -> bool {
        self.body.contains(point)
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }
}

#[derive(Debug, PartialEq)]
pub enum SnakeStatus {
    Alive,
    AteFood,
    BodyHit,
    WallHit,
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
                Point {
                    x: WIDTH / 2 - 2,
                    y: HEIGHT / 2,
                },
                Point {
                    x: WIDTH / 2 - 1,
                    y: HEIGHT / 2,
                },
                Point {
                    x: WIDTH / 2 - 1,
                    y: HEIGHT / 2 - 1,
                },
                Point {
                    x: WIDTH / 2 - 2,
                    y: HEIGHT / 2 - 1,
                },
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
                Point { x: WIDTH - 2, y: 1 },
                Point { x: WIDTH - 3, y: 1 },
                Point { x: WIDTH - 4, y: 1 },
            ]),
            direction: Direction::Right,
            last_step: Direction::Right,
        };
        let food = Point { x: 1, y: 1 };

        let snake_status = snake.make_move(&food);

        assert_eq!(snake_status, SnakeStatus::WallHit);
    }
}
