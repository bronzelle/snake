use crate::game_engine;
use crate::game_engine::{Action, Draw, Game, GameObject, GameScreen, Update};
use crate::game_utils::{Direction, Position, Speed};
use crate::snake_game_scene::SnakeGameSceneActions;
use crate::snake_parts::SnakeBody;
use std::any::Any;
use std::fmt::{Error, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_POSITION_X: u16 = (game_engine::GAME_AREA_WIDTH / 2) as u16;
const DEFAULT_POSITION_Y: u16 = (game_engine::GAME_AREA_HEIGHT / 2) as u16;

pub struct Snake {
    body: Mutex<Arc<SnakeBody>>,
    speed: Mutex<Speed>,
    lives: Mutex<u16>,
    apples: Mutex<u16>,
}

impl Snake {
    pub fn new() -> Snake {
        let body = SnakeBody::new(DEFAULT_POSITION_X, DEFAULT_POSITION_Y);
        let body = Mutex::new(Arc::new(body));
        let speed = Speed::period_in_milliseconds(100);
        Snake {
            body,
            speed: Mutex::new(speed),
            lives: Mutex::new(5),
            apples: Mutex::new(0),
        }
    }

    pub fn add_body(&self) {
        self.add_body_locked(&self.body.lock().unwrap());
    }

    pub fn apples(&self) -> u16 {
        *self.apples.lock().unwrap()
    }

    pub fn lives(&self) -> u16 {
        *self.lives.lock().unwrap()
    }

    pub fn speed(&self) -> Speed {
        *self.speed.lock().unwrap()
    }

    fn add_body_locked(&self, body: &Arc<SnakeBody>) {
        let tail = Self::find_last_body_part(body);
        let tail_direction = tail.direction.lock().unwrap();
        let position = *tail.position.lock().unwrap() + (*tail_direction * -1f64);
        let new_tail = SnakeBody {
            _parent: Mutex::new(Some(Arc::downgrade(&tail))),
            child: Mutex::new(None),
            position: Mutex::new(position),
            direction: Mutex::new(*tail_direction),
        };
        *tail.child.lock().unwrap() = Some(Arc::new(new_tail));
    }

    fn find_last_body_part(snake: &Arc<SnakeBody>) -> Arc<SnakeBody> {
        if let Some(snake) = &*snake.child.lock().unwrap() {
            return Self::find_last_body_part(snake);
        }
        Arc::clone(snake)
    }

    fn navigate(text: &mut String, snake: &Arc<SnakeBody>) {
        text.push_str(
            format!(
                "\n{}{}",
                snake.direction.lock().unwrap(),
                snake.position.lock().unwrap()
            )
            .as_str(),
        );
        if let Some(snake) = &*snake.child.lock().unwrap() {
            Self::navigate(text, snake);
        }
    }

    fn update_position(snake_parent: &mut Arc<SnakeBody>, parent_direction_before_move: Direction) {
        if let Some(snake) = &mut *snake_parent.child.lock().unwrap() {
            let current_direction = *snake.direction.lock().unwrap();
            *snake.position.lock().unwrap() =
                *snake_parent.position.lock().unwrap() + parent_direction_before_move * -1f64;
            *snake.direction.lock().unwrap() = parent_direction_before_move;
            Self::update_position(snake, current_direction);
        }
    }

    fn check_collision(&self, mut body: Arc<SnakeBody>) -> bool {
        //let mut body = Arc::clone(&*self.body.lock().unwrap());
        let position = (*body.position.lock().unwrap()).get_screen_coordinates();
        loop {
            let child = match &*body.child.lock().unwrap() {
                Some(child) => Arc::clone(child),
                None => return false,
            };
            let child_position = (*child.position.lock().unwrap()).get_screen_coordinates();
            if position == child_position {
                return true;
            }
            body = child;
        }
    }

    fn reset(&self, body: &mut Arc<SnakeBody>) {
        *body.position.lock().unwrap() = Position {
            x: DEFAULT_POSITION_X as f64,
            y: DEFAULT_POSITION_Y as f64,
        };
        (*body.child.lock().unwrap()).take();
        self.add_body_locked(body);
        self.add_body_locked(body);
        self.add_body_locked(body);
        self.add_body_locked(body);
    }
    fn eat_apple(&mut self) {
        let body = &mut self.body.lock().unwrap();
        self.add_body_locked(body);
        let apples = &mut *self.apples.lock().unwrap();
        *apples += 1;
    }
    fn hit_wall(&mut self) {
        {
            let lives = &mut *self.lives.lock().unwrap();
            if *lives > 1 {
                *lives -= 1;
                let body = &mut self.body.lock().unwrap();
                self.reset(&mut *body);
                return;
            }
        }
        self.restart();
    }
    fn restart(&mut self) {
        let mut lives = self.lives.lock().unwrap();
        let mut apples = self.apples.lock().unwrap();
        let body = &mut self.body.lock().unwrap();
        *lives = 5;
        *apples = 0;
        self.reset(&mut *body);
    }
}

impl Update for Snake {
    fn update(&mut self, time_since_last_call: Duration) {
        let body = &mut self.body.lock().unwrap();
        let position = *body.position.lock().unwrap();
        let direction = *body.direction.lock().unwrap();
        let speed = self.speed.lock().unwrap().get_speed_steps_per_millisecond();
        let (x0, y0) = body.position.lock().unwrap().get_screen_coordinates();
        *body.position.lock().unwrap() =
            position + direction * (speed * time_since_last_call.as_millis() as f64);
        let (x1, y1) = body.position.lock().unwrap().get_screen_coordinates();
        if x0 != x1 || y0 != y1 {
            Snake::update_position(&mut *body, direction);
            if self.check_collision(Arc::clone(&*body)) {
                let mut lives = self.lives.lock().unwrap();
                *lives -= 1;
                self.reset(&mut *body);
            }
        }
    }
}

impl Draw for Snake {
    fn draw(&self, screen: &mut GameScreen) {
        let mut body = Arc::clone(&*self.body.lock().unwrap());
        let mut i = 0u8;
        loop {
            let (x, y) = body.position.lock().unwrap().get_screen_coordinates();
            if x > 0 && y > 0 {
                // let direction = *body.direction.lock().unwrap();
                // if direction == Direction::up() || direction == Direction::down() {

                Game::<SnakeGameSceneActions>::draw_point(screen, "▒", x, y);
                // }
                // Game::<SnakeGameSceneActions>::draw_point(("0", x, y);
            }
            let child = match &*body.child.lock().unwrap() {
                Some(body) => Arc::clone(body),
                None => break,
            };
            body = child;
            i = (i + 1) % 10;
        }
    }
    fn get_position(&self) -> Position {
        *(*self.body.lock().unwrap()).position.lock().unwrap()
    }
}

impl GameObject for Snake {
    type Item = SnakeGameSceneActions;

    fn action(&mut self, action: Action) {
        let body = &mut self.body.lock().unwrap();
        let mut body_direction = body.direction.lock().unwrap();
        let (new_direction, opposite_direction) = match action {
            Action::MoveDown => (Direction::down(), Direction::up()),
            Action::MoveUp => (Direction::up(), Direction::down()),
            Action::MoveRight => (Direction::right(), Direction::left()),
            Action::MoveLeft => (Direction::left(), Direction::right()),
            _ => return,
        };
        if new_direction != *body_direction && opposite_direction != *body_direction {
            *body_direction = new_direction;
            // FIX Try to find a way to force move as soon as the action is sent
            //     Code below doesn't respect speed.
            // let mut body_position = body.position.lock().unwrap();
            // *body_position = *body_position + *body_direction * 0.49999f64;
        };
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn scene_action(&mut self, action: SnakeGameSceneActions) {
        match action {
            SnakeGameSceneActions::EatApple => self.eat_apple(),
            SnakeGameSceneActions::HitWall => self.hit_wall(),
            SnakeGameSceneActions::Restart => self.restart(),
        }
    }
}

impl std::fmt::Display for Snake {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut text = String::new();
        Self::navigate(&mut text, &self.body.lock().unwrap());
        write!(f, "{}", text)?;
        Ok(())
    }
}

impl Default for Snake {
    fn default() -> Self {
        Snake::new()
    }
}
