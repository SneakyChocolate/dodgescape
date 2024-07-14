use std::{sync::{Arc, Mutex, MutexGuard}, thread::{self, JoinHandle}, time::Duration};

use crate::{enemy::Enemy, player::Player, vector, wall::Wall};
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
    Poly{corners: Vec<(f32,f32)>},
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

#[derive(Default)]
pub struct Game {
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Vec<Enemy>,
    pub map: Vec<((f32, f32), DrawPack)>,
    pub walls: Vec<Wall>,
}

pub fn handle_players(players: &mut Vec<Player>) {
    for object in players {
        if object.alive {
            object.draw_packs[0].color = "blue".to_owned();
            object.handle_keys();
        }
        else {
            object.draw_packs[0].color = "red".to_owned();
        }
    }
}
pub fn handle_enemies(enemies: &mut Vec<Enemy>) {
    for object in enemies {
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
pub fn handle_kill_revive(game: &mut MutexGuard<Game>) {
    let mut deaths: Vec<usize> = vec![];
    let mut revives: Vec<usize> = vec![];
    // handle deaths
    for (i, player) in game.players.iter().enumerate() {
        for enemy in game.enemies.iter() {
            let dd = distance(player, enemy).2;
            if dd <= (player.radius + enemy.radius) {
                deaths.push(i);
            }
        }
    }
    for i in deaths {
        let player = game.players.get_mut(i).unwrap();
        player.alive = false;
    }

    // handle revives later so new deaths are accounted
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
pub fn handle_collision(game: &mut MutexGuard<Game>) {
    let mut enemy_collisions: Vec<(usize, (f32, f32))> = vec![];
    let mut player_collisions: Vec<(usize, (f32, f32))> = vec![];
    for wall in game.walls.iter() {
        // enemies
        if wall.enemy {
            for (i, enemy) in game.enemies.iter().enumerate() {
                let cp = wall.get_nearest_point(&(enemy.x, enemy.y));
                if vector::distance(cp, (enemy.x, enemy.y)).2 <= enemy.radius {
                    if enemy_collisions.iter().any(|(e, _)| {*e == i}) {
                        continue;
                    }
                    enemy_collisions.push((i, cp));
                }
            }
        }
        // players
        if wall.player {
            for (i, player) in game.players.iter().enumerate() {
                let cp = wall.get_nearest_point(&(player.x, player.y));
                if vector::distance(cp, (player.x, player.y)).2 <= player.radius {
                    if player_collisions.iter().any(|(e, _)| {*e == i}) {
                        continue;
                    }
                    player_collisions.push((i, cp));
                }
            }
        }
    }
    // offset for pushing object away on collision so collision doesnt trigger again
    const OFFSET: f32 = 0.001;
    for (i, cp) in enemy_collisions {
        let enemy = game.enemies.get_mut(i).unwrap();
        let speed = vector::abs(enemy.velocity);
        let dist = vector::distance(cp, (enemy.x, enemy.y));
        let push = vector::normalize((dist.0, dist.1), enemy.radius + OFFSET);
        enemy.x = cp.0 + push.0;
        enemy.y = cp.1 + push.1;
        let new_v = vector::normalize(vector::collision((enemy.x, enemy.y), enemy.velocity, cp), speed);
        enemy.velocity = new_v;
    }
    for (i, cp) in player_collisions {
        let player = game.players.get_mut(i).unwrap();
        let speed = vector::abs(player.velocity);
        let dist = vector::distance(cp, (player.x, player.y));
        let push = vector::normalize((dist.0, dist.1), player.radius + OFFSET);
        player.x = cp.0 + push.0;
        player.y = cp.1 + push.1;
        let new_v = vector::normalize(vector::collision((player.x, player.y), player.velocity, cp), speed);
        player.velocity = new_v;
        player.skip_move = true;
    }
}
pub fn handle_movements(game: &mut MutexGuard<Game>) {
    for object in &mut game.players {
        if object.alive && !object.skip_move {
            move_object(object);
        }
        else {
            object.skip_move = false;
        }
    }
    for object in &mut game.enemies {
        move_object(object);
    }
}

impl Game {
    pub fn spawn_enemies(&mut self) {
        for i in 0..200 {
            let cap = 0.5;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            self.enemies.push(Enemy::new(0.0, 1000.0, velocity, rand::thread_rng().gen_range(10.0..=50.0)));
        }
    }
    pub fn spawn_grid(&mut self) {
        for i in 0..20 {
            let size = 2000.0;
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
    pub fn spawn_walls(&mut self) {
        self.walls.push(Wall::new((-200.0, -200.0), (200.0, -200.0), false, true));
        self.walls.push(Wall::new((-200.0, 200.0), (200.0, 200.0), true, true));
        self.walls.push(Wall::new((200.0, 200.0), (200.0, -200.0), true, true));
        self.walls.push(Wall::new((-200.0, 200.0), (-200.0, -200.0), true, true));
    }
    pub fn new() -> Game {
        let mut g = Game {
            game_loop: None,
            running: false,
            ..Default::default()
        };
        g.spawn_enemies();
        g.spawn_grid();
        g.spawn_walls();

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
                handle_kill_revive(&mut game);
                handle_collision(&mut game);
                handle_movements(&mut game);
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
        // walls
        for wall in self.walls.iter() {
            let draw_pack = DrawPack::new("green", Shape::Line { width: 5.0, x: wall.b.0, y: wall.b.1 }, (0.0, 0.0));
            let acc = draw(&wall.a, &draw_pack, &camera);
            objects.push_str(&acc);
        }

        // // wall colliders
        // for wall in self.walls.iter() {
        //     for player in self.players.iter() {
        //         let start = (player.x, player.y);
        //         let target = wall.get_nearest_point(&start);
        //         let draw_pack = DrawPack::new("blue", Shape::Line { width: 5.0, x: target.0, y: target.1 }, (0.0, 0.0));
        //         let acc = draw(&start, &draw_pack, &camera);
        //         objects.push_str(&acc);
        //     }
        // }

        const VIEW: f32 = 1000.0;
        // players
        for object in self.players.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > VIEW {continue;}
            let acc = draw_object(object, &camera);
            objects.push_str(&acc);
        }
        // enemies
        for object in self.enemies.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > VIEW {continue;}
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

