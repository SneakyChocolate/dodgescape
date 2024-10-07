use crate::{vector, Float};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WallType {
    Dirt,
    Wind,
    Flower,
    Water,
    Fire,
    SpawnA,
    SpawnB,
    Blackhole,
    Shooting,
    Explosion,
    Snake,
    Ice,
    Poison,
    Lightning,
    Candy,
    Hell,
}


#[derive(Default, Clone)]
pub struct Wall {
    pub a: (Float, Float),
    pub b: (Float, Float),
    pub d: (Float, Float, Float),
    pub player: bool,
    pub enemy: bool,
}

impl Wall {
    pub fn new(a: (Float, Float), b: (Float, Float), player: bool, enemy: bool) -> Self {
        let mut w = Wall {
            a,
            b,
            player,
            enemy,
            ..Default::default()
        };
        w.d = vector::distance(w.a, w.b);
        w
    }
    pub fn get_percentage(&self, position: &(Float, Float)) -> Float {
        (self.d.0 * position.0 - self.d.0 * self.a.0 + self.d.1 * position.1 - self.d.1 * self.a.1) /
        (Float::powi(self.d.0, 2) + Float::powi(self.d.1, 2))
    }
    pub fn get_point(&self, percentage: Float) -> (Float, Float) {
        (self.a.0 + self.d.0 * percentage, self.a.1 + self.d.1 * percentage)
    }
    pub fn get_nearest_point(&self, position: &(Float, Float)) -> (Float, Float) {
        let percentage = self.get_percentage(&position);
        if percentage < 0.0 {return self.a;}
        if percentage > 1.0 {return self.b;}
        self.get_point(percentage)
    }
}
