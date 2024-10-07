use crate::{color::Color, game::{DrawPack, Shape, Walls}, impl_Drawable, impl_Entity,  impl_Moveable, impl_Position, item::Item, player::Player, wall::WallType, Float};
use crate::gametraits::*;
use crate::{impl_RadiusTrait};

#[derive(Default)]
pub struct Collectable {
    pub velocity: (Float, Float),
    pub speed_multiplier: Float,
    pub radius_multiplier: Float,
    pub x: Float,
    pub y: Float,
    pub draw_packs: Vec<DrawPack>,
    pub radius: Float,
    pub just_collided: bool,
    pub items: Vec<Item>,
    pub old_position: (Float, Float),
}

impl_Entity!(Collectable);

impl Collectable {
    pub fn new(x: Float, y: Float, color: Color, items: Vec<Item>) -> Self {
        let mut p = Self {
            x,y,
            old_position: (x, y),
            velocity: (0.0, 0.0),
            radius: 15.0,
            draw_packs: vec![],
            items,
            radius_multiplier: 1.0,
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(color.mul(0.8).to_string().as_str(), Shape::Circle { radius: Radius::Relative(1.0) }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new(color.mul(0.5).to_string().as_str(), Shape::Circle { radius: Radius::Relative(0.8) }, (0.0, 0.0)));
        p.draw_packs.push(DrawPack::new(color.to_string().as_str(), Shape::Text { content: "B".to_owned(), size: 20.0 }, (-5.0, 7.0)));

        p
    }
    pub fn collect(&mut self, player: &mut Player) {
        player.inventory.items.append(&mut self.items);
    }
}



