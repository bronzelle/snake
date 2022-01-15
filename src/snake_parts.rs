use std::sync::{Arc, Mutex, Weak};
use crate::game_utils::{Direction, Position};

pub struct SnakeBody {
    pub _parent: Mutex<Option<Weak<SnakeBody>>>,
    pub child: Mutex<Option<Arc<SnakeBody>>>,
    pub position: Mutex<Position>,
    pub direction: Mutex<Direction>,
}

impl SnakeBody {
    pub fn new(x: u16, y: u16) -> Self {
        SnakeBody {
            _parent: Mutex::new(None),
            child: Mutex::new(None),
            position: Mutex::new(Position { x: x as f64, y: y as f64 }),
            direction: Mutex::new(Direction::right()),
        }
    }
}

impl Default for SnakeBody {
    fn default() -> Self {
        SnakeBody::new(20, 20)
    }
}
