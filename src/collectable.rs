use crate::{game::{DrawPack, Shape}, impl_Drawable, impl_Movable, impl_Position};
use crate::gametraits::*;

#[derive(Default)]
pub struct Collectable {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub just_collided: bool,
}

impl_Position!(Collectable);
impl_Movable!(Collectable);
impl_Drawable!(Collectable);

impl Collectable {
    pub fn new(x: f32, y: f32, color: &str) -> Self {
        let mut p = Self {
            x,y,
            velocity: (0.0, 0.0),
            radius: 15.0,
            draw_packs: vec![],
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new("rgb(150,120,0)", Shape::Circle { radius: p.radius }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new("rgb(200,170,0)", Shape::Circle { radius: p.radius * 0.8 }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new("rgb(255,255,0)", Shape::Text { content: "$".to_owned(), size: 20.0 }, (-5.0, 7.0)));

        p
    }
}



