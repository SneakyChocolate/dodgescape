use crate::{game::{Drawable, Moveable, Position, Shape}, impl_Drawable, impl_Movable, impl_Position};


#[derive(Debug, Default)]
pub struct Enemy {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub shapes: Vec<(String, Shape, (f32,f32))>,
    pub radius: f32,
}

impl_Position!(Enemy);
impl_Movable!(Enemy);
impl_Drawable!(Enemy);

impl Enemy {
    pub fn new(x: f32, y: f32, velocity: (f32, f32)) -> Enemy {
        let mut p = Enemy {
            x,y,
            velocity,
            radius: 10.0,
            ..Default::default()
        };
        p.shapes.push(("rgb(200,200,200)".to_owned(), Shape::Circle { radius: p.radius }, (0.0, 0.0)));

        p
    }
}


