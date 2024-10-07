
use serde::Serialize;

use crate::{game::{DrawPack, Walls}, wall::WallType, Float};

#[derive(PartialEq, Eq, Hash)]
pub enum EntityIndex {
    Player{p: usize},
    Enemy{g: usize, e: usize},
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum Radius {
    Absolute(Float),
    Relative(Float),
}

impl Radius {
    pub fn translate(&self, origin: Float) -> Float {
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
    fn get_x(&self) -> Float;
    fn get_y(&self) -> Float;
    fn get_old(&self) -> (Float, Float);
}
#[macro_export]
macro_rules! impl_Position {
    ($struct_name:ident) => {
        impl Position for $struct_name {
            fn get_x(&self) -> Float {
                self.x
            }
            fn get_y(&self) -> Float {
                self.y
            }
            fn get_old(&self) -> (Float, Float) {
                self.old_position
            }
        }
    };
}

pub trait RadiusTrait {
    fn get_radius(&self) -> Float;
    fn set_radius(&mut self, v: Float);
}
#[macro_export]
macro_rules! impl_RadiusTrait {
    ($struct_name:ident) => {
        impl RadiusTrait for $struct_name {
            fn get_radius(&self) -> Float {
                self.radius * self.radius_multiplier
            }
            fn set_radius(&mut self, v: Float) {
                self.radius = v;
            }
        }
    };
}

pub trait Moveable: Position + RadiusTrait {
    fn set_pos(&mut self, x: Float, y: Float);
    fn set_velocity(&mut self, v: (Float, Float));
    fn set_speed_multiplier(&mut self, v: Float);
    fn get_velocity(&self) -> (Float, Float);
    fn get_speed_multiplier(&self) -> Float;
    fn get_just_collided(&self) -> bool;
    fn set_just_collided(&mut self, v: bool);
}

#[macro_export]
macro_rules! impl_Moveable {
    ($struct_name:ident) => {
        impl Moveable for $struct_name {
            fn get_velocity(&self) -> (Float, Float) {
                (self.velocity.0, self.velocity.1)
            }
            fn get_speed_multiplier(&self) -> Float {
                self.speed_multiplier
            }
            fn set_pos(&mut self, x: Float, y: Float) {
                self.x = x;
                self.y = y;
            }
            fn set_velocity(&mut self, v: (Float, Float)) {
                self.velocity = v;
            }
            fn set_speed_multiplier(&mut self, v: Float) {
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
