use std::f32::consts::PI;

use rand::Rng;

use crate::math::Matrix;

pub type Point = (f32, f32);

#[derive(Debug)]
pub struct Line {
    pub dir: Point,
    pub start: Point,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.dir.0 == other.dir.0 &&
        self.start.0 == other.start.0 &&
        self.dir.1 == other.dir.1 &&
        self.start.1 == other.start.1
    }
}

impl Line {
    pub fn new(dir: Point, start: Point) -> Self {
        Self {
            dir,
            start,
        }
    }
    pub fn from_points(a: Point, b: Point) -> Self {
        let dist = distance(a, b);
        Self {
            dir: (dist.0, dist.1),
            start: a,
        }
    }
    pub fn point(&self, x: f32) -> Point {
        (self.dir.0 * x + self.start.0, self.dir.1 * x + self.start.1)
    }
}

pub fn abs(a: Point) -> f32 {
    f32::sqrt(f32::powi(a.0, 2) + f32::powi(a.1, 2))
}
pub fn distance(a: Point, b: Point) -> (f32, f32, f32) {
    let (ax, ay) = a;
    let (bx, by) = b;

    let dx = bx - ax;
    let dy = by - ay;

    (dx, dy, abs((dx, dy)))
}
pub fn normalize(a: Point, value: f32) -> Point {
    let dd = distance(a, (0.0, 0.0)).2;
    if dd == 0.0 {
        return (0.0, 0.0);
    }

    (a.0 / dd * value, a.1 / dd * value)
}
pub fn normalize_mut(a: &mut Point, value: f32) {
    let dd = distance(*a, (0.0, 0.0)).2;
    if dd == 0.0 {
        a.0 = 0.0;
        a.1 = 0.0;
        return;
    }

    a.0 = a.0 / dd * value;
    a.1 = a.1 / dd * value;
}
pub fn point_from_angle(angle: f32) -> Point {
    (f32::cos(angle * PI / 180.0), f32::sin(angle * PI / 180.0))
}
pub fn angle_from_point(point: Point) -> f32 {
    let mut r = f32::atan(point.1 / point.0) * 180.0 / PI;
    if point.0 < 0.0 {
        r += 180.0;
    }
    r
}
pub fn collision(position: Point, velocity: Point, point: Point) -> Point {
    if velocity == (0.0, 0.0) {
        return velocity;
    }
    let dist = distance(position, point);
    let angle = angle_from_point((dist.0, dist.1));
    let new_angle = 180.0 + 2.0 * angle - angle_from_point(velocity);
    let new_velocity = point_from_angle(new_angle);
    new_velocity
}
pub fn random_point(center: Point, distance: Point) -> Point {
    let angle = rand::thread_rng().gen_range(0.0..360.0);
    let distance = rand::thread_rng().gen_range(distance.0..distance.1);
    (center.0 + point_from_angle(angle).0 * distance, center.1 + point_from_angle(angle).1 * distance)
}
pub fn lgs(a: Line, b: Line) -> Matrix {
    let mut matrix = vec![
        vec![a.dir.0, -b.dir.0, b.start.0 - a.start.0],
        vec![a.dir.1, -b.dir.1, b.start.1 - a.start.1],
    ];
    matrix
    
}
pub fn get_intersection(a: Line, b: Line) -> Option<(f32, f32)> {
    let mut matrix = lgs(a, b);
    crate::math::matrix::normalize(&mut matrix);
    if matrix[0][0] == 0.0 || matrix[1][1] == 0.0 {
        return None;
    }
    Some((matrix[0][2], matrix[1][2]))
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
    
    #[test]
    fn lgs_test() {
        let a = Line::from_points((0.0, 0.0), (0.0, 1.0));
        let b = Line::from_points((3.0, 0.0), (8.0, 0.0));
        let result = lgs(a, b);
        let ex = vec![
            vec![0.0, -5.0, 3.0],
            vec![1.0, 0.0, 0.0],
        ];
        assert_eq!(result, ex);
    }
    
    #[test]
    fn intersection_test() {
        let a = Line::from_points((0.0, 4.0), (2.0, 0.0));
        let b = Line::from_points((1.0, 4.0), (1.0, 0.0));
        let result = get_intersection(a, b);
        assert_eq!(result, Some((0.5, 0.5)));
    }
    
    #[test]
    fn intersection_test_negative() {
        let a = Line::from_points((0.0, 0.0), (0.0, 1.0));
        let b = Line::from_points((3.0, 0.0), (8.0, 0.0));
        let result = get_intersection(b, a);
        assert_eq!(result, Some((-3.0 / 5.0, 0.0)));
    }

    #[test]
    fn intersection_test_no_intersection() {
        let a = Line::from_points((0.0, 1.0), (1.0, 1.0));
        let b = Line::from_points((0.0, 0.0), (1.0, 0.0));
        let result = get_intersection(a, b);
        assert_eq!(result, None);
    }
}

