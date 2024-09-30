use crate::{
    math::{pos2, rect, Pos2, Vec2},
    Color, Pixel, Shape,
};

#[derive(Copy, Clone, Debug)]
pub struct Border {
    pub left_top: Pixel,
    pub right_top: Pixel,
    pub right_bottom: Pixel,
    pub left_bottom: Pixel,
    pub top: Pixel,
    pub right: Pixel,
    pub bottom: Pixel,
    pub left: Pixel,
}

impl Border {
    pub fn fg(mut self, fg: impl Into<Color> + Copy) -> Self {
        for pixel in [
            &mut self.left_top,
            &mut self.right_top,
            &mut self.right_bottom,
            &mut self.left_bottom,
            &mut self.top,
            &mut self.right,
            &mut self.bottom,
            &mut self.left,
        ] {
            *pixel = pixel.fg(fg);
        }
        self
    }

    pub fn bg(mut self, bg: impl Into<Color> + Copy) -> Self {
        for pixel in [
            &mut self.left_top,
            &mut self.right_top,
            &mut self.right_bottom,
            &mut self.left_bottom,
            &mut self.top,
            &mut self.right,
            &mut self.bottom,
            &mut self.left,
        ] {
            *pixel = pixel.bg(bg);
        }
        self
    }
}

impl Border {
    pub const EMPTY: Self = Self {
        left_top: Pixel::new(' '),
        top: Pixel::new(' '),
        right_top: Pixel::new(' '),
        right: Pixel::new(' '),
        right_bottom: Pixel::new(' '),
        bottom: Pixel::new(' '),
        left_bottom: Pixel::new(' '),
        left: Pixel::new(' '),
    };

    pub const THIN: Self = Self {
        left_top: Pixel::new('┌'),
        top: Pixel::new('─'),
        right_top: Pixel::new('┐'),
        right: Pixel::new('│'),
        right_bottom: Pixel::new('┘'),
        bottom: Pixel::new('─'),
        left_bottom: Pixel::new('└'),
        left: Pixel::new('│'),
    };

    pub const THIN_WIDE: Self = Self {
        left_top: Pixel::new('▁'),
        top: Pixel::new('▁'),
        right_top: Pixel::new('▁'),
        right: Pixel::new('▕'),
        right_bottom: Pixel::new('▔'),
        bottom: Pixel::new('▔'),
        left_bottom: Pixel::new('▔'),
        left: Pixel::new('▏'),
    };

    pub const ROUNDED: Self = Self {
        left_top: Pixel::new('╭'),
        top: Pixel::new('─'),
        right_top: Pixel::new('╮'),
        right: Pixel::new('│'),
        right_bottom: Pixel::new('╯'),
        bottom: Pixel::new('─'),
        left_bottom: Pixel::new('╰'),
        left: Pixel::new('│'),
    };

    pub const DOUBLE: Self = Self {
        left_top: Pixel::new('╔'),
        top: Pixel::new('═'),
        right_top: Pixel::new('╗'),
        right: Pixel::new('║'),
        right_bottom: Pixel::new('╝'),
        bottom: Pixel::new('═'),
        left_bottom: Pixel::new('╚'),
        left: Pixel::new('║'),
    };

    pub const THICK: Self = Self {
        left_top: Pixel::new('┏'),
        top: Pixel::new('━'),
        right_top: Pixel::new('┓'),
        right: Pixel::new('┃'),
        right_bottom: Pixel::new('┛'),
        bottom: Pixel::new('━'),
        left_bottom: Pixel::new('┗'),
        left: Pixel::new('┃'),
    };

    pub const THICK_TALL: Self = Self {
        left_top: Pixel::new('▛'),
        top: Pixel::new('▀'),
        right_top: Pixel::new('▜'),
        right: Pixel::new('▐'),
        right_bottom: Pixel::new('▟'),
        bottom: Pixel::new('▄'),
        left_bottom: Pixel::new('▙'),
        left: Pixel::new('▌'),
    };

    pub const THICK_WIDE: Self = Self {
        left_top: Pixel::new('▗'),
        top: Pixel::new('▄'),
        right_top: Pixel::new('▖'),
        right: Pixel::new('▌'),
        right_bottom: Pixel::new('▘'),
        bottom: Pixel::new('▀'),
        left_bottom: Pixel::new('▝'),
        left: Pixel::new('▐'),
    };
}

impl Shape for Border {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        let rect = rect(size);

        let (left_top, right_top, right_bottom, left_bottom) = (
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        );

        for x in left_top.x..right_top.x {
            put(pos2(x, left_top.y), self.top);
        }

        for x in left_bottom.x..right_bottom.x {
            put(pos2(x, left_bottom.y), self.bottom);
        }

        for y in right_top.y..right_bottom.y {
            put(pos2(right_top.x, y), self.right);
        }

        for y in left_top.y..left_bottom.y {
            put(pos2(left_top.x, y), self.left);
        }

        put(left_top, self.left_top);
        put(right_top, self.right_top);
        put(right_bottom, self.right_bottom);
        put(left_bottom, self.left_bottom);
    }
}
