mod apple;
mod game_engine;
mod game_utils;
mod snake;
mod snake_game_scene;
mod snake_parts;

extern crate termion;

use crate::game_engine::{Action, Game, GameData};
use crate::snake_game_scene::SnakeGameScene;
use std::sync::{mpsc, Arc, Mutex};

fn main() {
    let (sender, receiver) = mpsc::channel::<GameData>();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let game_scene = SnakeGameScene::new();

    let game = Game::new(Box::new(game_scene), Arc::clone(&sender));
    game.lock().unwrap().start();

    loop {
        if let Ok(game_data) = receiver.recv() {
            if let Action::Quit = game_data.action {
                break;
            }
        };
    }
}
