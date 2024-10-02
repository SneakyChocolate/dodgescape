use std::{collections::HashMap, sync::mpsc::{Receiver, Sender}, thread::{self, JoinHandle}, time::Duration};

use crate::{action::Action, collectable::Collectable, color, enemy::Enemy, gametraits::{Drawable, EntityIndex, Moveable, Position, Radius}, player::Player, server::ServerMessage, vector::{self, get_intersection, Line}, wall::{Wall, WallType}};
use crate::gametraits::*;
use crate::{impl_RadiusTrait, impl_Drawable, impl_Entity, impl_Moveable, impl_Position};
use serde::Serialize;

pub fn move_object<T: Moveable>(object: &mut T, walls: &Walls, walltypes: Option<&Vec<WallType>>) {
    let (vx, vy) = object.get_velocity();
    let x = object.get_x() + vx;
    let y = object.get_y() + vy;
    object.set_pos(x, y);
}
pub fn get_player<'a>(game: &'a mut Game, player: usize) -> &'a mut Player {
    game.players.get_mut(player).unwrap()
}
pub fn get_enemy<'a>(game: &'a mut Game, group: usize, enemy: usize) -> &'a mut Enemy {
    game.enemies.get_mut(group).unwrap().1.get_mut(enemy).unwrap()
}

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
    let pos = (object.get_x(), object.get_y());
    let draw_packs = object.get_draw_packs();
    let mut output = "".to_owned();
    for (d, draw_pack) in draw_packs.iter().enumerate() {
        let s = draw(object.get_radius(), &pos, draw_pack, &camera, zoom);
        output.push_str(&s);
    }
    output
}

pub fn distance<T: Position, B: Position>(a: &T, b: &B) -> (f32, f32, f32) {
    let a = (a.get_x(), a.get_y());
    let b = (b.get_x(), b.get_y());
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

pub type Walls = Vec<(WallType, Vec<Wall>)>;
pub type Enemies = Vec<(Vec<WallType>, Vec<Enemy>)>;

pub struct Game {
    pub receiver: Receiver<ServerMessage>,
    
    pub players: Vec<Player>,
    pub game_loop: Option<JoinHandle<()>>,
    pub running: bool,
    pub enemies: Enemies,
    pub grid: Vec<((f32, f32), DrawPack, bool)>,
    pub map: Vec<((f32, f32), DrawPack)>,
    pub walls: Walls,
    pub collectables: Vec<Collectable>,
}

pub fn handle_players(players: &mut Vec<Player>, collectables: &mut Vec<Collectable>) {
    for object in players {
        object.handle_keys(collectables);
        if object.alive {
            object.draw_packs[0].color = object.color.clone();
        }
        else {
            object.draw_packs[0].color = "rgb(255,0,0)".to_owned();
        }
    }
}
pub fn handle_collectables(game: &mut Game) {
    let mut rems: Vec<usize> = vec![];
    for p in game.players.iter_mut() {
        for (i, c) in game.collectables.iter_mut().enumerate() {
            let dist = distance(p, c);
            if dist.2 <= p.get_radius() + c.get_radius() {
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

pub fn cross_barrier_check<T: Moveable>(object: &T, wall: &Wall) -> Option<f32> {
    let a = Line::from_points(object.get_old(), (object.get_x(), object.get_y()));
    if vector::abs(a.dir) < object.get_radius() {
        return None;
    }
    let b = Line::from_points(wall.a, wall.b);
    let intersection = get_intersection(a, b);
    match intersection {
        Some(xy) => {
            let condition =
                xy.0 <= 1.0 && xy.0 >= 0.0 &&
                xy.1 <= 1.0 && xy.1 >= 0.0
            ;
            if condition {
                Some(xy.0)
            }
            else {
                None
            }
        },
        None => None,
    }
}

pub fn handle_collision(game: &mut Game) {
    for group in game.enemies.iter_mut() {
        for enemy in group.1.iter_mut() {
            enemy.just_collided = false;
        }
    }
    let mut collisions: Vec<(EntityIndex, (f32, f32))> = vec![];
    // let mut barrier_crosses: Vec<(usize, Action, f32, usize)> = vec![];
    let mut barrier_crosses: HashMap<(usize, usize), f32> = HashMap::new();
    for wgroup in game.walls.iter() {
        for wall in wgroup.1.iter() {
            // enemies
            if wall.enemy {
                for (g, egroup) in game.enemies.iter().enumerate() {
                    if !egroup.0.contains(&wgroup.0) {continue;} 
                    for (e, enemy) in egroup.1.iter().enumerate() {
                        match cross_barrier_check(enemy, wall) {
                            Some(f) => {
                                let enemyv = Line::from_points(enemy.old_position, (enemy.get_x(), enemy.get_y()));
                                let pos = enemyv.point(0.0);
                                let bcs = barrier_crosses.get_mut(&(e, g));
                                match bcs {
                                    Some(bc) => {
                                        if f < *bc {
                                            *bc = f;
                                        }
                                    },
                                    None => {
                                        // no shorter distance found
                                        barrier_crosses.insert((e, g), f);
                                    },
                                }
                            },
                            None => {},
                        };
                        let cp = wall.get_nearest_point(&(enemy.get_x(), enemy.get_y()));
                        if vector::distance(cp, (enemy.get_x(), enemy.get_y())).2 <= enemy.get_radius() {
                            if !collisions.iter().any(|c| {
                                match c.0 {
                                    EntityIndex::Player { p } => false,
                                    EntityIndex::Enemy { g, e: e2 } => e2 == e,
                                }
                            }) {
                                collisions.push((EntityIndex::Enemy { g, e }, cp));
                            }
                        }
                    }
                }
            }
            // players
            if wall.player {
                for (p, player) in game.players.iter().enumerate() {
                    // cross_barrier_check(player, wall);
                    let cp = wall.get_nearest_point(&(player.get_x(), player.get_y()));
                    if vector::distance(cp, (player.get_x(), player.get_y())).2 <= player.get_radius() {
                        if collisions.iter().any(|c| {
                            match c.0 {
                                EntityIndex::Player { p: p2 } => p == p2,
                                EntityIndex::Enemy { g, e } => false,
                            }
                        }) {
                            continue;
                        }
                        collisions.push((EntityIndex::Player { p }, cp));
                    }
                }
            }
        }
    }
    for bc in barrier_crosses.iter() {
        
    }
    // offset for pushing object away on collision so collision doesnt trigger again
    const OFFSET: f32 = 0.001;
    for (collision, cp) in collisions {
        let object: Box<&mut dyn Moveable> = match collision {
            EntityIndex::Player { p } => {
                Box::new(get_player(game, p))
            },
            EntityIndex::Enemy { g, e } => {
                Box::new(get_enemy(game, g, e))
            },
        };
        let speed = vector::abs(object.get_velocity());
        let dist = vector::distance(cp, (object.get_x(), object.get_y()));
        let push = vector::normalize((dist.0, dist.1), object.get_radius() + OFFSET);
        let x = cp.0 + push.0;
        let y = cp.1 + push.1;
        object.set_pos(x, y);
        let new_v = vector::normalize(vector::collision((object.get_x(), object.get_y()), object.get_velocity(), cp), speed);
        object.set_velocity(new_v);
        object.set_just_collided(true);
    }
}
pub fn handle_movements(game: &mut Game) {
    for object in &mut game.players {
        if object.alive && !object.skip_move {
            move_object(object, &game.walls, None);
        }
        else {
            object.skip_move = false;
        }
    }
    for group in game.enemies.iter_mut() {
        for object in group.1.iter_mut() {
            move_object(object, &game.walls, Some(&group.0));
        }
    }
}

impl Game {
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

                crate::enemy::handle_effects(&mut self);
                crate::player::handle_effects(&mut self);
                crate::item::handle_effects(&mut self);
                handle_players(&mut self.players, &mut self.collectables);
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
        let camera = (player.get_x(), player.get_y());
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
                        let acc = draw(player.get_radius(), &(player.get_x(), player.get_y()), &dp, &camera, zoom);
                        objects.push_str(&acc);
                    }
                }
            }
        }
        // players
        for player in self.players.iter() {
            if vector::distance(camera, (player.get_x(), player.get_y())).2 > view {continue;}
            if player.invincible {
                let mut dps = player.draw_packs.clone();
                match dps.get_mut(0) {
                    Some(dp) => {
                        dp.color = color::Color::from_str(dp.color.as_str()).mul(0.5).to_string();
                    },
                    None => {},
                }
                for dp in dps.iter() {
                    let acc = draw(player.get_radius(), &(player.get_x(), player.get_y()), dp, &camera, zoom);
                    objects.push_str(&acc);
                }
            }
            else {
                let acc = draw_object(player, &camera, zoom);
                objects.push_str(&acc);
            }

            // player effects
            for effect in player.effects.iter() {
                match effect {
                    crate::player::PlayerEffect::Harden { ease, cooldown } => {
                        let pos = (player.get_x() - 20.0, player.get_y() - 60.0);
                        let dp = DrawPack::new("rgba(0,0,255,0.5)", Shape::Text { content: format!("cd: {}", *cooldown), size: 20.0 }, (0.0, 0.0));
                        let acc = draw(0.0, &pos, &dp, &camera, zoom);
                        objects.push_str(&acc);
                    },
                    _ => {}
                }
            }
        }
        // enemies
        for group in self.enemies.iter() {
            for enemy in group.1.iter() {
                if vector::distance(camera, (enemy.get_x(), enemy.get_y())).2 - enemy.view_radius.translate(enemy.get_radius()) > view {continue;}
                let acc = draw_object(enemy, &camera, zoom);
                objects.push_str(&acc);
            }
        }
        // inventory
        for object in self.players.iter() {
            if *name == *object.name && object.inventory.open {
                let drawpack = DrawPack::new("rgba(200,100,50,0.8)", Shape::Rectangle { width: 400.0, height: 800.0 }, (-900.0, -400.0));
                let acc = draw(0.0, &(object.get_x(), object.get_y()), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                let drawpack = DrawPack::new("white", Shape::Text { content: "Inventory".to_owned(), size: 30.0 }, (-850.0, -350.0));
                let acc = draw(0.0, &(object.get_x(), object.get_y()), &drawpack, &camera, 1.0);
                objects.push_str(&acc);

                // inventory items
                for (i, item) in object.inventory.items.iter().enumerate() {
                    let line_height = 50.0;
                    let line_offset = line_height * (i as f32);
                    match object.inventory.selected_item {
                        Some(s) => {
                            if i == s {
                                let drawpack = DrawPack::new("rgba(255,255,255,0.3)", Shape::Rectangle { width: 300.0, height: 40.0 }, (-850.0, -330.0 + line_offset));
                                let acc = draw(0.0, &(object.get_x(), object.get_y()), &drawpack, &camera, 1.0);
                                objects.push_str(&acc);
                            }
                        },
                        None => {},
                    }
                    let color = if item.active {
                        "rgb(0,100,0)"
                    }
                    else {
                        "rgb(100,0,0)"
                    };
                    let mut append = "".to_owned();
                    for effect in item.effects.iter() {
                        match effect {
                            crate::item::ItemEffect::Consumable { uses } => {
                                append.push_str(format!("({})", *uses).as_str());
                            },
                            _ => {}
                        };
                    }
                    let drawpack = DrawPack::new(color, Shape::Text { content: format!("{} {}", item.name.clone(), append), size: 30.0 }, (-850.0, -300.0 + line_offset));
                    let acc = draw(0.0, &(object.get_x(), object.get_y()), &drawpack, &camera, 1.0);
                    objects.push_str(&acc);
                    match &item.icon {
                        Some(icon) => {
                            let acc = draw(0.0, &(object.get_x() - 890.0, object.get_y() -325.0 + line_offset), icon, &camera, 1.0);
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

