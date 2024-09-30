use serde::Serialize;

use crate::{game::{DrawPack, Walls}, wall::WallType};

#[derive(Serialize, Debug, Clone, Copy)]
pub enum Radius {
    Absolute(f32),
    Relative(f32),
}

impl Radius {
    pub fn translate(&self, origin: f32) -> f32 {
        match self {
            Radius::Absolute(v) => *v,
            Radius::Relative(v) => origin * *v,
        }
    }
}

impl Default for Radius {
    fn default() -> Self {
        Radius::Relative(1.0)
    }
}

pub trait Drawable {
    fn get_radius(&self) -> f32;
    fn get_draw_packs(&self) -> &Vec<DrawPack>;
}
#[macro_export]
macro_rules! impl_Drawable {
    ($struct_name:ident) => {
        impl Drawable for $struct_name {
            fn get_draw_packs(&self) -> &Vec<DrawPack> {
                &self.draw_packs
            }
            fn get_radius(&self) -> f32 {
                self.radius * self.radius_multiplier
            }
        }
    };
}
pub trait Position {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}
#[macro_export]
macro_rules! impl_Position {
    ($struct_name:ident) => {
        impl Position for $struct_name {
            fn x(&self) -> f32 {
                self.x
            }
            fn y(&self) -> f32 {
                self.y
            }
        }
    };
}
pub trait Moveable {
    fn set_pos(&mut self, x: f32, y: f32, walls: &Walls, walltypes: Option<&Vec<WallType>>);
    fn set_velocity(&mut self, v: (f32, f32));
    fn set_speed_multiplier(&mut self, v: f32);
    fn get_x(&self) -> f32;
    fn get_y(&self) -> f32;
    fn get_velocity(&self) -> (f32, f32);
    fn get_speed_multiplier(&self) -> f32;
}
#[macro_export]
macro_rules! impl_Movable {
    ($struct_name:ident) => {
        impl Moveable for $struct_name {
            fn get_x(&self) -> f32 {
                self.x
            }
            fn get_y(&self) -> f32 {
                self.y
            }
            fn get_velocity(&self) -> (f32, f32) {
                self.velocity
            }
            fn get_speed_multiplier(&self) -> f32 {
                self.speed_multiplier
            }
            fn set_pos(&mut self, x: f32, y: f32, walls: &Walls, walltypes: Option<&Vec<WallType>>) {
                let old = (self.get_x(), self.get_y());
                self.x = x;
                self.y = y;
                self.barrier_cross_check(old, walls, walltypes);
            }
            fn set_velocity(&mut self, v: (f32, f32)) {
                self.velocity = v;
            }
            fn set_speed_multiplier(&mut self, v: f32) {
                self.speed_multiplier = v;
            }
        }
    };
}
pub trait MoveObject {
    fn barrier_cross_check(&mut self, old_position: (f32, f32), walls: &Walls, walltypes: Option<&Vec<WallType>>);
}
