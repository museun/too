use crate::{
    math::{pos2, rect, Pos2, Rect, Vec2},
    Color, Pixel,
};

pub struct Buffer {
    pub(crate) pixels: Vec<Pixel>,
    size: Vec2,
}

impl Buffer {
    pub fn new(size: Vec2) -> Self {
        Self {
            pixels: vec![Pixel::EMPTY; size.x as usize * size.y as usize],
            size,
        }
    }

    pub fn resize(&mut self, size: Vec2, pixel: Pixel) {
        if self.size == size {
            return;
        }

        *self = Self {
            pixels: vec![pixel; size.x as usize * size.y as usize],
            size,
        }
    }

    pub fn reset(&mut self) {
        for x in &mut self.pixels {
            *x = Pixel::EMPTY
        }
    }

    pub const fn rect(&self) -> Rect {
        rect(self.size)
    }

    pub const fn contains(&self, pos: Pos2) -> bool {
        pos.x < self.size.x && pos.y < self.size.y
    }

    pub fn get(&self, pos: Pos2) -> Option<&Pixel> {
        self.pixels.get(Self::pos_to_index(pos, self.size.x))
    }

    pub fn get_mut(&mut self, pos: Pos2) -> Option<&mut Pixel> {
        self.pixels.get_mut(Self::pos_to_index(pos, self.size.x))
    }

    pub fn diff<'a>(&'a mut self, other: &'a Self) -> impl Iterator<Item = (Pos2, &'a Pixel)> {
        self.pixels
            .iter_mut()
            .zip(other.pixels.iter())
            .enumerate()
            .filter_map(|(i, (left, right))| {
                if *left == *right || (right.fg == Color::Reuse && right.bg == Color::Reuse) {
                    return None;
                }
                *left = *right;
                // TODO when switching to a stack allocated string
                // we need to advance the index by UnicodeWidth::width(str)
                Some((Self::index_to_pos(i, self.size.x), right))
            })
    }

    const fn pos_to_index(pos: Pos2, w: i32) -> usize {
        (pos.y * w + pos.x) as usize
    }

    const fn index_to_pos(index: usize, w: i32) -> Pos2 {
        let index = index as i32;
        pos2(index % w, index / w)
    }
}

impl std::ops::Index<Pos2> for Buffer {
    type Output = Pixel;
    fn index(&self, index: Pos2) -> &Self::Output {
        &self.pixels[Self::pos_to_index(index, self.size.x)]
    }
}

impl std::ops::IndexMut<Pos2> for Buffer {
    fn index_mut(&mut self, index: Pos2) -> &mut Self::Output {
        &mut self.pixels[Self::pos_to_index(index, self.size.x)]
    }
}
