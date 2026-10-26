use std::time::Duration;

// Размеры поля
pub const WIDTH: u16 = 30;
pub const HEIGHT: u16 = 10;

// Параметры тиков для каждого из уровней сложности
pub const EASY_START_TICK: Duration = Duration::from_millis(200);
pub const EASY_MIN_TICK: Duration = Duration::from_millis(100);
pub const MEDIUM_START_TICK: Duration = Duration::from_millis(150);
pub const MEDIUM_MIN_TICK: Duration = Duration::from_millis(70);
pub const HARD_START_TICK: Duration = Duration::from_millis(100);
pub const HARD_MIN_TICK: Duration = Duration::from_millis(30);
