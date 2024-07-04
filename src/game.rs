use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle, Thread}, time::Duration};

use crate::{enemy::Enemy, player::Player};

pub trait Drawable {
    fn get_pos(&self) -> (f32,f32);
    fn get_shapes(&self) -> &Vec<(String, Shape, (f32,f32))>;
}

pub fn draw<T: Drawable>(object: &T) -> String {
    let (x, y) = object.get_pos();
    let shapes = object.get_shapes();
    let mut o = "".to_owned();
    for shape in shapes {
        let (color, shape, (sx, sy)) = shape;
        let s = format!("[{:?}],", (color, shape, (x + sx, y + sy)));
        o.push_str(&s);
    }
    o
}

pub trait Moveable {
    fn get_x(&mut self) -> &mut f32;
    fn get_y(&mut self) -> &mut f32;
    fn get_velocity(&self) -> &(f32, f32);
}

pub fn move_object<T: Moveable + std::fmt::Debug>(object: &mut T) {
    let (vx, vy) = object.get_velocity().clone();
    *(object.get_x()) += vx;
    *(object.get_y()) += vy;
    // println!("{}, {}", object.get_x().clone(), object.get_y().clone());
}

#[derive(Debug)]
pub enum Shape {
    Circle{radius: f32},
    Rectangle{width: f32, height: f32},
    Line{x: f32, y: f32},
    Text{content: String, size: f32},
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle { radius: 20.0 }
        // Self::Rectangle { width: 10.0, height: 20.0 }
        // Self::Line {x: 0.0, y: 0.0}
    }
}

#[derive(Debug)]
pub struct Game {
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Vec<Enemy>,
}

impl Game {
    pub fn new() -> Game {
        let mut g = Game {
            players: vec![],
            game_loop: None,
            running: false,
            enemies: vec![],
        };
        g.enemies.push(Enemy::new(200.0, 100.0, (0.3, 0.0)));

        g
    }
    pub fn start(game_mutex: &Arc<Mutex<Game>>) {
        let g_outer = Arc::clone(&game_mutex);
        let mut game = g_outer.lock().unwrap();
        game.running = true;

        let g_inner = Arc::clone(&game_mutex);
        let t = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(1));
                let mut game = g_inner.lock().unwrap();
                if !game.running {
                    break;
                }
                // handle players
                let objects = &mut (game.players);
                for object in objects {
                    object.handle_keys();
                    move_object(object);
                }
                // handle enemies
                let objects = &mut (game.enemies);
                for object in objects {
                    move_object(object);
                    if object.x > 500.0 || object.x < -500.0 {
                        match object.velocity {
                            (x,y) => {object.velocity = (-x, y);}
                        }
                    }
                }

            }
        });
        game.game_loop = Some(t);
    }
    pub fn pack_objects(&self) -> String {
        let mut objects = "".to_owned();
        // players
        for object in self.players.iter() {
            let acc = draw(object);
            objects.push_str(&acc);
        }
        // enemies
        for object in self.enemies.iter() {
            let acc = draw(object);
            objects.push_str(&acc);
        }
        objects
    }
    pub fn handle_input(&mut self, player: &String, mouse: (i32, i32), keys_down: Vec<String>) -> String {
        let player: &mut Player = match self.get(player) {
            Some(p) => p,
            None => return "".to_owned(),
        };

        player.mouse = mouse;
        player.keys_down = keys_down;

        // retrieve object data
        self.pack_objects()
    }
    pub fn get(&mut self, player: &String) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| {p.name == *player})
    }
    pub fn logout(&mut self, player: &String) -> String {
        let index = self.players.iter().position(|p| {p.name == *player});
        match index {
            Some(i) => {
                self.players.remove(i);
                format!("player {} logged out.", player)
            },
            None => {
                format!("failed to log out {}", player)
            }
        }
    }
}

