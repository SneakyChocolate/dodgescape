use std::{sync::{Arc, Mutex, MutexGuard}, thread::{self, JoinHandle}, time::Duration};

use crate::{enemy::Enemy, player::Player, vector};
use rand::prelude::*;

pub trait Drawable {
    fn get_draw_packs(&self) -> &Vec<DrawPack>;
}
#[macro_export]
macro_rules! impl_Drawable {
    ($struct_name:ident) => {
        impl Drawable for $struct_name {
            fn get_draw_packs(&self) -> &Vec<DrawPack> {
                &self.draw_packs
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
pub fn draw(position: &(f32, f32), draw_pack: &DrawPack, camera: &(f32, f32)) -> String {
    let (x, y) = position;
    let (cx, cy) = camera;
    match draw_pack.shape {
        Shape::Line { width: lw , x: lx, y: ly } => {
            format!("[(\"{}\", {:?}, ({}, {}))],",
                draw_pack.color,
                Shape::Line { x: lx - cx, y: ly - cy, width: lw },
                x + draw_pack.offset.0 - cx,
                y + draw_pack.offset.1 - cy
            )
        },
        _ => format!("[(\"{}\", {:?}, ({}, {}))],",
            draw_pack.color,
            draw_pack.shape,
            x + draw_pack.offset.0 - cx,
            y + draw_pack.offset.1 - cy
        ),
    }
}
pub fn draw_object<T: Drawable + Position>(object: &T, camera: &(f32, f32)) -> String {
    let pos = (object.x(), object.y());
    let draw_packs = object.get_draw_packs();
    let mut output = "".to_owned();
    for draw_pack in draw_packs {
        let s = draw(&pos, draw_pack, &camera);
        output.push_str(&s);
    }
    output
}
pub fn move_object<T: Moveable>(object: &mut T) {
    let (vx, vy) = object.get_velocity().clone();
    *(object.get_x()) += vx;
    *(object.get_y()) += vy;
}

pub fn distance<T: Position, B: Position>(a: &T, b: &B) -> (f32, f32, f32) {
    let a = (a.x(), a.y());
    let b = (b.x(), b.y());
    vector::distance(a, b)
}

#[derive(Debug)]
pub enum Shape {
    Circle{radius: f32},
    Rectangle{width: f32, height: f32},
    Line{width: f32, x: f32, y: f32},
    Text{content: String, size: f32},
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle { radius: 20.0 }
        // Self::Rectangle { width: 10.0, height: 20.0 }
        // Self::Line {x: 0.0, y: 0.0}
    }
}
pub struct DrawPack {
    color: String,
    shape: Shape,
    offset: (f32, f32),
}
impl DrawPack {
    pub fn new(color: &str, shape: Shape, offset: (f32, f32)) -> Self {
        Self {
            color: color.to_owned(),
            shape,
            offset,
        }
    }
}

pub struct Game {
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Vec<Enemy>,
    pub map: Vec<((f32, f32), DrawPack)>,
}

pub fn handle_players(players: &mut Vec<Player>) {
    for object in players {
        if object.alive {
            object.draw_packs.get_mut(0).unwrap().color = "blue".to_owned();
            object.handle_keys();
            move_object(object);
        }
        else {
            object.draw_packs.get_mut(0).unwrap().color = "red".to_owned();
        }
    }
}
pub fn handle_enemies(enemies: &mut Vec<Enemy>) {
    for object in enemies {
        move_object(object);
        // TODO collision over map struct
        let boarder = 2000.0;
        if object.x > boarder || object.x < -boarder {
            match object.velocity {
                (x,y) => {object.velocity = (-x, y);}
            }
        }
        if object.y > boarder || object.y < -boarder {
            match object.velocity {
                (x,y) => {object.velocity = (x, -y);}
            }
        }
    }
}
// player enemy collision
pub fn handle_collision(game: &mut MutexGuard<Game>) {
    let mut deaths: Vec<usize> = vec![];
    let mut revives: Vec<usize> = vec![];
    for (i, player) in game.players.iter().enumerate() {
        for enemy in game.enemies.iter() {
            let dd = distance(player, enemy).2;
            if dd <= (player.radius + enemy.radius) {
                deaths.push(i);
            }
        }
        // for other in game.players.iter() {
        //     if std::ptr::eq(player, other) || !other.alive {continue;}
        //     let dd = distance(player, other).2;
        //     if dd <= (player.radius + other.radius) {
        //         revives.push(i);
        //     }
        // }
    }
    for i in deaths {
        let player = game.players.get_mut(i).unwrap();
        player.alive = false;
    }
    for (i, player) in game.players.iter().enumerate() {
        for other in game.players.iter() {
            if std::ptr::eq(player, other) || !other.alive {continue;}
            let dd = distance(player, other).2;
            if dd <= (player.radius + other.radius) {
                revives.push(i);
            }
        }
    }
    for i in revives {
        let player = game.players.get_mut(i).unwrap();
        player.alive = true;
    }
}

impl Game {
    pub fn spawn_enemies(&mut self) {
        for i in 0..200 {
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-0.5..=0.5), rand::thread_rng().gen_range(-0.5..=0.5));
            self.enemies.push(Enemy::new(200.0, 100.0, velocity));
        }
    }
    pub fn spawn_grid(&mut self) {
        for i in 0..10 {
            let size = 1000.0;
            let offset = i as f32 * 100.0;
            self.map.push((
                (offset, -size),
                DrawPack::new("rgb(255,255,255,0.1)", Shape::Line { width: 5.0, x: offset, y: size }, (0.0, 0.0))
            ));
            self.map.push((
                (-offset, -size),
                DrawPack::new("rgb(255,255,255,0.1)", Shape::Line { width: 5.0, x: -offset, y: size }, (0.0, 0.0))
            ));
            self.map.push((
                (-size, offset),
                DrawPack::new("rgb(255,255,255,0.1)", Shape::Line { width: 5.0, x: size, y: offset }, (0.0, 0.0))
            ));
            self.map.push((
                (-size, -offset),
                DrawPack::new("rgb(255,255,255,0.1)", Shape::Line { width: 5.0, x: size, y: -offset }, (0.0, 0.0))
            ));
        }
    }
    pub fn new() -> Game {
        let mut g = Game {
            players: vec![],
            game_loop: None,
            running: false,
            enemies: vec![],
            map: vec![],
        };
        g.spawn_enemies();
        g.spawn_grid();

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
                handle_collision(&mut game);
            }
        });
        game.game_loop = Some(t);
    }
    pub fn pack_objects(&self, camera: (f32, f32)) -> String {
        let mut objects = "".to_owned();
        // map
        for shape in self.map.iter() {
            let acc = draw(&shape.0, &shape.1, &camera);
            objects.push_str(&acc);
        }
        // players
        for object in self.players.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > 1000.0 {continue;}
            let acc = draw_object(object, &camera);
            objects.push_str(&acc);
        }
        // enemies
        for object in self.enemies.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > 1000.0 {continue;}
            let acc = draw_object(object, &camera);
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

