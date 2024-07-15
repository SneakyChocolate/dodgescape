use crate::{game::{DrawPack, Drawable, Moveable, Position, Shape}, impl_Drawable, impl_Movable, impl_Position};


#[derive(Default)]
pub struct Enemy {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub effects: Vec<fn(&mut f32, &mut f32, &mut (f32, f32), &mut f32) -> ()>
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
    pub fn run_effects(&mut self) {
        for effect in self.effects.iter_mut() {
            effect(&mut self.x, &mut self.y, &mut self.velocity, &mut self.radius);
        }
    }
}


