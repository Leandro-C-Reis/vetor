use raylib::prelude::Vector2;

pub fn vector2_rotate(length: f32, angle: f32)  -> Vector2 {
    let cs = (angle.cos() * 100.0).round() / 100.0;
    let sn = (angle.sin() * 100.0).round() / 100.0;

    // Multiply by -1 because coordinate rotation is reversed.
    let x = (-1.0 * length * cs).round();
    let y = (-1.0 * length * sn).round();
    
    Vector2::new(x, y)
}

pub fn vector2_subtract(v1: Vector2, v2: Vector2) -> Vector2 {
    Vector2::new(v1.x - v2.x, v1.y - v2.y)
}

pub fn vector2_add(v1: Vector2, v2: Vector2) -> Vector2 {
    Vector2::new(v1.x + v2.x, v1.y + v2.y)
}

pub trait Vector2Maths {
    fn add(&self, p2: Vector2) -> Vector2;
    fn sub(&self, p2: Vector2) -> Vector2;
}

impl Vector2Maths for Vector2 {
    fn add(&self, p2: Vector2) -> Vector2 {
        vector2_add(*self, p2)
    }

    fn sub(&self, p2: Vector2) -> Vector2 {
        vector2_subtract(*self, p2)
    }
}