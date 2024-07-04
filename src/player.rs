use crate::game::{Drawable, Moveable, Shape};


#[derive(Debug, Default)]
pub struct Player {
    pub mouse: (i32, i32),
    pub keys_down: Vec<String>,
    pub velocity: (f32, f32),
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub shapes: Vec<(String, Shape, (f32,f32))>,
}

impl Moveable for Player {
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

impl Drawable for Player {
    fn get_pos(&self) -> (f32,f32) {
        (self.x, self.y)
    }
    fn get_shapes(&self) -> &Vec<(String, Shape, (f32,f32))> {
        &self.shapes
    }
}

impl Player {
    pub fn new(name: &String) -> Player {
        let mut p = Player {
            name: name.clone(),
            ..Default::default()
        };
        p.shapes.push(("blue".to_owned(), Shape::Circle { radius: 30.0 }, (0.0, 0.0)));
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


