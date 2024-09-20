use too_math::{Pos2, Vec2};

use crate::Pixel;

// TODO this needs a `Patch` trait, or at the very least a patch() method

pub trait Shape {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel));
}

impl<T: Shape> Shape for &T {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
        <T as Shape>::draw(self, size, put)
    }
}
