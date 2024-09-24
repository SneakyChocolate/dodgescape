use serde::Serialize;

use crate::game::DrawPack;

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
    fn get_x(&mut self) -> &mut f32;
    fn get_y(&mut self) -> &mut f32;
    fn get_velocity(&mut self) -> &mut (f32, f32);
    fn get_speed_multiplier(&mut self) -> &mut f32;
}
#[macro_export]
macro_rules! impl_Movable {
    ($struct_name:ident) => {
        impl Moveable for $struct_name {
            fn get_x(&mut self) -> &mut f32 {
                &mut self.x
            }
            fn get_y(&mut self) -> &mut f32 {
                &mut self.y
            }
            fn get_velocity(&mut self) -> &mut (f32, f32) {
                &mut self.velocity
            }
            fn get_speed_multiplier(&mut self) -> &mut f32 {
                &mut self.speed_multiplier
            }
        }
    };
}
