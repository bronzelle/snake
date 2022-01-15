use std::fmt::{Error, Formatter};
use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
pub struct Direction {
    x: i8,
    y: i8,
}

impl Direction {
    pub fn up() -> Direction {
        Direction { x: 0, y: -1 }
    }
    pub fn down() -> Direction {
        Direction { x: 0, y: 1 }
    }
    pub fn right() -> Direction {
        Direction { x: 1, y: 0 }
    }
    pub fn left() -> Direction {
        Direction { x: -1, y: 0 }
    }
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}

impl Mul<f64> for Direction {
    type Output = Position;
    fn mul(self, rhs: f64) -> Self::Output {
        Position {
            x: f64::from(self.x) * rhs,
            y: f64::from(self.y) * rhs,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match *self {
                Direction { x: 0, y: -1 } => String::from("^"),
                Direction { x: 0, y: 1 } => String::from("v"),
                Direction { x: 1, y: 0 } => String::from(">"),
                Direction { x: -1, y: 0 } => String::from("<"),
                Direction { x: _, y: _ } => String::from("*"),
            }
        )?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn get_screen_coordinates(&self) -> (usize, usize) {
        (self.x.round() as usize, self.y.round() as usize)
    }
}

impl Add<Position> for Position {
    type Output = Position;
    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        let left_screen = self.get_screen_coordinates();
        let other_screen = other.get_screen_coordinates();
        left_screen.0 == other_screen.0 && left_screen.1 == other_screen.1
    }

    fn ne(&self, other: &Self) -> bool {
        let left_screen = self.get_screen_coordinates();
        let other_screen = other.get_screen_coordinates();
        left_screen.0 != other_screen.0 || left_screen.1 != other_screen.1
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[{}, {}]", self.x, self.y)?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Speed {
    period_in_milliseconds: u128,
}

impl Speed {
    pub fn period_in_milliseconds(period_in_milliseconds: u128) -> Speed {
        Speed {
            period_in_milliseconds,
        }
    }

    pub fn _period_in_seconds(period_in_seconds: u128) -> Speed {
        Speed {
            period_in_milliseconds: period_in_seconds * 1000,
        }
    }

    pub fn get_speed_steps_per_millisecond(&self) -> f64 {
        1f64 / self.period_in_milliseconds as f64
    }
    pub fn get_speed_steps_per_second(&self) -> f64 {
        1000f64 / self.period_in_milliseconds as f64
    }
}
