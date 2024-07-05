
pub fn distance(a: (f32, f32), b: (f32, f32)) -> (f32, f32, f32) {
    let (ax, ay) = a;
    let (bx, by) = b;

    let dx = bx - ax;
    let dy = by - ay;

    (dx, dy, f32::sqrt(f32::powi(dx, 2) + f32::powi(dy, 2)))
}
pub fn normalize(a: (f32, f32), value: f32) -> (f32, f32) {
    let (dx, dy, dd) = distance(a, (0.0, 0.0));
    if dd == 0.0 {
        return (0.0, 0.0);
    }

    (a.0 / dd * value, a.1 / dd * value)
}
