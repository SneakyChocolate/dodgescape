use std::{collections::btree_map::Entry, sync::{Arc, Mutex}, thread::{self, JoinHandle, Thread}, time::Duration};

use crate::{enemy::Enemy, player::Player};
use rand::prelude::*;

pub trait Drawable {
    fn get_shapes(&self) -> &Vec<(String, Shape, (f32,f32))>;
}
#[macro_export]
macro_rules! impl_Drawable {
    ($struct_name:ident) => {
        impl Drawable for $struct_name {
            fn get_shapes(&self) -> &Vec<(String, Shape, (f32,f32))> {
                &self.shapes
            }
        }
    };
}
pub trait Position {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}
#[macro_export]
macro_rules! impl_Position {
    ($struct_name:ident) => {
        impl Position for $struct_name {
            fn x(&self) -> f32 {
                self.x
            }
            fn y(&self) -> f32 {
                self.y
            }
        }
    };
}
pub trait Moveable {
    fn get_x(&mut self) -> &mut f32;
    fn get_y(&mut self) -> &mut f32;
    fn get_velocity(&self) -> &(f32, f32);
}
#[macro_export]
macro_rules! impl_Movable {
    ($struct_name:ident) => {
        impl Moveable for $struct_name {
            fn get_x(&mut self) -> &mut f32 {
                &mut self.x
            }
            fn get_y(&mut self) -> &mut f32 {
                &mut self.y
            }
            fn get_velocity(&self) -> &(f32, f32) {
                &self.velocity
            }
        }
    };
}

pub fn draw<T: Drawable + Position>(object: &T, camera: (f32, f32)) -> String {
    let (cx, cy) = camera;
    let x = object.x();
    let y = object.y();
    let shapes = object.get_shapes();
    let mut output = "".to_owned();
    for shape in shapes {
        let (color, shape, (sx, sy)) = shape;
        let s = match shape {
            Shape::Line { x: lx, y: ly } => {
                format!("[{:?}],", (color, Shape::Line { x: lx - cx, y: ly - cy }, (x + sx - cx, y + sy - cy)))
            },
            _ => format!("[{:?}],", (color, shape, (x + sx - cx, y + sy - cy))),
        };
        output.push_str(&s);
    }
    output
}
pub fn move_object<T: Moveable + std::fmt::Debug>(object: &mut T) {
    let (vx, vy) = object.get_velocity().clone();
    *(object.get_x()) += vx;
    *(object.get_y()) += vy;
    // println!("{}, {}", object.get_x().clone(), object.get_y().clone());
}

pub fn distance<T: Position, B: Position>(a: &T, b: &B) -> (f32, f32, f32) {
    let ax = a.x();
    let ay = a.y();
    let bx = b.x();
    let by = b.y();

    let dx = bx - ax;
    let dy = by - ay;

    (dx, dy, f32::sqrt(f32::powi(dx, 2) + f32::powi(dy, 2)))
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

pub fn handle_players(players: &mut Vec<Player>) {
    for object in players {
        if object.alive {
            object.shapes.get_mut(0).unwrap().0 = "blue".to_owned();
            object.handle_keys();
            move_object(object);
        }
        else {
            object.shapes.get_mut(0).unwrap().0 = "red".to_owned();
        }
    }
}
pub fn handle_enemies(enemies: &mut Vec<Enemy>) {
    for object in enemies {
        move_object(object);
        // TODO collision over map struct
        if object.x > 3000.0 || object.x < -3000.0 {
            match object.velocity {
                (x,y) => {object.velocity = (-x, y);}
            }
        }
        if object.y > 3000.0 || object.y < -3000.0 {
            match object.velocity {
                (x,y) => {object.velocity = (x, -y);}
            }
        }
    }
}

impl Game {
    pub fn new() -> Game {
        let mut g = Game {
            players: vec![],
            game_loop: None,
            running: false,
            enemies: vec![],
        };
        for i in 0..300 {
            let velocity: (f32, f32) = (rand::random::<f32>(), rand::random::<f32>());
            g.enemies.push(Enemy::new(200.0, 100.0, velocity));
        }

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

                handle_players(&mut game.players);
                handle_enemies(&mut game.enemies);

                // collisions
                // enemy kill
                let mut collisions: Vec<usize> = vec![];
                for (i, player) in game.players.iter().enumerate() {
                    for enemy in game.enemies.iter() {
                        // when colliding then change player.alive = false;
                        let (dx, dy, dd) = distance(player, enemy);
                        if dd <= (player.radius + enemy.radius) {
                            collisions.push(i);
                        }
                    }
                }
                for i in collisions {
                    let player = game.players.get_mut(i).unwrap();
                    player.alive = false;
                }
                // team revive
                let mut collisions: Vec<usize> = vec![];
                for (i, player) in game.players.iter().enumerate() {
                    for other in game.players.iter() {
                        if std::ptr::eq(player, other) || !other.alive {continue;}
                        // when colliding then change player.alive = false;
                        let (dx, dy, dd) = distance(player, other);
                        if dd <= (player.radius + other.radius) {
                            collisions.push(i);
                        }
                    }
                }
                for i in collisions {
                    let player = game.players.get_mut(i).unwrap();
                    player.alive = true;
                }
            }
        });
        game.game_loop = Some(t);
    }
    pub fn pack_objects(&self, camera: (f32, f32)) -> String {
        let mut objects = "".to_owned();
        // players
        for object in self.players.iter() {
            let acc = draw(object, camera);
            objects.push_str(&acc);
        }
        // enemies
        for object in self.enemies.iter() {
            let acc = draw(object, camera);
            objects.push_str(&acc);
        }
        objects
    }
    pub fn handle_input(&mut self, player: &String, mouse: (i32, i32), keys_down: Vec<String>) -> String {
        let player: &mut Player = match self.get(player) {
            Some(p) => p,
            None => return "".to_owned(),
        };

        let camera = (player.x.clone(), player.y.clone());

        player.mouse = mouse;
        player.keys_down = keys_down;

        // retrieve object data
        self.pack_objects(camera)
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

