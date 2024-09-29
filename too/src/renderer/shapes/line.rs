use crate::{pos2, Pixel, Pos2, Rgba, Shape, Vec2};

pub enum Line {
    Vertical(Pixel),
    Horizontal(Pixel),
}

impl Line {
    pub const LIGHT_HORIZONTAL: char = '─';
    pub const HEAVY_HORIZONTAL: char = '━';
    pub const LIGHT_VERTICAL: char = '│';
    pub const HEAVY_VERTICAL: char = '┃';
    pub const LIGHT_TRIPLE_DASH_HORIZONTAL: char = '┄';
    pub const HEAVY_TRIPLE_DASH_HORIZONTAL: char = '┅';
    pub const LIGHT_TRIPLE_DASH_VERTICAL: char = '┆';
    pub const HEAVY_TRIPLE_DASH_VERTICAL: char = '┇';
    pub const LIGHT_QUADRUPLE_DASH_HORIZONTAL: char = '┈';
    pub const HEAVY_QUADRUPLE_DASH_HORIZONTAL: char = '┉';
    pub const LIGHT_QUADRUPLE_DASH_VERTICAL: char = '┊';
    pub const HEAVY_QUADRUPLE_DASH_VERTICAL: char = '┋';

    pub fn vertical(ch: char) -> Self {
        Self::Vertical(Pixel::new(ch))
    }

    pub fn horizontal(ch: char) -> Self {
        Self::Horizontal(Pixel::new(ch))
    }

    pub fn fg(mut self, fg: impl Into<Rgba>) -> Self {
        match &mut self {
            Self::Vertical(p) | Self::Horizontal(p) => *p = p.fg(fg.into()),
        };
        self
    }

    pub fn bg(mut self, bg: impl Into<Rgba>) -> Self {
        match &mut self {
            Self::Vertical(p) | Self::Horizontal(p) => *p = p.bg(bg.into()),
        };
        self
    }
}

impl Shape for Line {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        match self {
            Self::Vertical(pixel) => {
                for y in 0..size.y {
                    put(pos2(0, y), *pixel)
                }
            }
            Self::Horizontal(pixel) => {
                for x in 0..size.x {
                    put(pos2(x, 0), *pixel)
                }
            }
        }
    }
}
