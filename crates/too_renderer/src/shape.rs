use too_math::{Pos2, Vec2};

use crate::Pixel;

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
    /// ```rust,no_run
    /// struct FillBg { bg: Rgba }
    /// impl Shape for FillBg {
    ///     fn draw(&self, size: Vec2, mut put: impl Fnmut(Pos2, Pixel)) {
    ///         for y in 0..size.y {
    ///             for x in 0..size.x {
    ///                 put(pos2(x, y), Pixel::new(' ').bg(self.bg))
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel));
}

impl<T: Shape> Shape for &T {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
        <T as Shape>::draw(self, size, put)
    }
}
