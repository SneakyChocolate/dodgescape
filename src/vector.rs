use std::f32::consts::PI;

use rand::Rng;

pub fn abs(a: (f32, f32)) -> f32 {
    f32::sqrt(f32::powi(a.0, 2) + f32::powi(a.1, 2))
}
pub fn distance(a: (f32, f32), b: (f32, f32)) -> (f32, f32, f32) {
    let (ax, ay) = a;
    let (bx, by) = b;

    let dx = bx - ax;
    let dy = by - ay;

    (dx, dy, abs((dx, dy)))
}
pub fn normalize(a: (f32, f32), value: f32) -> (f32, f32) {
    let dd = distance(a, (0.0, 0.0)).2;
    if dd == 0.0 {
        return (0.0, 0.0);
    }

    (a.0 / dd * value, a.1 / dd * value)
}
pub fn point_from_angle(angle: f32) -> (f32, f32) {
    (f32::cos(angle * PI / 180.0), f32::sin(angle * PI / 180.0))
}
pub fn angle_from_point(point: (f32, f32)) -> f32 {
    let mut r = f32::atan(point.1 / point.0) * 180.0 / PI;
    if point.0 < 0.0 {
        r += 180.0;
    }
    r
}
pub fn collision(position: (f32, f32), velocity: (f32, f32), point: (f32, f32)) -> (f32, f32) {
    if velocity == (0.0, 0.0) {
        return velocity;
    }
    let dist = distance(position, point);
    let angle = angle_from_point((dist.0, dist.1));
    let new_angle = 180.0 + 2.0 * angle - angle_from_point(velocity);
    let new_velocity = point_from_angle(new_angle);
    new_velocity
}
pub fn random_point(center: (f32, f32), distance: (f32, f32)) -> (f32, f32) {
    let angle = rand::thread_rng().gen_range(0.0..360.0);
    let distance = rand::thread_rng().gen_range(distance.0..distance.1);
    (center.0 + point_from_angle(angle).0 * distance, center.1 + point_from_angle(angle).1 * distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_from_point_test() {
        let result = angle_from_point((1.0, 1.0));
        assert_eq!(result, 45.0);
    }
    #[test]
    fn point_from_angle_test() {
        let result = point_from_angle(45.0);
        assert_eq!(45.0, angle_from_point(result));
    }
}

