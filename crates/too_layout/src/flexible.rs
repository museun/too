use too_math::{midpoint, Rect, Vec2};

use crate::Direction;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MainAlign {
    Start,
    Center,
    End,
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CrossAlign {
    Start,
    Center,
    End,
    Stretch,
}

// row(&mut [a, b, c])
//  .main_align(MainAlign::SpaceBetween)
//  .cross_align(CrossAlign::Center)
//  .calculate(window.rect());
//
// for rect in [a, b, c] {
//   surface.crop(rect).draw(Fill::new(0xFF00FF))
// }

pub struct Flexible<'a> {
    main_align: MainAlign,
    cross_align: CrossAlign,
    spacing: i32,
    direction: Direction,
    children: &'a mut [Rect],
}

impl<'a> Flexible<'a> {
    pub fn direction(direction: Direction, children: &'a mut [Rect]) -> Self {
        Flexible {
            main_align: MainAlign::Start,
            cross_align: CrossAlign::Start,
            spacing: 0,
            direction,
            children,
        }
    }

    pub const fn main_align(mut self, main_align: MainAlign) -> Self {
        self.main_align = main_align;
        self
    }

    pub const fn cross_align(mut self, cross_align: CrossAlign) -> Self {
        self.cross_align = cross_align;
        self
    }

    pub const fn spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn calculate(self, rect: Rect) -> Rect {
        match self.direction {
            Direction::Horizontal => self.horizontal(rect),
            Direction::Vertical => self.vertical(rect),
        }
    }

    fn horizontal(mut self, rect: Rect) -> Rect {
        let len = self.children.len();

        let max_height = self.children.iter().fold(0, |h, r| h.max(r.height()));
        let max_width = self.children.iter().fold(0, |w, r| w.max(r.width()));

        // calculate the cross axis first (the y offset)
        let y = match self.cross_align {
            CrossAlign::Start | CrossAlign::Stretch => 0,
            CrossAlign::Center => midpoint(rect.height(), max_height),
            CrossAlign::End => rect.height() - max_height,
        };

        for child in self.children {
            match self.main_align {
                MainAlign::Start => todo!(),
                MainAlign::Center => todo!(),
                MainAlign::End => todo!(),
                MainAlign::SpaceAround => todo!(),
                MainAlign::SpaceBetween => todo!(),
                MainAlign::SpaceEvenly => todo!(),
            }
        }

        todo!();
    }

    fn vertical(mut self, rect: Rect) -> Rect {
        let len = self.children.len();
        // calculate the cross axis first (the x offset)
        match self.cross_align {
            CrossAlign::Start => todo!(),
            CrossAlign::Center => todo!(),
            CrossAlign::End => todo!(),
            CrossAlign::Stretch => todo!(),
        }
        todo!();
    }
}

pub fn row(children: &mut [Rect]) -> Flexible<'_> {
    Flexible::direction(Direction::Horizontal, children)
}

pub fn column(children: &mut [Rect]) -> Flexible<'_> {
    Flexible::direction(Direction::Vertical, children)
}

fn calculate_spacing(align: MainAlign, children: i32, main_size: i32, total_size: i32) -> Vec2 {
    match align {
        MainAlign::Start => Vec2::ZERO,
        MainAlign::SpaceAround if children == 0 => Vec2::ZERO,
        MainAlign::SpaceBetween if children <= 1 => Vec2::ZERO,

        MainAlign::Center => Vec2::new((total_size - main_size) / 2, 0),
        MainAlign::End => Vec2::new(total_size - main_size, 0),

        MainAlign::SpaceAround => {
            let y = (total_size - main_size) / children;
            Vec2::new(y / 2, y)
        }

        MainAlign::SpaceBetween => Vec2::new(0, (total_size - main_size) / (children - 1)),

        MainAlign::SpaceEvenly => Vec2::splat((total_size - main_size) / (children + 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing() {
        use MainAlign::*;

        for align in [Start, Center, End, SpaceAround, SpaceBetween, SpaceEvenly] {
            let t = calculate_spacing(align, 3, 5, 15);

            eprintln!(
                "{: <12}: {t:?}",
                match align {
                    Start => "Start",
                    Center => "Center",
                    End => "End",
                    SpaceAround => "SpaceAround",
                    SpaceBetween => "SpaceBetween",
                    SpaceEvenly => "SpaceEvenly",
                }
            )
        }
    }
}
