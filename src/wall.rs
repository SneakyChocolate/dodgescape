use crate::vector;

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
}


#[derive(Default)]
pub struct Wall {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub d: (f32, f32, f32),
    pub player: bool,
    pub enemy: bool,
}

impl Wall {
    pub fn new(a: (f32, f32), b: (f32, f32), player: bool, enemy: bool) -> Self {
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
    pub fn get_percentage(&self, position: &(f32, f32)) -> f32 {
        (self.d.0 * position.0 - self.d.0 * self.a.0 + self.d.1 * position.1 - self.d.1 * self.a.1) /
        (f32::powi(self.d.0, 2) + f32::powi(self.d.1, 2))
    }
    pub fn get_point(&self, percentage: f32) -> (f32, f32) {
        (self.a.0 + self.d.0 * percentage, self.a.1 + self.d.1 * percentage)
    }
    pub fn get_nearest_point(&self, position: &(f32, f32)) -> (f32, f32) {
        let percentage = self.get_percentage(&position);
        if percentage < 0.0 {return self.a;}
        if percentage > 1.0 {return self.b;}
        self.get_point(percentage)
    }
}
