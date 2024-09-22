use too_math::{pos2, Pos2, Vec2};
use too_renderer::{Color, Pixel, Shape};

// TODO rename this (something like `quad`)
/// Fill the region with a specific [`Color`]
#[derive(Debug)]
pub struct Fill {
    color: Color,
}

impl Fill {
    /// Create a new [`Fill`] shape
    ///
    /// # Example
    ///
    /// ```rust
    /// # use too_shapes::Fill;
    /// # use too_renderer::{Rgba, Color};
    /// let fill_red = Fill::new("#F00");
    /// let blend_blue_half = Fill::new(Rgba::new(0, 0, 255, 128));
    /// let reset_bg = Fill::new(Color::Reset);
    /// ```
    pub fn new(fill: impl Into<Color>) -> Self {
        Self { color: fill.into() }
    }
}

impl Shape for Fill {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        let pixel = Pixel::new(' ').bg(self.color);
        for y in 0..size.y.max(1) {
            for x in 0..size.x {
                put(pos2(x, y), pixel)
            }
        }
    }
}
