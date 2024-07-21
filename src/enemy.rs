use crate::{game::{DrawPack, Shape}, impl_Drawable, impl_Movable, impl_Position};
use crate::gametraits::*;

pub enum Effect {
    Chase {radius: f32, power: f32},
    Crumble,
    Lifetime(usize),
    Push {radius: f32, power: f32},
}

#[derive(Default)]
pub struct Enemy {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub effects: Vec<Effect>,
    pub just_collided: bool,
}

impl_Position!(Enemy);
impl_Movable!(Enemy);
impl_Drawable!(Enemy);

impl Enemy {
    pub fn new(x: f32, y: f32, velocity: (f32, f32), radius: f32, color: &str) -> Enemy {
        let mut p = Enemy {
            x,y,
            velocity,
            radius,
            draw_packs: vec![],
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: p.radius }, (0.0, 0.0)));

        p
    }
}


