use crate::{
    math::{pos2, Pos2, Vec2},
    Pixel, PixelColor, Shape,
};

// TODO rename this (something like `quad`)
/// Fill the region with a specific [`PixelColor`]
#[derive(Debug)]
pub struct Fill {
    color: PixelColor,
}

impl Fill {
    /// Create a new [`Fill`] shape
    ///
    /// # Example
    ///
    /// ```rust
    /// # use too::{Rgba, PixelColor, shapes::Fill};
    /// let fill_red = Fill::new("#F00");
    /// let blend_blue_half = Fill::new(Rgba::new(0, 0, 255, 128));
    /// let reset_bg = Fill::new(PixelColor::Reset);
    /// ```
    pub fn new(fill: impl Into<PixelColor>) -> Self {
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
