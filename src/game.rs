use std::{sync::mpsc::{Receiver, Sender}, thread::{self, JoinHandle}, time::Duration};

use crate::{collectable::Collectable, color::Color, enemy::{Enemy, EnemyEffect}, gametraits::{Drawable, Moveable, Position, Radius}, item::{Item, ItemEffect}, player::Player, server::ServerMessage, vector::{self, point_from_angle, random_point}, wall::{Wall, WallType}};
use rand::prelude::*;
use serde::Serialize;

pub fn draw(radius: f32, position: &(f32, f32), draw_pack: &DrawPack, camera: &(f32, f32), zoom: f32) -> String {
    format!("{{\"radius\":{},\"position\":{{\"x\":{},\"y\":{}}},\"draw_pack\":{},\"camera\":{{\"x\":{},\"y\":{}}},\"zoom\":{}}},",
        radius,
        position.0,
        position.1,
        serde_json::to_string(&draw_pack).unwrap(),
        camera.0,
        camera.1,
        zoom,
    )
}
pub fn draw_object<T: Drawable + Position>(object: &T, camera: &(f32, f32), zoom: f32) -> String {
    let pos = (object.x(), object.y());
    let draw_packs = object.get_draw_packs();
    let mut output = "".to_owned();
    for draw_pack in draw_packs {
        let s = draw(object.get_radius(), &pos, draw_pack, &camera, zoom);
        output.push_str(&s);
    }
    output
}
pub fn move_object<T: Moveable>(object: &mut T) {
    let (vx, vy) = object.get_velocity().clone();
    *(object.get_x()) += vx * *object.get_speed_multiplier();
    *(object.get_y()) += vy * *object.get_speed_multiplier();
}

pub fn distance<T: Position, B: Position>(a: &T, b: &B) -> (f32, f32, f32) {
    let a = (a.x(), a.y());
    let b = (b.x(), b.y());
    vector::distance(a, b)
}

#[derive(Serialize, Debug, Clone)]
pub enum Shape {
    Circle{radius: Radius},
    Rectangle{width: f32, height: f32},
    Line{width: f32, x: f32, y: f32},
    Text{content: String, size: f32},
    Poly{corners: Vec<(f32,f32)>},
    Image{keyword: String, scale: f32},
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle { radius: Radius::Relative(1.0) }
    }
}
#[derive(Serialize, Debug, Clone)]
pub struct DrawPack {
    pub color: String,
    pub shape: Shape,
    pub offset: (f32, f32),
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
    
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Vec<(Vec<WallType>, Vec<Enemy>)>,
    pub grid: Vec<((f32, f32), DrawPack, bool)>,
    pub map: Vec<((f32, f32), DrawPack)>,
    pub walls: Vec<(WallType, Vec<Wall>)>,
    pub collectables: Vec<Collectable>,
}

pub fn handle_players(players: &mut Vec<Player>, collectables: &mut Vec<Collectable>) {
    for object in players {
        object.handle_keys(collectables);
        if object.alive {
            object.draw_packs[0].color = object.color.clone();
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
            if dist.2 <= p.get_radius() + c.radius {
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
                if enemy.harmless || player.invincible {
                    continue;
                }
                let dd = distance(player, enemy).2;
                if dd <= (player.get_radius() + enemy.get_radius()) {
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
            if dd <= (player.get_radius() + other.get_radius()) {
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
                        if vector::distance(cp, (enemy.x, enemy.y)).2 <= enemy.get_radius() {
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
                    if vector::distance(cp, (player.x, player.y)).2 <= player.get_radius() {
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
        let push = vector::normalize((dist.0, dist.1), enemy.get_radius() + OFFSET);
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
        let push = vector::normalize((dist.0, dist.1), player.get_radius() + OFFSET);
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
        let ids = vec![WallType::Dirt, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..120 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(1500.0, 1000.0, velocity, rand::thread_rng().gen_range(10.0..=50.0), "rgb(50,40,20)");
            enemy.effects.push(EnemyEffect::Crumble);
            enemy.effects.push(EnemyEffect::ShrinkPlayers { radius: Radius::Relative(10.0), shrink: 0.9, duration: 1 });
            enemy.draw_packs.push(DrawPack::new("rgba(50,40,20,0.2)", Shape::Circle { radius: Radius::Relative(10.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(10.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_wind_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Wind, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..40 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, 1000.0, velocity, rand::thread_rng().gen_range(40.0..=100.0), "rgb(200,200,255)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,255,0.1)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(3.0), power: 5.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_flower_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Flower, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..150 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, -1000.0, velocity, rand::thread_rng().gen_range(10.0..=30.0), "rgb(255,250,5)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,255,0.2)", Shape::Circle { radius: Radius::Relative(5.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Chase { radius: Radius::Relative(5.0), power: 0.2});
            // enemy.harmless = true;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_water_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Water, WallType::SpawnA];
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
            let radius = rand::thread_rng().gen_range(400.0..=600.0);
            let mut enemy = Enemy::new(3000.0, -3000.0, velocity, radius, "rgb(10,10,100)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(10,10,100,0.5)", Shape::Circle { radius: Radius::Relative(1.3) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(1.3), power: -2.0 });
            // enemy.draw_packs.push(DrawPack::new("", Shape::Image { keyword: "candytop".to_owned(), scale: radius / 300.0 }, (-radius / 1.2, -radius / 1.2)));
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_fire_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let size = 20.0..=50.0;
        let amount = 150;
        let speed = 1.0;
        let dist = 4500.0;
        let ids = vec![WallType::Dirt,WallType::Wind,WallType::Flower,WallType::Water,WallType::Fire,WallType::SpawnA,WallType::SpawnB];
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
        let amount = 50;
        let speed = 2.0;
        let dist = 15000.0;
        let ids = vec![
            WallType::Fire,
            WallType::Shooting,
            WallType::Explosion,
            WallType::Snake,
            WallType::Ice,
            WallType::Blackhole,
            WallType::SpawnB,
            WallType::Poison,
            WallType::Hell,
            WallType::Candy,
            WallType::Lightning,
        ];
        let color = "black";
        let auracolor = "rgba(0,0,0,0.2)";
        let mut enemies = vec![];
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_tech_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Shooting];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=30.0), "rgb(25,25,25)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,0,0.02)", Shape::Circle { radius: Radius::Relative(30.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: Radius::Relative(30.0), speed: 10.0, cooldown: 60, time_left: 0, lifetime: 1000, projectile_radius: 20.0, color: "black".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = Radius::Relative(30.0);
            enemies.push(enemy);
        }
        for _ in 0..50 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=70.0), "rgb(255,125,125)");
            let r = Radius::Relative(5.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,0,0.1)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: r, slow: 0.5, duration: 1 });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_ice_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Ice];
        let mut enemies = vec![];
        // snowballs
        for _ in 0..25 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, -20000.0, velocity, rand::thread_rng().gen_range(50.0..=70.0), "rgb(255,255,255)");
            let r = Radius::Relative(2.0);
            enemy.effects.push(EnemyEffect::Grow { size: 0.2, maxsize: 10.0 * enemy.get_radius(), defaultsize: enemy.get_radius() });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        // snowmans
        for _ in 0..25 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, -20000.0, velocity, rand::thread_rng().gen_range(50.0..=70.0), "rgb(255,255,255)");
            enemy.draw_packs.push(DrawPack::new("rgb(255,150,0)", Shape::Circle { radius: Radius::Relative(0.8)}, (0.0, 0.0)));
            enemy.draw_packs.push(DrawPack::new("rgb(0,0,0)", Shape::Circle { radius: Radius::Relative(0.1)}, (-10.0, -10.0)));
            enemy.draw_packs.push(DrawPack::new("rgb(0,0,0)", Shape::Circle { radius: Radius::Relative(0.1)}, (10.0, -10.0)));
            let r = Radius::Relative(10.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(0,0,255,0.05)", Shape::Circle { radius: Radius::Relative(10.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Chase { radius: r, power: 0.03 });
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: r, slow: 0.8, duration: 1 });
            enemy.effects.push(EnemyEffect::Shoot { lifetime: 200, radius: r, projectile_radius: 20.0, speed: 8.0, time_left: 0, cooldown: 100, color: "rgb(200,200,200)".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_snake_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Snake];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(20000.0, 0.0, velocity, 90.0, "rgb(25,25,25)");
            let r = Radius::Relative(20.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(0,255,255,0.02)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: r, speed: 10.0, cooldown: 5, time_left: 0, lifetime: 50, projectile_radius: 40.0, color: "rgb(0,0,50)".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_explosion_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Explosion];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.1 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, 20000.0, velocity, 90.0, "rgb(25,25,25)");
            let cd = rand::thread_rng().gen_range(200..=400);
            let radius = rand::thread_rng().gen_range(10.0..=30.0);
            enemy.effects.push(EnemyEffect::Explode { lifetime: 400, radius: (10.0, 30.0), speed: 10.0, time_left: 0, cooldown: cd, color: "rgb(255,255,0)".to_owned(), amount: 10, effects: Vec::new(), under_dps: vec![] });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_lightning_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Lightning];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.1 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let cloudradius = rand::thread_rng().gen_range(100.0..=300.0);
            let color = "rgba(100,80,150,0.7)";
            let mut enemy = Enemy::new(-25000.0, 25000.0, velocity, cloudradius, color);
            enemy.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: Radius::Relative(0.8) }, (cloudradius, cloudradius / 5.0)));
            enemy.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: Radius::Relative(0.7) }, (-cloudradius, cloudradius / 4.0)));
            enemy.harmless = true;
            let cd = rand::thread_rng().gen_range(400..=500);
            let lightning_aura_radius = Radius::Relative(5.0);
            enemy.effects.push(EnemyEffect::Explode {
                lifetime: 400,
                radius: (10.0, 30.0),
                speed: 15.0,
                time_left: 0,
                cooldown: cd,
                color: "rgb(255,255,255)".to_owned(),
                amount: (cloudradius / 20.0) as usize,
                effects: vec![EnemyEffect::SlowPlayers { radius: lightning_aura_radius, slow: 0.0, duration: 100 }],
                under_dps: vec![DrawPack::new("rgba(255,255,0,0.2)", Shape::Circle { radius: lightning_aura_radius }, (0.0,0.0))],
            });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_hell_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Hell];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(100.0..=300.0);
            let color = "rgb(60,0,0)";
            let mut enemy = Enemy::new(25000.0, -25000.0, velocity, radius, color);
            let cd = rand::thread_rng().gen_range(400..=500);
            let fire_aura_radius = Radius::Relative(5.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,0,0.2)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(3.0);
            enemy.effects.push(EnemyEffect::Explode {
                lifetime: 400,
                radius: (10.0, 30.0),
                speed: 15.0,
                time_left: 0,
                cooldown: cd,
                color: "rgb(255,255,0)".to_owned(),
                amount: (radius / 20.0) as usize,
                effects: vec![EnemyEffect::SlowPlayers { radius: fire_aura_radius, slow: 0.3, duration: 100 }, EnemyEffect::Chase { radius: Radius::Absolute(1000.0), power: 0.2 }],
                under_dps: vec![DrawPack::new("rgba(255,200,0,0.2)", Shape::Circle { radius: fire_aura_radius }, (0.0,0.0))],
            });
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: Radius::Relative(3.0), slow: 0.5, duration: 1 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_poison_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Poison];
        let mut enemies = vec![];
        for _ in 0..150 * spawn_m {
            let cap = 0.6 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(100.0..=200.0);
            let color = "rgb(0,255,0)";
            let mut enemy = Enemy::new(25000.0, 25000.0, velocity, radius, color);
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: Radius::Relative(3.0), slow: 0.5, duration: 200 });
            enemy.draw_packs.push(DrawPack::new("rgba(0,255,0,0.2)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(3.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    fn spawn_candy_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Candy];
        let mut enemies = vec![];
        for _ in 0..400 * spawn_m {
            let cap = 0.3 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(50.0..=100.0);
            let color = "rgb(255,0,255)";
            let mut enemy = Enemy::new(-25000.0, -25000.0, velocity, radius, color);
            enemy.id = 1;
            enemy.effects.push(EnemyEffect::Chase { radius: Radius::Relative(3.0), power: -0.05 });
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(4.0), power: -1.5 });
            enemy.draw_packs.push(DrawPack::new("rgba(255,0,255,0.2)", Shape::Circle { radius: Radius::Relative(4.0) }, (0.0, 0.0)));
            enemy.draw_packs.push(DrawPack::new("", Shape::Image { keyword: "candytop".to_owned(), scale: radius / 300.0 }, (-radius / 1.2, -radius / 1.2)));
            enemy.view_radius = Radius::Relative(4.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
    pub fn spawn_enemies(&mut self) {
        let spawn_m = 3;
        let speed_m = 8.0;
        self.spawn_dirt_enemies(speed_m, spawn_m);
        self.spawn_wind_enemies(speed_m, spawn_m);
        self.spawn_flower_enemies(speed_m, spawn_m);
        self.spawn_water_enemies(speed_m, spawn_m);
        self.spawn_fire_enemies(speed_m, spawn_m);
        self.spawn_space_enemies(speed_m, spawn_m);
        self.spawn_tech_enemies(speed_m, spawn_m);
        self.spawn_snake_enemies(speed_m, spawn_m);
        self.spawn_explosion_enemies(speed_m, spawn_m);
        self.spawn_ice_enemies(speed_m, spawn_m);
        self.spawn_lightning_enemies(speed_m, spawn_m);
        self.spawn_poison_enemies(speed_m, spawn_m);
        self.spawn_candy_enemies(speed_m, spawn_m);
        self.spawn_hell_enemies(speed_m, spawn_m);
    }
    pub fn spawn_area(&mut self, corners: Vec<(f32, f32)>, color: &str, walltype: WallType) {
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
            let group = self.walls.iter_mut().find(|(i, _)| {*i == walltype});
            match group {
                Some(g) => {
                    g.1.push(addition);
                },
                None => {
                    self.walls.push((walltype, vec![addition]));
                },
            }
        }
        let poly = Shape::Poly { corners };
        let draw_pack = DrawPack::new(color, poly, (0.0, 0.0));
        self.map.push((start, draw_pack));
    }
    pub fn spawn_grid(&mut self, size: f32, color: &str, space: f32, width: f32) {
        for i in 0..(size as i32 / space as i32) {
            let offset = i as f32 * space;
            self.grid.push((
                (offset, -size),
                DrawPack::new(color, Shape::Line { width, x: offset, y: size }, (0.0, 0.0)),
                true
            ));
            self.grid.push((
                (-offset, -size),
                DrawPack::new(color, Shape::Line { width, x: -offset, y: size }, (0.0, 0.0)),
                true
            ));
            self.grid.push((
                (-size, offset),
                DrawPack::new(color, Shape::Line { width, x: size, y: offset }, (0.0, 0.0)),
                false
            ));
            self.grid.push((
                (-size, -offset),
                DrawPack::new(color, Shape::Line { width, x: size, y: -offset }, (0.0, 0.0)),
                false
            ));
        }
    }
    pub fn spawn_map(&mut self) {
        let multiplier = 2000.0;
        
        // surround
        let corners = vec![(-20.0,0.0),(-15.0,15.0),(0.0,20.0),(15.0,15.0),(20.0,0.0),(15.0,-15.0),(0.0,-20.0),(-15.0,-15.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(20,0,30)", WallType::Blackhole);
        let corners = vec![(-5.0,-5.0),(5.0,-5.0),(5.0,5.0),(-5.0,5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(50,20,30)", WallType::Fire);

        // inner spikes
        let corners = vec![(0.0,0.0),(3.0,1.0),(4.0,4.0),(1.0,3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(80,70,50)", WallType::Dirt);
        let corners = vec![(0.0,0.0),(-3.0,1.0),(-4.0,4.0),(-1.0,3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(120,150,150)", WallType::Wind);
        let corners = vec![(0.0,0.0),(-3.0,-1.0),(-4.0,-4.0),(-1.0,-3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(10,50,20)", WallType::Flower);
        let corners = vec![(0.0,0.0),(3.0,-1.0),(4.0,-4.0),(1.0,-3.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(0,0,50)", WallType::Water);

        // outer spikes
        let corners = vec![(-5.0,-3.0),(-5.0,3.0),(-10.0,2.0),(-12.0,1.0),(-12.5,0.0),(-12.0,-1.0),(-10.0,-2.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(50,50,50)", WallType::Shooting);
        let corners = vec![(5.0,-3.0),(5.0,3.0),(10.0,2.0),(12.0,1.0),(12.5,0.0),(12.0,-1.0),(10.0,-2.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(40,50,40)", WallType::Snake);
        let corners = vec![(-3.0,5.0),(3.0,5.0),(2.0,10.0),(1.0,12.0),(0.0,12.5),(-1.0,12.0),(-2.0,10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(90,70,50)", WallType::Explosion);
        let corners = vec![(-3.0,-5.0),(3.0,-5.0),(2.0,-10.0),(1.0,-12.0),(0.0,-12.5),(-1.0,-12.0),(-2.0,-10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(100,100,150)", WallType::Ice);

        // space spikes
        let corners = vec![(15.0,15.0),(10.0,15.0 + 5.0 / 3.0),(8.0,8.0),(15.0 + 5.0 / 3.0,10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(50,100,50)", WallType::Poison);
        let corners = vec![(-15.0,15.0),(-10.0,15.0 + 5.0 / 3.0),(-8.0,8.0),(-15.0 - 5.0 / 3.0,10.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(20,20,100)", WallType::Lightning);
        let corners = vec![(-15.0,-(15.0)),(-10.0,-(15.0 + 5.0 / 3.0)),(-8.0,-(8.0)),(-15.0 - 5.0 / 3.0,-(10.0))]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(110,50,100)", WallType::Candy);
        let corners = vec![(15.0,-(15.0)),(10.0,-(15.0 + 5.0 / 3.0)),(8.0,-(8.0)),(15.0 + 5.0 / 3.0,-(10.0))]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(100,30,20)", WallType::Hell);
        
        // spawns
        let corners = vec![(-0.4,0.0),(0.0,0.4),(0.4,0.0),(0.0,-0.4)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "black", WallType::SpawnA);

        let corners = vec![(-6.0,6.0),(-5.0,4.0),(-4.0,5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(150,150,150)", WallType::SpawnB);
        let corners = vec![(6.0,6.0),(5.0,4.0),(4.0,5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(150,150,150)", WallType::SpawnB);
        let corners = vec![(-6.0,-6.0),(-5.0,-4.0),(-4.0,-5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(150,150,150)", WallType::SpawnB);
        let corners = vec![(6.0,-6.0),(5.0,-4.0),(4.0,-5.0)]
            .iter().map(|e| {(e.0 * multiplier, e.1 * multiplier)}).collect();
        self.spawn_area(corners, "rgb(150,150,150)", WallType::SpawnB);
        
        // grid
        self.spawn_grid(40000.0, "rgb(255,255,255,0.05)", 500.0, 10.0);
    }
    pub fn spawn_collectables(&mut self) {
        let scale = 0.3;
        let mut item_counter = 0;
        let c = Collectable::new(2000.0, 2000.0, Color::new(200, 200, 100, 1), vec![
            Item::new("monocle", vec![ItemEffect::Vision((0.9,0.9))], vec![], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "monocle".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(0.0, -2000.0, Color::new(200, 200, 100, 1), vec![
            Item::new("microscope", vec![
                ItemEffect::Vision((1.0,5.0)),
            ], vec![], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "microscope".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(4000.0, -4000.0, Color::new(255, 255, 255, 1), vec![
            Item::new("binoculars", vec![ItemEffect::Vision((0.7,1.0))], vec![], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "binoculars".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(-6000.0, 0.0, Color::new(200, 200, 0, 1), vec![
            Item::new("telescope", vec![ItemEffect::Vision((0.4,0.6))], vec![], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "telescope".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(0.0, 0.0, Color::new(255,0,0,1), vec![
            Item::new("heatwave", vec![
                ItemEffect::SlowEnemies { power: 0.5, radius: Radius::Relative(7.0), duration: 100 },
            ], vec![
                DrawPack::new("rgba(255,0,0,0.2)", Shape::Circle { radius: Radius::Relative(7.0) }, (0.0, 0.0))
            ], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "heatwave".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(0.0, 0.0, Color::new(255,0,0,1), vec![
            Item::new("blizzard", vec![
                ItemEffect::SlowEnemies { power: 0.8, radius: Radius::Relative(20.0), duration: 1 },
            ], vec![
                DrawPack::new("rgba(100,100,255,0.1)", Shape::Circle { radius: Radius::Relative(20.0) }, (0.0, 0.0))
            ], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "blizzard".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let c = Collectable::new(200.0, 0.0, Color::new(255,0,0,1), vec![
            Item::new("univeye", vec![
                ItemEffect::Vision((0.01,1.0)),
            ], vec![ ], &mut item_counter,
                Some(DrawPack::new("", Shape::Image { keyword: "univeye".to_owned(), scale }, (0.0, 0.0)))
            )
        ]);
        self.collectables.push(c);
        let cords = vec![(8,0),(-8,0),(0,8),(0,-8)];
        for c in cords {
            for i in 0..10 {
                let center = (c.0 as f32 * 1000.0, c.1 as f32 * 1000.0);
                let distance = (0.0, 2000.0);
                let point = random_point(center, distance);
                let c = Collectable::new(point.0, point.1, Color::new(255,0,0,1), vec![
                    Item::new("dragonfire rune", vec![
                        ItemEffect::Speed(1.1),
                    ], vec![], &mut item_counter,
                        Some(DrawPack::new("", Shape::Image { keyword: "dragonfirerune".to_owned(), scale }, (0.0, 0.0)))
                    )
                ]);
                self.collectables.push(c);
            }
        }
    }
    pub fn new(receiver: Receiver<ServerMessage>) -> Game {
        let mut g = Game {
            game_loop: None,
            running: false,
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
            let mut connections: Vec<(String, Sender<String>)> = vec![];
            loop {
                // handle all messages via loop
                loop {
                    match self.receiver.try_recv() {
                        Ok(message) => {
                            match message {
                                ServerMessage::Login(name, sender) => {
                                    self.players.push(Player::new(&name));
                                    connections.push((name, sender));
                                },
                                ServerMessage::Logout(name) => {
                                    self.logout(&name);
                                    let r = connections.iter().position(|e| {e.0 == name});
                                    match r {
                                        Some(i) => {
                                            connections.remove(i);
                                        },
                                        None => {
                                            println!("tried to delete connection, but connection wasnt there");
                                        },
                                    }
                                },
                                ServerMessage::Input { name, mouse, keys, wheel } => {
                                    self.handle_input(&name, mouse, keys, wheel);
                                },
                            }
                        },
                        Err(error) => {
                            match error {
                                std::sync::mpsc::TryRecvError::Empty => {
                                    break;
                                },
                                std::sync::mpsc::TryRecvError::Disconnected => {},
                            }
                        },
                    }
                }
                let mut deprecated_connections = vec![];
                for (i, connection) in connections.iter().enumerate() {
                    let name = &connection.0;
                    let sender = &connection.1;
                    let r = sender.send(self.pack_objects(name));
                    match r {
                        Ok(_) => {},
                        Err(_) => {
                            // remove from connections request
                            deprecated_connections.push(i);
                        },
                    };
                }
                // remove in reverse order
                for i in deprecated_connections.iter().rev() {
                    connections.remove(*i);
                }

                thread::sleep(Duration::from_millis(1));
                if !self.running {
                    break;
                }

                handle_players(&mut self.players, &mut self.collectables);
                crate::enemy::handle_effects(&mut self);
                crate::item::handle_effects(&mut self);
                crate::player::handle_effects(&mut self);
                handle_collision(&mut self);
                handle_kill_revive(&mut self);
                handle_collectables(&mut self);
                handle_movements(&mut self);
            }
        });
    }
    pub fn pack_objects(&mut self, name: &String) -> String {
        let player = match self.get_mut(name) {
            Some(p) => p,
            None => return "".to_owned(),
        };
        let camera = (player.x, player.y);
        let zoom = player.zoom;

        let view = 1000.0 / zoom;
        let mut objects = "{\"objects\":[".to_owned();
        // map
        for shape in self.map.iter() {
            let acc = draw(0.0, &shape.0, &shape.1, &camera, zoom);
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
            let acc = draw(0.0, &position, &drawpack, &camera, zoom);
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
        // item effects
        for player in self.players.iter() {
            for item in player.inventory.items.iter() {
                if item.active {
                    for dp in item.drawpacks.iter() {
                        // let acc = draw(&(player.x, player.y), &dp, &camera, zoom);
                        let acc = draw(player.get_radius(), &(player.x, player.y), &dp, &camera, zoom);
                        objects.push_str(&acc);
                    }
                }
            }
        }
        // players
        for object in self.players.iter() {
            if vector::distance(camera, (object.x, object.y)).2 > view {continue;}
            let acc = draw_object(object, &camera, zoom);
            objects.push_str(&acc);
        }
        // enemies
        for group in self.enemies.iter() {
            for enemy in group.1.iter() {
                if vector::distance(camera, (enemy.x, enemy.y)).2 - enemy.view_radius.translate(enemy.get_radius()) > view {continue;}
                let acc = draw_object(enemy, &camera, zoom);
                objects.push_str(&acc);
            }
        }
        // inventory
        for object in self.players.iter() {
            if *name == *object.name && object.inventory.open {
                let drawpack = DrawPack::new("rgba(200,100,50,0.8)", Shape::Rectangle { width: 400.0, height: 800.0 }, (-900.0, -400.0));
                let acc = draw(0.0, &(object.x, object.y), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                let drawpack = DrawPack::new("white", Shape::Text { content: "Inventory".to_owned(), size: 30.0 }, (-850.0, -350.0));
                let acc = draw(0.0, &(object.x, object.y), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                // inventory items
                for (i, item) in object.inventory.items.iter().enumerate() {
                    match object.inventory.selected_item {
                        Some(s) => {
                            if i == s {
                                let drawpack = DrawPack::new("rgba(255,255,255,0.3)", Shape::Rectangle { width: 300.0, height: 40.0 }, (-850.0, -330.0 + 50.0 * (i as f32)));
                                let acc = draw(0.0, &(object.x, object.y), &drawpack, &camera, 1.0);
                                objects.push_str(&acc);
                            }
                        },
                        None => {},
                    }
                    let color = if item.active {
                        "green"
                    }
                    else {
                        "black"
                    };
                    let drawpack = DrawPack::new(color, Shape::Text { content: item.name.clone(), size: 30.0 }, (-850.0, -300.0 + 50.0 * (i as f32)));
                    let acc = draw(0.0, &(object.x, object.y), &drawpack, &camera, 1.0);
                    objects.push_str(&acc);
                    match &item.icon {
                        Some(icon) => {
                            let acc = draw(0.0, &(object.x - 890.0, object.y -325.0 + 50.0 * (i as f32)), icon, &camera, 1.0);
                            objects.push_str(&acc);
                        },
                        None => {},
                    }
                }
            }
        }

        objects.push_str("null]}");
        objects
    }
    pub fn handle_input(&mut self, player_name: &String, mouse: (f32, f32), keys_down: Vec<String>, wheel: i32) {
        let player = match self.get_mut(player_name) {
            Some(p) => p,
            None => return,
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
    }
    pub fn get_mut(&mut self, player: &String) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| {p.name == *player})
    }
    pub fn get(&mut self, player: &String) -> Option<&Player> {
        self.players.iter().find(|p| {p.name == *player})
    }
    pub fn logout(&mut self, player: &String) {
        let index = self.players.iter().position(|p| {p.name == *player});
        match index {
            Some(i) => {
                self.players.remove(i);
            },
            None => { }
        };
    }
}

