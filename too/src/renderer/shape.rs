use crate::{
    math::{pos2, Pos2, Vec2},
    Pixel,
};

// TODO this needs a `Patch` trait, or at the very least a patch() method

/// Shapes are drawing primitives, like _fill_ or _line_
pub trait Shape {
    /// This allows you to draw into a local __rect__
    ///
    /// `size` is the total size that you're allowed to draw in
    ///
    /// `put` is how you put a pixel as a specific position
    ///
    /// # Example:
    /// This will fill the entire rect with a specific color
    /// ```rust
    /// # use too::{Pixel, Rgba, Shape, pos2, vec2, Pos2, Vec2, Surface, rect};
    /// # let mut surface = Surface::new(vec2(80, 25));
    /// # let mut surface = surface.crop(rect(vec2(80, 25)));
    /// struct FillBg { bg: Rgba }
    /// impl Shape for FillBg {
    ///     fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
    ///         for y in 0..size.y {
    ///             for x in 0..size.x {
    ///                 put(pos2(x, y), Pixel::new(' ').bg(self.bg))
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// surface.draw(FillBg { bg: too::Rgba::hex("#FFF") });
    ///
    /// ```
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel));
}

impl<T: Shape> Shape for &T {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
        <T as Shape>::draw(self, size, put)
    }
}

impl Shape for () {
    fn draw(&self, _: Vec2, _: impl FnMut(Pos2, Pixel)) {}
}

impl Shape for Pixel {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        for y in 0..size.y {
            for x in 0..size.x {
                put(pos2(x, y), *self)
            }
        }
    }
}
