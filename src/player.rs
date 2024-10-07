

use crate::{action::Action, collectable::Collectable, color::Color, game::{DrawPack, Game, Shape, Walls}, impl_Drawable, impl_Entity, impl_Moveable, impl_Position, inventory::Inventory, vector, wall::WallType, Float};
use crate::gametraits::*;
use crate::{impl_RadiusTrait};

#[derive(Clone, Copy, Debug)]
pub enum PlayerEffect {
    Shrink {origin: usize, shrink: Float, ease: usize},
    SpeedAlter {origin: usize, slow: Float, ease: usize},
    Harden {ease: usize, cooldown: usize, speed: Float},
}

#[derive(Default)]
pub struct Player {
    pub alive: bool,
    pub color: String,
    pub draw_packs: Vec<DrawPack>,
    pub effects: Vec<PlayerEffect>,
    pub inventory: Inventory,
    pub invincible: bool,
    just_pressed: Vec<String>,
    pub keys_down: Vec<String>,
    pub mouse: (Float, Float),
    pub name: String,
    old_keys_down: Vec<String>,
    pub radius: Float,
    pub radius_multiplier: Float,
    pub skip_move: bool,
    pub speed: Float,
    pub speed_multiplier: Float,
    pub velocity: (Float, Float),
    pub x: Float,
    pub y: Float,
    pub zoom: Float,
    pub zoomlimit: (Float, Float),
    pub just_collided: bool,
    pub old_position: (Float, Float),
}

impl_Entity!(Player);

impl Player {
    pub fn new(name: &String) -> Player {
        let color = Color::random().to_string();
        let mut p = Player {
            x: 0.0,
            y: 0.0,
            name: name.clone(),
            radius: 30.0,
            alive: true,
            speed: 15.0,
            zoom: 1.0,
            zoomlimit: (1.0, 1.0),
            color: color.clone(),
            speed_multiplier: 1.0,
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(p.color.as_str(), Shape::Circle { radius: Radius::Relative(1.0) }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new("white", Shape::Text { content: name.clone(), size: 20.0 }, (-20.0, -40.0)));
        // p.draw_packs.push(DrawPack::new("red", Shape::Line { x: 0.0, y: 0.0, width: 10.0 }, (0.0, 0.0)));

        p
    }
    fn scroll_use(&mut self) {
        self.alive = true;
        let scroll = self.inventory.items.iter().position(|e| {e.name == "teleportation scroll".to_owned()}).unwrap();
        self.inventory.items.remove(scroll);
    }
    fn tp_possibility(&mut self, target: (Float, Float), key: &str) {
        let scroll = self.inventory.items.iter().position(|e| {e.name == "teleportation scroll".to_owned()});
        match scroll {
            Some(_) => {
                let key = key.to_owned();
                if self.just_pressed.contains(&key) {
                    self.x = target.0;
                    self.y = target.1;
                    self.scroll_use();
                }
            },
            None => {},
        };
    }
    fn handle_respawn(&mut self) {
        let key = "KeyR".to_owned();
        if self.keys_down.contains(&key) {
            self.x = 0.0;
            self.y = 0.0;
            self.alive = true;
        }
        
        let spawnd = 11000.0;
        self.tp_possibility((-spawnd, -spawnd), "Digit1");
        self.tp_possibility((spawnd, -spawnd), "Digit2");
        self.tp_possibility((-spawnd, spawnd), "Digit3");
        self.tp_possibility((spawnd, spawnd), "Digit4");

        let key = "KeyQ".to_owned();
        if self.just_pressed.contains(&key) {
            let heart = self.inventory.items.iter_mut().find(|e| {e.name == "heart"});
            match heart {
                Some(heart) => {
                    heart.active = true;
                },
                None => {},
            };
            // self.alive = true;
        }
    }
    fn handle_inventory(&mut self, collectables: &mut Vec<Collectable>) {
        // handle keybindings
        if !self.inventory.bind_mode {
            for key in self.just_pressed.iter() {
                let b = self.inventory.bindings.get(key);
                match b {
                    Some(i) => {
                        let item = self.inventory.items.get_mut(*i);
                        match item {
                            Some(item) => {
                                item.active = !item.active;
                            },
                            None => {},
                        }
                    },
                    None => {},
                }
            }
        }

        let key = "KeyE".to_owned();
        if self.just_pressed.contains(&key) {
            self.inventory.open = !self.inventory.open;
        }
        let key = "KeyC".to_owned();
        if self.just_pressed.contains(&key) {
            for c in collectables.iter_mut() {
                c.collect(self);
            }
            collectables.clear();
        }
        let key = "KeyG".to_owned();
        if self.just_pressed.contains(&key) {
            match &mut self.inventory.selected_item {
                Some(i) => {
                    let mut item = self.inventory.items.remove(*i);
                    if self.inventory.items.len() == 0 {
                        self.inventory.selected_item = None;
                    }
                    else if *i == self.inventory.items.len() {
                        *i -= 1;
                    }
                    item.active = false;
                    let collectable = Collectable::new(self.x, self.y + 50.0, Color::new(0, 0, 255, 1), vec![item]);
                    collectables.push(collectable);
                },
                None => {},
            };
        }
        if self.inventory.open {
            if self.inventory.items.len() > 0 {
                match &mut self.inventory.selected_item {
                    None => {
                        self.inventory.selected_item = Some(0);
                    },
                    Some(s) => {
                        let key = "KeyB".to_owned();
                        if self.just_pressed.contains(&key) {
                            if self.inventory.bind_mode {
                                self.inventory.bind_mode = false;
                                self.inventory.bindings.clear();
                            }
                            else {
                                self.inventory.bind_mode = true;
                            }
                        }
                        else {
                            if self.inventory.bind_mode {
                                let key = "Escape".to_owned();
                                if self.just_pressed.contains(&key) {
                                    self.inventory.bind_mode = false;
                                }
                                else {
                                    let key = self.just_pressed.get(0);
                                    match key {
                                        Some(binding) => {
                                            self.inventory.bind_mode = false;
                                            self.inventory.bindings.insert(binding.clone(), *s);
                                        },
                                        None => {},
                                    }
                                }
                            }
                        }

                        let key = "ArrowDown".to_owned();
                        if self.just_pressed.contains(&key) {
                            if *s == self.inventory.items.len() - 1 {
                                *s = 0;
                            }
                            else {
                                *s += 1;
                            }
                        }
                        let key = "ArrowUp".to_owned();
                        if self.just_pressed.contains(&key) {
                            if *s == 0 {
                                *s = self.inventory.items.len() - 1;
                            }
                            else {
                                *s -= 1;
                            }
                        }
                        let key = "ArrowRight".to_owned();
                        if self.just_pressed.contains(&&key) {
                            let item = self.inventory.items.get_mut(*s);
                            match item {
                                Some(item) => {
                                    item.active = true;
                                },
                                None => {},
                            }
                        }
                        let key = "ArrowLeft".to_owned();
                        if self.just_pressed.contains(&&key) {
                            let item = self.inventory.items.get_mut(*s);
                            match item {
                                Some(item) => {
                                    item.active = false;
                                },
                                None => {},
                            }
                        }
                    },
                }
            }
        }
        else {
            self.inventory.selected_item = None;
            self.inventory.bind_mode = false;
        }
    }
    fn handle_movement(&mut self) {
        let mut vx = 0.0;
        let mut vy = 0.0;
        let key = "Space".to_owned();
        if self.keys_down.contains(&key) {
            vx = self.mouse.0 as Float / 50.0 * self.speed;
            vy = self.mouse.1 as Float / 50.0 * self.speed;
        }
        else {
            let key = "KeyW".to_owned();
            if self.keys_down.contains(&key) {
                vy += -self.speed;
            }
            let key = "KeyS".to_owned();
            if self.keys_down.contains(&key) {
                vy += self.speed;
            }
            let key = "KeyD".to_owned();
            if self.keys_down.contains(&key) {
                vx += self.speed;
            }
            let key = "KeyA".to_owned();
            if self.keys_down.contains(&key) {
                vx += -self.speed;
            }
        }
        if vector::abs((vx, vy)) > self.speed {
            (vx, vy) = vector::normalize((vx, vy), self.speed);  
        }
        // slowing down
        let key = "ShiftLeft".to_owned();
        if self.keys_down.contains(&key) {
            self.speed_multiplier *= 0.5;
        }
        self.velocity = (vx, vy);
    }
    pub fn get_just_pressed(&mut self) -> Vec<String> {
        let mut jp = vec![];
        for key in self.keys_down.iter() {
            if !self.old_keys_down.contains(key) {
                jp.push(key.clone());
            }
        }
        jp
    }
    pub fn handle_keys(&mut self, collectables: &mut Vec<Collectable>) {
        self.just_pressed = self.get_just_pressed();
        self.handle_respawn();
        self.handle_inventory(collectables);
        self.handle_movement();
        self.old_keys_down = self.keys_down.clone();
    }
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    let mut deletions: Vec<(usize, Action)> = vec![];
    // convert effects to actions
    for (p, player) in game.players.iter_mut().enumerate() {
        for (e, effect) in player.effects.iter_mut().enumerate() {
            match effect {
                PlayerEffect::SpeedAlter { origin, slow, ease } => {
                    if *ease == 0 {
                        // remove this effect
                        deletions.push((p, Action::RemovePlayerEffect { effect: e }));
                    }
                    else {
                        *ease -= 1;
                        actions.push((p, Action::MulPlayerSpeedMultiplier { f: *slow }));
                    }
                },
                PlayerEffect::Shrink { origin, shrink, ease } => {
                    if *ease == 0 {
                        // remove this effect
                        deletions.push((p, Action::RemovePlayerEffect { effect: e }));
                    }
                    else {
                        *ease -= 1;
                        actions.push((p, Action::MulPlayerRadiusMultiplier { f: *shrink }));
                    }
                },
                PlayerEffect::Harden { ease, cooldown, speed } => {
                    if *cooldown == 0 {
                        // remove this effect
                        deletions.push((p, Action::RemovePlayerEffect { effect: e }));
                    }
                    else {
                        *cooldown -= 1;
                    }
                    if *ease == 0 {
                        // effect shouldnt work anymore
                    }
                    else {
                        *ease -= 1;
                        actions.push((p, Action::MulPlayerSpeedMultiplier { f: *speed }));
                        actions.push((p, Action::SetPlayerInvincible(true)));
                    }
                },
            }
        }
    }
    // reset enemy speed multiplier to 1.0
    for player in game.players.iter_mut() {
        player.speed_multiplier = 1.0;
        player.radius_multiplier = 1.0;
        player.zoomlimit = (1.0, 1.0);
        player.invincible = false;
    }
    // reverse order due to deletions and index errors
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
    for (entity, action) in deletions.iter().rev() {
        action.execute(game, *entity);
    }
}
