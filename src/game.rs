use std::{sync::{mpsc::{Receiver, Sender}, Arc, Mutex, MutexGuard}, thread::{self, JoinHandle}, time::Duration};

use crate::{action::Action, collectable::Collectable, enemy::{EnemyEffect, Enemy}, gametraits::{Drawable, Moveable, Position}, item::Item, player::Player, server::ServerMessage, vector, wall::Wall};
use rand::prelude::*;

pub fn draw(position: &(f32, f32), draw_pack: &DrawPack, camera: &(f32, f32), zoom: f32) -> String {
    let (x, y) = position;
    let (cx, cy) = camera;
    let shape = match &draw_pack.shape {
        Shape::Line { width: lw , x: lx, y: ly } => {
            Shape::Line { x: (lx - cx) * zoom, y: (ly - cy) * zoom, width: *lw * zoom }
        },
        Shape::Poly { corners: c } => {
            let changed = c.iter().map(|(corx, cory)| { (
                (corx - cx + x + draw_pack.offset.0) * zoom,
                (cory - cy + y + draw_pack.offset.1) * zoom
            )}).collect::<Vec<(f32, f32)>>();
            Shape::Poly { corners: changed }
        },
        Shape::Circle { radius } => {
            Shape::Circle { radius: radius * zoom }
        },
        Shape::Text { content, size } => {
            Shape::Text { content: content.clone(), size: size * zoom }
        },
        Shape::Rectangle { width, height } => {
            Shape::Rectangle { width: width * zoom, height: height * zoom }
        },
    };
    format!("[(\"{}\", {:?}, ({}, {}))],",
        draw_pack.color,
        shape,
        (x + draw_pack.offset.0 - cx) * zoom,
        (y + draw_pack.offset.1 - cy) * zoom
    )
}
pub fn draw_object<T: Drawable + Position>(object: &T, camera: &(f32, f32), zoom: f32) -> String {
    let pos = (object.x(), object.y());
    let draw_packs = object.get_draw_packs();
    let mut output = "".to_owned();
    for draw_pack in draw_packs {
        let s = draw(&pos, draw_pack, &camera, zoom);
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

#[derive(Debug, Clone)]
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

pub struct Game {
    pub receiver: Receiver<ServerMessage>,
    pub sender: Sender<String>,
    
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Vec<(Vec<usize>, Vec<Enemy>)>,
    pub grid: Vec<((f32, f32), DrawPack, bool)>,
    pub map: Vec<((f32, f32), DrawPack)>,
    pub walls: Vec<(usize, Vec<Wall>)>,
    pub collectables: Vec<Collectable>,
}

pub fn handle_players(players: &mut Vec<Player>) {
    for object in players {
        object.handle_keys();
        if object.alive {
            object.draw_packs[0].color = "blue".to_owned();
        }
        else {
            object.draw_packs[0].color = "red".to_owned();
        }
    }
}
pub fn handle_collectables(game: &mut Game) {
    let mut rems: Vec<usize> = vec![];
    for p in game.players.iter_mut() {
        for (i, c) in game.collectables.iter_mut().enumerate() {
            let dist = distance(p, c);
            if dist.2 <= p.radius + c.radius {
                c.collect(p);
                rems.push(i);
            }
        }
    }
    for r in rems.iter().rev() {
        game.collectables.remove(*r);
    }
}
// player enemy collision
pub fn handle_kill_revive(game: &mut Game) {
    let mut deaths: Vec<usize> = vec![];
    let mut revives: Vec<usize> = vec![];
    // handle deaths
    for (i, player) in game.players.iter().enumerate() {
        for group in game.enemies.iter() {
            for enemy in group.1.iter() {
                let dd = distance(player, enemy).2;
                if dd <= (player.radius + enemy.radius) {
                    deaths.push(i);
                }
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

pub fn handle_collision(game: &mut Game) {
    for group in game.enemies.iter_mut() {
        for enemy in group.1.iter_mut() {
            enemy.just_collided = false;
        }
    }
    let mut enemy_collisions: Vec<(usize, usize, (f32, f32))> = vec![];
    let mut player_collisions: Vec<(usize, (f32, f32))> = vec![];
    for wgroup in game.walls.iter() {
        for wall in wgroup.1.iter() {
            // enemies
            if wall.enemy {
                for (g, egroup) in game.enemies.iter().enumerate() {
                    if !egroup.0.contains(&wgroup.0) {continue;} 
                    for (i, enemy) in egroup.1.iter().enumerate() {
                        let cp = wall.get_nearest_point(&(enemy.x, enemy.y));
                        if vector::distance(cp, (enemy.x, enemy.y)).2 <= enemy.radius {
                            if enemy_collisions.iter().any(|(_, e, _)| {*e == i}) {
                                continue;
                            }
                            enemy_collisions.push((g, i, cp));
                        }
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
    }
    // offset for pushing object away on collision so collision doesnt trigger again
    const OFFSET: f32 = 0.001;
    for (g, i, cp) in enemy_collisions {
        let enemy = game.enemies.get_mut(g).unwrap().1.get_mut(i).unwrap();
        let speed = vector::abs(enemy.velocity);
        let dist = vector::distance(cp, (enemy.x, enemy.y));
        let push = vector::normalize((dist.0, dist.1), enemy.radius + OFFSET);
        enemy.x = cp.0 + push.0;
        enemy.y = cp.1 + push.1;
        let new_v = vector::normalize(vector::collision((enemy.x, enemy.y), enemy.velocity, cp), speed);
        enemy.velocity = new_v;
        enemy.just_collided = true;
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
pub fn handle_movements(game: &mut Game) {
    for object in &mut game.players {
        if object.alive && !object.skip_move {
            move_object(object);
        }
        else {
            object.skip_move = false;
        }
    }
    for group in game.enemies.iter_mut() {
        for object in group.1.iter_mut() {
            move_object(object);
        }
    }
}

impl Game {
    fn spawn_dirt_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![0, 5];
        let mut enemies = vec![];
        for _ in 0..100 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(1500.0, 1000.0, velocity, rand::thread_rng().gen_range(10.0..=50.0), "rgb(50,40,20)");
            enemy.effects.push(EnemyEffect::Crumble);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_wind_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![1, 5];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 1.0 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, 1000.0, velocity, rand::thread_rng().gen_range(40.0..=100.0), "rgb(200,200,255)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,255,0.1)", Shape::Circle { radius: enemy.radius * 3.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 3.0, power: 5.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_plant_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![2, 5];
        let mut enemies = vec![];
        for _ in 0..200 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, -1000.0, velocity, rand::thread_rng().gen_range(10.0..=30.0), "rgb(255,250,5)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,255,0.2)", Shape::Circle { radius: enemy.radius * 5.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Chase { radius: enemy.radius * 5.0, power: 0.2});
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_water_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![3,5];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(2000.0, -2000.0, velocity, rand::thread_rng().gen_range(50.0..=100.0), "rgb(50,50,200)");
            enemies.push(enemy);
        }
        for _ in 0..5 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(3000.0, -3000.0, velocity, rand::thread_rng().gen_range(400.0..=600.0), "rgb(10,10,100)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(10,10,100,0.5)", Shape::Circle { radius: enemy.radius * 1.3 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 1.3, power: -2.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_fire_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let size = 20.0..=50.0;
        let amount = 200;
        let speed = 1.0;
        let dist = 4500.0;
        let ids = vec![0,1,2,3,4,5];
        let mut enemies = vec![];
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(0.0, -dist, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(0.0, dist, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(-dist, 0.0, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(dist, 0.0, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_space_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let size = 200.0..=500.0;
        let amount = 30;
        let speed = 2.0;
        let dist = 15000.0;
        let ids = vec![4,6,7,8,9];
        let color = "black";
        let auracolor = "rgba(0,0,0,0.2)";
        let mut enemies = vec![];
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: enemy.radius * 2.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 2.0, power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: enemy.radius * 2.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 2.0, power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: enemy.radius * 2.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 2.0, power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: enemy.radius * 2.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: enemy.radius * 2.0, power: -6.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_tech_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![7];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=30.0), "rgb(25,25,25)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,0,0.02)", Shape::Circle { radius: enemy.radius * 30.0 }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: enemy.radius * 30.0, speed: 10.0, cooldown: 60, time_left: 0, lifetime: 1000, projectile_radius: 20.0, color: "black".to_owned() });
            enemy.view_radius = enemy.radius * 30.0;
            enemies.push(enemy);
        }
        for _ in 0..50 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=70.0), "rgb(255,125,125)");
            let r = enemy.radius * 5.0;
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,0,0.1)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Slow { radius: r, power: 0.5 });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_snake_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![8];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(20000.0, 0.0, velocity, 90.0, "rgb(25,25,25)");
            let r = enemy.radius * 20.0;
            enemy.draw_packs.insert(0, DrawPack::new("rgba(0,255,255,0.02)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: r, speed: 10.0, cooldown: 5, time_left: 0, lifetime: 50, projectile_radius: 40.0, color: "rgb(0,0,50)".to_owned() });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_bomb_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![9];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.1 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, 20000.0, velocity, 90.0, "rgb(25,25,25)");
            let cd = rand::thread_rng().gen_range(200..=500);
            enemy.effects.push(EnemyEffect::Explode { lifetime: 300, radius: (10.0, 30.0), speed: 15.0, time_left: 0, cooldown: cd, color: "rgb(255,255,0)".to_owned(), amount: 10 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    pub fn spawn_enemies(&mut self) {
        let spawn_m = 3;
        let speed_m = 9.0;
        self.spawn_dirt_enemies(speed_m, spawn_m);
        self.spawn_wind_enemies(speed_m, spawn_m);
        self.spawn_plant_enemies(speed_m, spawn_m);
        self.spawn_water_enemies(speed_m, spawn_m);
        self.spawn_fire_enemies(speed_m, spawn_m);
        self.spawn_space_enemies(speed_m, spawn_m);
        self.spawn_tech_enemies(speed_m, spawn_m);
        self.spawn_snake_enemies(speed_m, spawn_m);
        self.spawn_bomb_enemies(speed_m, spawn_m);
    }
    pub fn spawn_area(&mut self, corners: Vec<(f32, f32)>, color: &str, id: usize) {
        let start = (0.0, 0.0);
        for c in 0..corners.len() {
            let a = corners[c];
            let b = if c + 1 == corners.len() {
                corners[0]
            }
            else {
                corners[c + 1]
            };
            let addition = Wall::new(a, b, false, true);
            let group = self.walls.iter_mut().find(|(i, _)| {*i == id});
            match group {
                Some(g) => {
                    g.1.push(addition);
                },
                None => {
                    self.walls.push((id, vec![addition]));
                },
            }
        }
        let poly = Shape::Poly { corners };
        let draw_pack = DrawPack::new(color, poly, (0.0, 0.0));
        self.map.push((start, draw_pack));
    }
    pub fn spawn_grid(&mut self, size: f32, color: &str) {
        for i in 0..(size as i32 / 100) {
            let offset = i as f32 * 100.0;
            self.grid.push((
                (offset, -size),
                DrawPack::new(color, Shape::Line { width: 5.0, x: offset, y: size }, (0.0, 0.0)),
                true
            ));
            self.grid.push((
                (-offset, -size),
                DrawPack::new(color, Shape::Line { width: 5.0, x: -offset, y: size }, (0.0, 0.0)),
                true
            ));
            self.grid.push((
                (-size, offset),
                DrawPack::new(color, Shape::Line { width: 5.0, x: size, y: offset }, (0.0, 0.0)),
                false
            ));
            self.grid.push((
                (-size, -offset),
                DrawPack::new(color, Shape::Line { width: 5.0, x: size, y: -offset }, (0.0, 0.0)),
                false
            ));
        }
    }
    pub fn spawn_map(&mut self) {
        let multiplier = 2000.0;
        
        // space area
        let corners = vec![(-15.0,0.0),(-10.0,10.0),(0.0,15.0),(10.0,10.0),(15.0,0.0),(10.0,-10.0),(0.0,-15.0),(-10.0,-10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(20,0,30)", 6);
        // fire area
        let corners = vec![(-5.0,-5.0),(5.0,-5.0),(5.0,5.0),(-5.0,5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(50,20,30)", 4);

        // dirt area
        let corners = vec![(0.0,0.0),(3.0,1.0),(4.0,4.0),(1.0,3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(80,70,50)", 0);
        // wind area
        let corners = vec![(0.0,0.0),(-3.0,1.0),(-4.0,4.0),(-1.0,3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(100,100,150)", 1);
        // plant area
        let corners = vec![(0.0,0.0),(-3.0,-1.0),(-4.0,-4.0),(-1.0,-3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(10,50,20)", 2);
        // water area
        let corners = vec![(0.0,0.0),(3.0,-1.0),(4.0,-4.0),(1.0,-3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(0,0,50)", 3);

        // tech area
        let corners = vec![(-5.0,-3.0),(-5.0,3.0),(-10.0,2.0),(-12.0,1.0),(-12.5,0.0),(-12.0,-1.0),(-10.0,-2.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(50,50,50)", 7);
        // snake area
        let corners = vec![(5.0,-3.0),(5.0,3.0),(10.0,2.0),(12.0,1.0),(12.5,0.0),(12.0,-1.0),(10.0,-2.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(40,50,40)", 8);
        // bomb area
        let corners = vec![(-3.0,5.0),(3.0,5.0),(2.0,10.0),(1.0,12.0),(0.0,12.5),(-1.0,12.0),(-2.0,10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(90,70,50)", 9);
        
        // spawn area
        let corners = vec![(-0.4,0.0),(0.0,0.4),(0.4,0.0),(0.0,-0.4)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "black", 5);
        
        // grid
        self.spawn_grid(30000.0, "rgb(255,255,255,0.05)");
    }
    pub fn spawn_collectables(&mut self) {
        let c = Collectable::new(100.0, 0.0, "rgb(200,200,0)", vec![Item::new("binoculars", 1)]);
        self.collectables.push(c);
        let c = Collectable::new(-100.0, 0.0, "rgb(200,200,0)", vec![Item::new("telescope", 1)]);
        self.collectables.push(c);
    }
    pub fn new(sender: Sender<String>, receiver: Receiver<ServerMessage>) -> Game {
        let mut g = Game {
            game_loop: None,
            running: false,
            sender,
            receiver,
            players: Default::default(),
            enemies: Default::default(),
            grid: Default::default(),
            map: Default::default(),
            walls: Default::default(),
            collectables: Default::default(),
        };
        g.spawn_enemies();
        g.spawn_map();
        g.spawn_collectables();

        g
    }
    pub fn start(mut self) {
        self.running = true;
        let t = thread::spawn(move || {
            loop {
                match self.receiver.try_recv() {
                    Ok(message) => {
                        match message {
                            ServerMessage::Login(name) => {
                                self.players.push(Player::new(&name));
                                self.sender.send("".to_owned()).unwrap();
                            },
                            ServerMessage::Logout(name) => {
                                let output = self.logout(&name);
                                self.sender.send(output).unwrap();
                            },
                            ServerMessage::Input { name, mouse, keys, wheel } => {
                                let output = self.handle_input(&name, mouse, keys, wheel);
                                self.sender.send(output).unwrap();
                            },
                        }
                    },
                    Err(_) => {},
                }
                thread::sleep(Duration::from_millis(1));
                if !self.running {
                    break;
                }

                handle_players(&mut self.players);
                crate::enemy::handle_effects(&mut self);
                handle_collision(&mut self);
                handle_kill_revive(&mut self);
                handle_collectables(&mut self);
                handle_movements(&mut self);
            }
        });
    }
    pub fn pack_objects(&mut self, camera: (f32, f32), name: &String, zoom: f32) -> String {
        let view = 1000.0 / zoom;
        let mut objects = "".to_owned();
        // map
        for shape in self.map.iter() {
            let acc = draw(&shape.0, &shape.1, &camera, zoom);
            objects.push_str(&acc);
        }
        for (position, drawpack, xory) in self.grid.iter() {
            let dist = vector::distance(*position, camera);
            if *xory {
                if dist.0.abs() > view {continue;}
            }
            else {
                if dist.1.abs() > view {continue;}
            }
            let acc = draw(&position, &drawpack, &camera, zoom);
            objects.push_str(&acc);
        }

        // walls
        // for wall in self.walls.iter() {
        //     let draw_pack = DrawPack::new("green", Shape::Line { width: 5.0, x: wall.b.0, y: wall.b.1 }, (0.0, 0.0));
        //     let acc = draw(&wall.a, &draw_pack, &camera, zoom);
        //     objects.push_str(&acc);
        // }

        // collectables
        for object in self.collectables.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > view {continue;}
            let acc = draw_object(object, &camera, zoom);
            objects.push_str(&acc);
        }
        // players
        for object in self.players.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > view {continue;}
            let acc = draw_object(object, &camera, zoom);
            objects.push_str(&acc);
        }
        // enemies
        for group in self.enemies.iter() {
            for object in group.1.iter() {
                if vector::distance(camera, (object.x, object.y)).2 - object.view_radius > view {continue;}
                let acc = draw_object(object, &camera, zoom);
                objects.push_str(&acc);
            }
        }
        // inventory
        for object in self.players.iter() {
            if *name == *object.name && object.inventory.open {
                let drawpack = DrawPack::new("rgba(200,100,50,0.8)", Shape::Rectangle { width: 400.0, height: 800.0 }, (-900.0, -400.0));
                let acc = draw(&(object.x, object.y), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                let drawpack = DrawPack::new("white", Shape::Text { content: "Inventory".to_owned(), size: 30.0 }, (-850.0, -350.0));
                let acc = draw(&(object.x, object.y), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                for (i, item) in object.inventory.items.iter().enumerate() {
                    let drawpack = DrawPack::new("black", Shape::Text { content: item.name.clone(), size: 30.0 }, (-850.0, -300.0 + 50.0 * (i as f32)));
                    let acc = draw(&(object.x, object.y), &drawpack, &camera, 1.0);
                    objects.push_str(&acc);
                }
            }
        }

        objects
    }
    pub fn handle_input(&mut self, player_name: &String, mouse: (f32, f32), keys_down: Vec<String>, wheel: i32) -> String {
        let player = match self.get_mut(player_name) {
            Some(p) => p,
            None => return "".to_owned(),
        };
        player.mouse = mouse;
        player.keys_down = keys_down;
        if wheel > 0 {
            player.zoom /= 1.1;
        }
        else if wheel < 0 {
            player.zoom *= 1.1;
        }
        if player.zoom > player.zoomlimit.1 {
            player.zoom = player.zoomlimit.1;
        }
        else if player.zoom < player.zoomlimit.0 {
            player.zoom = player.zoomlimit.0;
        }

        // retrieve object data
        let camera = (player.x, player.y);
        let zoom = player.zoom;
        self.pack_objects(camera, &player_name, zoom)
    }
    pub fn get_mut(&mut self, player: &String) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| {p.name == *player})
    }
    pub fn get(&mut self, player: &String) -> Option<&Player> {
        self.players.iter().find(|p| {p.name == *player})
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

