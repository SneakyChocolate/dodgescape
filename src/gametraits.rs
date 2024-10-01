use serde::Serialize;

use crate::{game::{DrawPack, Walls}, wall::WallType};

pub enum EntityIndex {
    Player{p: usize},
    Enemy{g: usize, e: usize},
}

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

pub trait Drawable: RadiusTrait {
    fn get_draw_packs(&self) -> &Vec<DrawPack>;
}
#[macro_export]
macro_rules! impl_Drawable {
    ($struct_name:ident) => {
        impl Drawable for $struct_name {
            fn get_draw_packs(&self) -> &Vec<DrawPack> {
                &self.draw_packs
            }
        }
    };
}
pub trait Position {
    fn get_x(&self) -> f32;
    fn get_y(&self) -> f32;
}
#[macro_export]
macro_rules! impl_Position {
    ($struct_name:ident) => {
        impl Position for $struct_name {
            fn get_x(&self) -> f32 {
                self.x
            }
            fn get_y(&self) -> f32 {
                self.y
            }
        }
    };
}

pub trait RadiusTrait {
    fn get_radius(&self) -> f32;
    fn set_radius(&mut self, v: f32);
}
#[macro_export]
macro_rules! impl_RadiusTrait {
    ($struct_name:ident) => {
        impl RadiusTrait for $struct_name {
            fn get_radius(&self) -> f32 {
                self.radius * self.radius_multiplier
            }
            fn set_radius(&mut self, v: f32) {
                self.radius = v;
            }
        }
    };
}

pub trait Moveable: Position + RadiusTrait {
    fn set_pos(&mut self, x: f32, y: f32);
    fn set_velocity(&mut self, v: (f32, f32));
    fn set_speed_multiplier(&mut self, v: f32);
    fn get_velocity(&self) -> (f32, f32);
    fn get_speed_multiplier(&self) -> f32;
    fn get_just_collided(&self) -> bool;
    fn set_just_collided(&mut self, v: bool);
}

#[macro_export]
macro_rules! impl_Moveable {
    ($struct_name:ident) => {
        impl Moveable for $struct_name {
            fn get_velocity(&self) -> (f32, f32) {
                self.velocity
            }
            fn get_speed_multiplier(&self) -> f32 {
                self.speed_multiplier
            }
            fn set_pos(&mut self, x: f32, y: f32) {
                let old = (self.get_x(), self.get_y());
                self.x = x;
                self.y = y;
                self.old_position = old;
            }
            fn set_velocity(&mut self, v: (f32, f32)) {
                self.velocity = v;
            }
            fn set_speed_multiplier(&mut self, v: f32) {
                self.speed_multiplier = v;
            }
            fn get_just_collided(&self) -> bool {
                self.just_collided
            }
            fn set_just_collided(&mut self, v: bool) {
                self.just_collided = v;
            }
        }
    };
}
#[macro_export]
macro_rules! impl_Entity {
    ($struct_name:ident) => {
        impl_RadiusTrait!($struct_name);
        impl_Drawable!($struct_name);
        impl_Position!($struct_name);
        impl_Moveable!($struct_name);
    };
}
