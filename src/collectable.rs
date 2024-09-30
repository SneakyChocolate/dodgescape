use crate::{color::Color, game::{DrawPack, Shape, Walls}, impl_Drawable, impl_Movable, impl_Position, item::Item, player::Player, wall::WallType};
use crate::gametraits::*;

#[derive(Default)]
pub struct Collectable {
    pub velocity: (f32, f32),
    pub speed_multiplier: f32,
    pub radius_multiplier: f32,
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub just_collided: bool,
    pub items: Vec<Item>,
}

impl_Position!(Collectable);
impl_Movable!(Collectable);
impl_Drawable!(Collectable);

impl MoveObject for Collectable {
    fn barrier_cross_check(&mut self, old_position: (f32, f32), walls: &Walls, walltypes: Option<&Vec<WallType>>) {
        // TODO
    }
}

impl Collectable {
    pub fn new(x: f32, y: f32, color: Color, items: Vec<Item>) -> Self {
        let mut p = Self {
            x,y,
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



