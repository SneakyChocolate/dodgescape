use std::array::TryFromSliceError;

use crate::{game::{Drawable, Moveable, Position, Shape}, impl_Drawable, impl_Movable, impl_Position};

#[derive(Debug, Default)]
pub struct Player {
    pub mouse: (i32, i32),
    pub keys_down: Vec<String>,
    pub velocity: (f32, f32),
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub shapes: Vec<(String, Shape, (f32,f32))>,
    pub alive: bool,
    pub radius: f32,
}

impl_Position!(Player);
impl_Movable!(Player);
impl_Drawable!(Player);

impl Player {
    pub fn new(name: &String) -> Player {
        let mut p = Player {
            name: name.clone(),
            radius: 30.0,
            alive: true,
            ..Default::default()
        };
        p.shapes.push(("blue".to_owned(), Shape::Circle { radius: p.radius }, (0.0, 0.0)));
        p.shapes.push(("white".to_owned(), Shape::Text { content: name.clone(), size: 20.0 }, (-20.0, -40.0)));
        p.shapes.push(("red".to_owned(), Shape::Line { x: 0.0, y: 0.0 }, (0.0, 0.0)));

        p
    }
    pub fn handle_keys(&mut self) {
        let key = "KeyW".to_owned();
        let mut vx = 0.0;
        let mut vy = 0.0;
        if self.keys_down.contains(&key) {
            vy = -1.0;
        }
        let key = "KeyS".to_owned();
        if self.keys_down.contains(&key) {
            vy = 1.0;
        }
        let key = "KeyD".to_owned();
        if self.keys_down.contains(&key) {
            vx = 1.0;
        }
        let key = "KeyA".to_owned();
        if self.keys_down.contains(&key) {
            vx = -1.0;
        }
        self.velocity = (vx, vy);
    }
}

