extern crate termion;

use crate::game_utils::Position;
use std::any::Any;
use std::io::{stdin, stdout, StdoutLock, Write};
use std::process;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

const TERMINAL_WIDTH: usize = 60;
const TERMINAL_HEIGHT: usize = 30;

const TITLE_POSITION_X: usize = 1;
const TITLE_POSITION_Y: usize = 1;
const TITLE_WIDTH: usize = TERMINAL_WIDTH;
const TITLE_HEIGHT: usize = 3;

const HUD_POSITION_X: usize = 1;
const HUD_POSITION_Y: usize = TITLE_POSITION_Y + TITLE_HEIGHT - 1;
const HUD_WIDTH: usize = TERMINAL_WIDTH;
const HUD_HEIGHT: usize = 6;

const GAME_POSITION_X: usize = 1;
const GAME_POSITION_Y: usize = HUD_POSITION_Y + HUD_HEIGHT - 1;
const GAME_WIDTH: usize = TERMINAL_WIDTH;
const GAME_HEIGHT: usize = TERMINAL_HEIGHT - HUD_HEIGHT - TITLE_HEIGHT;

pub const GAME_AREA_WIDTH: usize = GAME_WIDTH - 2;
pub const GAME_AREA_HEIGHT: usize = GAME_HEIGHT - 2;

type GameSender = Arc<Mutex<Option<Sender<GameData>>>>;

pub type GameScreen<'a> = AlternateScreen<RawTerminal<StdoutLock<'a>>>;

pub trait Draw {
    fn draw(&self, screen: &mut GameScreen);
    fn get_position(&self) -> Position;
}

pub trait Update {
    fn update(&mut self, time_since_last_call: Duration);
}

pub trait GameObject: Draw + Update {
    type Item;
    fn action(&mut self, action: Action);
    fn scene_action(&mut self, action: Self::Item);
    fn as_any(&self) -> &dyn Any;
}

#[derive(Copy, Clone)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    Command(char),
    Quit,
}

#[derive(Copy, Clone)]
pub struct GameData {
    pub action: Action,
}

pub trait GameScene<A> {
    fn set_game_engine(&mut self, game_engine: Arc<Mutex<Game<A>>>);
    fn load(&self);
    fn update(&self, interval: Duration);
    fn draw_hud(&self, width: usize, height: usize) -> Vec<String>;
    fn draw_title(&self, width: usize, height: usize) -> String;
    fn input(&self, game_data: GameData);
}

pub struct Game<A> {
    objects: Arc<Mutex<Vec<Arc<Mutex<Box<dyn GameObject<Item = A> + Send>>>>>>,
    game_scene: Arc<Mutex<Box<dyn GameScene<A> + Send + 'static>>>,
    main_thread_sender: GameSender,
    input_thread_sender: GameSender,
    game_thread_sender: GameSender,
    input_thread: Option<thread::JoinHandle<()>>,
    game_thread: Option<thread::JoinHandle<()>>,
}

impl<A: 'static> Game<A> {
    pub fn new<T: GameScene<A> + Send + 'static>(
        game_scene: Box<T>,
        main_thread_sender: Arc<Mutex<Option<Sender<GameData>>>>,
    ) -> Arc<Mutex<Game<A>>> {
        let game = Game {
            objects: Arc::new(Mutex::new(Vec::new())),
            game_scene: Arc::new(Mutex::new(game_scene)),
            main_thread_sender: Arc::clone(&main_thread_sender),
            input_thread_sender: Arc::new(Mutex::new(None)),
            game_thread_sender: Arc::new(Mutex::new(None)),
            input_thread: None,
            game_thread: None,
        };
        let game = Arc::new(Mutex::new(game));
        let game_clone = Arc::clone(&game);
        game.lock()
            .unwrap()
            .game_scene
            .lock()
            .unwrap()
            .set_game_engine(game_clone);
        game
    }

    pub fn start(&mut self) -> (GameSender, GameSender) {
        // Channel to transmit data to game thread
        let (game_sender, game_receiver) = mpsc::channel::<GameData>();
        *self.game_thread_sender.lock().unwrap() = Some(game_sender);
        // Channel to transmit data to input thread
        let (input_sender, input_receiver) = mpsc::channel::<GameData>();
        *self.input_thread_sender.lock().unwrap() = Some(input_sender);

        let main_sender = Arc::clone(&self.main_thread_sender);
        let game_sender = Arc::clone(&self.game_thread_sender);

        let thread = thread::spawn(move || {
            Self::input_thread(main_sender, game_sender, input_receiver);
        });
        self.input_thread = Some(thread);
        let objects = Arc::clone(&self.objects);
        let game_scene = Arc::clone(&self.game_scene);
        let main_sender = Arc::clone(&self.main_thread_sender);
        let input_sender = Arc::clone(&self.input_thread_sender);

        let thread = thread::spawn(move || {
            Self::game_thread(
                objects,
                game_scene,
                main_sender,
                input_sender,
                game_receiver,
            );
        });
        self.game_thread = Some(thread);

        (
            Arc::clone(&self.input_thread_sender),
            Arc::clone(&self.game_thread_sender),
        )
    }

    pub fn add_object(&mut self, object: Arc<Mutex<Box<dyn GameObject<Item = A> + Send>>>) {
        self.objects.lock().unwrap().push(object);
    }

    fn input_thread(
        main_thread_sender: GameSender,
        game_sender: GameSender,
        _input_receiver: Receiver<GameData>,
    ) {
        let stdin = stdin();
        let stdin = stdin.lock();
        let mut stdin = stdin.keys();

        loop {
            let action = match stdin.next().unwrap().unwrap() {
                Key::Esc => Action::Quit,
                Key::Up => Action::MoveUp,
                Key::Down => Action::MoveDown,
                Key::Left => Action::MoveLeft,
                Key::Right => Action::MoveRight,
                Key::Char(c) => Action::Command(c),
                _ => continue,
            };
            let game_data = GameData { action };
            if let Some(sender) = &*game_sender.lock().unwrap() {
                sender.send(game_data).unwrap();
            }
            if let Some(sender) = &*main_thread_sender.lock().unwrap() {
                sender.send(game_data).unwrap();
            }
            if let Action::Quit = action {
                break;
            }
        }
    }

    pub fn draw_point(screen: &mut GameScreen, c: &str, x: usize, y: usize) {
        if x < 1 || y < 1 {
            return;
        }
        let x = (x + GAME_POSITION_X) as u16;
        let y = (y + GAME_POSITION_Y) as u16;
        write!(screen, "{}{}", termion::cursor::Goto(x, y), c).unwrap();
    }

    fn game_thread(
        game_objects: Arc<Mutex<Vec<Arc<Mutex<Box<dyn GameObject<Item = A> + Send>>>>>>,
        game_scene: Arc<Mutex<Box<dyn GameScene<A> + Send + 'static>>>,
        _main_thread_sender: GameSender,
        _input_thread_sender: GameSender,
        game_receiver: Receiver<GameData>,
    ) {
        use termion::clear::{AfterCursor, All};
        use termion::cursor::Goto;

        let stdout = stdout();
        let stdout = stdout.lock();
        let stdout = stdout.into_raw_mode().unwrap();

        let mut screen = AlternateScreen::from(stdout);

        let mut refresh_screen = true;
        let mut refresh_interval = Instant::now();
        let mut now = Instant::now();

        let game_scene = game_scene.lock().unwrap();
        game_scene.load();
        // Clean screen
        write!(screen, "{}", All).unwrap();
        write!(screen, "{}", Goto(1, 1)).unwrap();

        // Draw title
        let title = game_scene.draw_title(TITLE_WIDTH - 2, TITLE_HEIGHT - 2);
        write!(
            screen,
            "{}{}",
            Goto(TITLE_POSITION_X as u16 + 1, TITLE_POSITION_Y as u16 + 1),
            title
        )
        .unwrap();
        Self::draw_title_square(
            &mut screen,
            TITLE_POSITION_X,
            TITLE_POSITION_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
        );
        loop {
            // Update Game Objects
            if now.elapsed().as_millis() > 100 {
                for o in &mut *game_objects.lock().unwrap() {
                    o.lock().unwrap().update(now.elapsed());
                }
                game_scene.update(now.elapsed());
                now = Instant::now();
            }

            // Update Screen
            if refresh_screen {
                write!(
                    screen,
                    "{}",
                    termion::cursor::Goto(HUD_POSITION_X as u16, HUD_POSITION_Y as u16)
                )
                .unwrap();
                write!(screen, "{}", AfterCursor).unwrap();

                // Draw HUD
                let hud = game_scene.draw_hud(HUD_WIDTH - 2, HUD_HEIGHT - 2);
                for (i, text) in hud.iter().enumerate() {
                    if i > (HUD_HEIGHT - 2) {
                        break;
                    }
                    write!(
                        screen,
                        "{}{}",
                        Goto(
                            HUD_POSITION_X as u16 + 1,
                            HUD_POSITION_Y as u16 + 1 + i as u16
                        ),
                        text,
                    )
                    .unwrap();
                }
                Self::draw_hud_square(
                    &mut screen,
                    HUD_POSITION_X,
                    HUD_POSITION_Y,
                    HUD_WIDTH,
                    HUD_HEIGHT,
                );

                // Draw Objects
                for o in &*game_objects.lock().unwrap() {
                    o.lock().unwrap().draw(&mut screen);
                }
                Self::draw_game_square(
                    &mut screen,
                    GAME_POSITION_X,
                    GAME_POSITION_Y,
                    GAME_WIDTH,
                    GAME_HEIGHT,
                );

                write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();

                screen.flush().unwrap();
                refresh_screen = false;
                refresh_interval = Instant::now();
            } else if refresh_interval.elapsed() > Duration::from_secs_f32(1f32 / 12f32) {
                refresh_screen = true;
            }

            let game_data = match game_receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(game_object) => game_object,
                Err(_) => continue,
            };
            game_scene.input(game_data);
            if let Action::Quit = game_data.action {
                break;
            }
        }

        screen.suspend_raw_mode().unwrap();
        print!("{}", termion::clear::All);
        process::exit(0);
    }

    fn draw_bezel(
        screen: &mut GameScreen,
        border: &str,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        use termion::cursor::Goto;
        // border should follow this order: top-left, top-right, bottom-left and bottom-right corner,
        // horizontal and vertical bars
        let mut b = Vec::<char>::new();
        for c in border.chars() {
            b.push(c);
        }
        let mut border = String::from(border);
        border.push_str("\n\x08");
        let horizontal = String::from(b[4]).repeat(width - 2);
        let mut vertical = String::from(b[5]);
        vertical.push_str("\n\x08");
        let vertical = vertical.repeat(height - 2);

        let (x, y, width, height) = (x as u16, y as u16, width as u16, height as u16);
        write!(screen, "{}{}{}{}", Goto(x, y), b[0], horizontal, b[1]).unwrap();
        write!(
            screen,
            "{}{}{}{}",
            Goto(x, y + height - 1),
            b[2],
            horizontal,
            b[3]
        )
        .unwrap();
        write!(screen, "{}{}", Goto(x, y + 1), vertical).unwrap();
        write!(screen, "{}{}", Goto(x + width - 1, y + 1), vertical).unwrap();
    }

    fn draw_title_square(screen: &mut GameScreen, x: usize, y: usize, width: usize, height: usize) {
        Self::draw_bezel(screen, "╒╕┝┥═│", x, y, width, height);
    }
    fn draw_hud_square(screen: &mut GameScreen, x: usize, y: usize, width: usize, height: usize) {
        Self::draw_bezel(screen, "┝┥┝┥─│", x, y, width, height);
    }
    fn draw_game_square(screen: &mut GameScreen, x: usize, y: usize, width: usize, height: usize) {
        Self::draw_bezel(screen, "┝┥└┘─│", x, y, width, height);
    }
}

impl<A> Drop for Game<A> {
    fn drop(&mut self) {
        let thread = self.input_thread.take();
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
        let thread = self.game_thread.take();
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }
}
