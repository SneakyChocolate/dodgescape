use crate::game::{Drawable, Moveable, Shape};


#[derive(Debug, Default)]
pub struct Enemy {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub shapes: Vec<(String, Shape, (f32,f32))>,
}

impl Moveable for Enemy {
    fn get_x(&mut self) -> &mut f32 {
        &mut self.x
    }
    fn get_y(&mut self) -> &mut f32 {
        &mut self.y
    }
    fn get_velocity(&self) -> &(f32, f32) {
        &self.velocity
    }
}

impl Drawable for Enemy {
    fn get_pos(&self) -> (f32,f32) {
        (self.x, self.y)
    }
    fn get_shapes(&self) -> &Vec<(String, Shape, (f32,f32))> {
        &self.shapes
    }
}

impl Enemy {
    pub fn new(x: f32, y: f32, velocity: (f32, f32)) -> Enemy {
        let mut p = Enemy {
            x,y,
            velocity,
            ..Default::default()
        };
        p.shapes.push(("rgb(200,200,200)".to_owned(), Shape::Circle { radius: 30.0 }, (0.0, 0.0)));

        p
    }
}


