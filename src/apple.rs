use crate::game_engine;
use crate::game_engine::{Action, Draw, Game, GameObject, GameScreen, Update};
use crate::game_utils::Position;
use crate::snake_game_scene::SnakeGameSceneActions;
use rand::Rng;
use std::any::Any;
use std::sync::Mutex;
use std::time::Duration;

pub struct Apple {
    position: Mutex<Position>,
}

impl Apple {
    pub fn new(position: Position) -> Apple {
        Apple {
            position: Mutex::new(position),
        }
    }

    pub fn get_random_position() -> Position {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(1..game_engine::GAME_AREA_WIDTH);
        let y = rng.gen_range(1..game_engine::GAME_AREA_HEIGHT);
        Position {
            x: x as f64,
            y: y as f64,
        }
    }
}

impl Default for Apple {
    fn default() -> Self {
        Apple::new(Apple::get_random_position())
    }
}

impl Draw for Apple {
    fn draw(&self, screen: &mut GameScreen) {
        let (x, y) = self.position.lock().unwrap().get_screen_coordinates();
        Game::<SnakeGameSceneActions>::draw_point(screen, "🍎", x, y);
    }
    fn get_position(&self) -> Position {
        *self.position.lock().unwrap()
    }
}

impl Update for Apple {
    fn update(&mut self, _time_since_last_call: Duration) {}
}

impl GameObject for Apple {
    type Item = SnakeGameSceneActions;
    fn action(&mut self, _action: Action) {
        // if let Action::EatApple = action {
        //     *self.position.lock().unwrap() = Self::get_random_position();
        // }
    }

    fn scene_action(&mut self, action: SnakeGameSceneActions) {
        if let SnakeGameSceneActions::EatApple = action {
            *self.position.lock().unwrap() = Self::get_random_position()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
