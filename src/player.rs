
use crate::{game::{DrawPack, Shape}, impl_Drawable, impl_Movable, impl_Position, inventory::Inventory, vector};
use crate::gametraits::*;

#[derive(Default)]
pub struct Player {
    pub mouse: (f32, f32),
    pub keys_down: Vec<String>,
    pub velocity: (f32, f32),
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub alive: bool,
    pub radius: f32,
    pub speed: f32,
    pub skip_move: bool,
    pub inventory: Inventory,
    pub zoom: f32,
}

impl_Position!(Player);
impl_Movable!(Player);
impl_Drawable!(Player);

impl Player {
    pub fn new(name: &String) -> Player {
        let mut p = Player {
            x: 10000.0,
            y: 0.0,
            name: name.clone(),
            radius: 30.0,
            alive: true,
            speed: 8.0,
            zoom: 1.0,
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new("blue", Shape::Circle { radius: p.radius }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new("white", Shape::Text { content: name.clone(), size: 20.0 }, (-20.0, -40.0)));
        // p.draw_packs.push(DrawPack::new("red", Shape::Line { x: 0.0, y: 0.0, width: 10.0 }, (0.0, 0.0)));

        p
    }
    fn handle_respawn(&mut self) {
        let key = "KeyR".to_owned();
        if self.keys_down.contains(&key) {
            self.x = 0.0;
            self.y = 0.0;
            self.alive = true;
        }
        let key = "KeyQ".to_owned();
        if self.keys_down.contains(&key) {
            self.alive = true;
        }
    }
    fn handle_inventory(&mut self) {
        let key = "KeyE".to_owned();
        if self.keys_down.contains(&key) {
            self.inventory.open = true;
        }
        else {
            self.inventory.open = false;
        }
    }
    fn handle_movement(&mut self) {
        let mut vx = 0.0;
        let mut vy = 0.0;
        let key = "Space".to_owned();
        if self.keys_down.contains(&key) {
            vx = self.mouse.0 as f32 / 200.0 * self.speed;
            vy = self.mouse.1 as f32 / 200.0 * self.speed;
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
            vx /= 2.0;
            vy /= 2.0;
        }
        self.velocity = (vx, vy);
    }
    pub fn handle_keys(&mut self) {
        self.handle_respawn();
        self.handle_inventory();
        self.handle_movement();
    }
}


