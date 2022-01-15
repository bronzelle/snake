use crate::apple::Apple;
use crate::game_engine;
use crate::game_engine::{Action, Game, GameData, GameObject, GameScene};
use crate::snake::Snake;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub enum SnakeGameSceneActions {
    EatApple,
    HitWall,
    Restart,
}

pub struct SnakeGameScene {
    game_engine: Option<Arc<Mutex<Game<SnakeGameSceneActions>>>>,
    snake: Arc<Mutex<Box<dyn GameObject<Item = SnakeGameSceneActions> + Send + 'static>>>,
    apple: Arc<Mutex<Box<dyn GameObject<Item = SnakeGameSceneActions> + Send + 'static>>>,
}

impl SnakeGameScene {
    pub fn new() -> SnakeGameScene {
        let snake = Snake::new();
        snake.add_body();
        snake.add_body();
        snake.add_body();
        snake.add_body();
        let apple = Apple::default();

        SnakeGameScene {
            game_engine: None,
            snake: Arc::new(Mutex::new(Box::new(snake))),
            apple: Arc::new(Mutex::new(Box::new(apple))),
        }
    }

    fn eat_apple(&self) {
        (*self.apple.lock().unwrap()).scene_action(SnakeGameSceneActions::EatApple);
        (*self.snake.lock().unwrap()).scene_action(SnakeGameSceneActions::EatApple);
    }

    fn hit_wall(&self) {
        (*self.apple.lock().unwrap()).scene_action(SnakeGameSceneActions::HitWall);
        (*self.snake.lock().unwrap()).scene_action(SnakeGameSceneActions::HitWall);
    }

    fn restart(&self) {
        (*self.apple.lock().unwrap()).scene_action(SnakeGameSceneActions::Restart);
        (*self.snake.lock().unwrap()).scene_action(SnakeGameSceneActions::Restart);
    }

    fn controls(&self, c: char) {
        match c {
            'r' | 'R' => self.restart(),
            _ => (),
        }
    }
}

impl GameScene<SnakeGameSceneActions> for SnakeGameScene {
    fn set_game_engine(&mut self, game_engine: Arc<Mutex<Game<SnakeGameSceneActions>>>) {
        self.game_engine = Some(game_engine);
    }
    fn load(&self) {
        if let Some(game) = &self.game_engine {
            let mut game = game.lock().unwrap();
            game.add_object(Arc::clone(&self.snake));
            game.add_object(Arc::clone(&self.apple));
        }
    }
    fn update(&self, _interval: Duration) {
        let snake_position = self.snake.lock().unwrap().get_position();
        let apple_position = self.apple.lock().unwrap().get_position();
        let (x, y) = snake_position.get_screen_coordinates();
        if !(1..=game_engine::GAME_AREA_WIDTH).contains(&x)
            || !(1..=game_engine::GAME_AREA_HEIGHT).contains(&y)
        {
            self.hit_wall();
        } else if snake_position == apple_position {
            self.eat_apple();   
        }
    }
    fn draw_hud(&self, _width: usize, _height: usize) -> Vec<String> {
        let snake = &*self.snake.lock().unwrap();
        let snake: &Snake = match snake.as_any().downcast_ref::<Snake>() {
            Some(snake) => snake,
            None => panic!("No snake"),
        };
        let speed = format!(
            "Speed  :{:>6} blocks/second",
            snake.speed().get_speed_steps_per_second()
        );
        let controls = String::from("[R] - Restart Game  /  Arrow keys - Change snake direction");
        let lives = format!("Lives  :{:>6}", snake.lives());
        let apples = format!("Apples :{:>6}", snake.apples());
        let resp = vec![speed, lives, apples, controls];
        resp
    }
    fn draw_title(&self, width: usize, _height: usize) -> String {
        let mut s = String::new();
        s.push_str(&" ".repeat((width - 13) / 2));
        s.push_str("*** SNAKE ***");
        s.push_str(&" ".repeat((width - 13) / 2));
        s
    }
    fn input(&self, game_data: GameData) {
        match game_data.action {
            Action::Command(c) => self.controls(c),
            Action::MoveDown | Action::MoveUp | Action::MoveLeft | Action::MoveRight => {
                self.snake.lock().unwrap().action(game_data.action)
            }
            _ => (),
        }
    }
}
