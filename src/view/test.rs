use std::{borrow::Cow, ops::RangeInclusive};

use crate::{
    layout::Axis,
    math::{vec2, Pos2, Rect, Vec2},
    rasterizer::{Rasterizer, TextShape},
    view::{Palette, State, ViewId},
    Animations, Cell, Event, Grapheme, Modifiers, MouseButton, Pixel, Rgba,
};

use super::Ui;

#[derive(Default)]
struct DebugRasterizer {
    current: ViewId,
    paint_list: Vec<(ViewId, Shape)>,
    rect: Rect,
}

impl DebugRasterizer {
    fn push_shape(&mut self, shape: Shape) {
        self.paint_list.push((self.current, shape));
    }
}

impl Rasterizer for DebugRasterizer {
    fn begin(&mut self, id: ViewId) {
        self.current = id;
    }

    fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn clear(&mut self, color: Rgba) {}

    fn fill_bg(&mut self, color: Rgba) {
        self.push_shape(Shape::FillBg {
            rect: self.rect,
            color,
        });
    }

    fn fill_with(&mut self, pixel: Pixel) {
        self.push_shape(Shape::FillWith {
            rect: self.rect,
            pixel,
        });
    }

    fn line(&mut self, axis: Axis, offset: Pos2, range: RangeInclusive<i32>, pixel: Pixel) {
        let cross: i32 = axis.cross(offset);
        let start: Pos2 = axis.pack(*range.start(), cross);
        let end: Pos2 = axis.pack(*range.end(), cross);
        self.push_shape(Shape::Line { start, end, pixel });
    }

    fn text(&mut self, shape: TextShape<'_>) {
        let shape = TextShape {
            label: Cow::from(shape.label.to_string()),
            ..shape
        };

        self.push_shape(Shape::Text {
            rect: self.rect,
            shape,
        });
    }

    fn pixel(&mut self, pos: Pos2, pixel: Pixel) {
        self.push_shape(Shape::Set {
            pos,
            cell: Cell::Pixel(pixel),
        });
    }

    fn grapheme(&mut self, pos: Pos2, grapheme: Grapheme) {
        self.push_shape(Shape::Set {
            pos,
            cell: Cell::Grapheme(grapheme),
        });
    }

    fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell> {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    FillBg {
        rect: Rect,
        color: Rgba,
    },
    FillWith {
        rect: Rect,
        pixel: Pixel,
    },
    Line {
        start: Pos2,
        end: Pos2,
        pixel: Pixel,
    },
    Text {
        rect: Rect,
        shape: TextShape<'static>,
    },
    Set {
        pos: Pos2,
        cell: Cell,
    },
}

pub enum TestInput {
    Held { pos: Pos2 },
    Click { pos: Pos2 },
    MouseMove { pos: Pos2 },
    Drag { start: Pos2, delta: Vec2 },
}

#[derive(Debug)]
pub struct TestOutput<R: 'static> {
    pub response: R,
    pub shapes: Vec<(ViewId, Shape)>,
}

pub fn test_view<R>(
    events: impl IntoIterator<Item = TestInput>,
    mut show: impl FnMut(&Ui) -> R,
) -> TestOutput<R>
where
    R: 'static,
{
    let mut state = State::new(Palette::dark(), Animations::new());
    state.build(Rect::from_min_size(Pos2::ZERO, vec2(80, 25)), &mut show);

    // TODO make this less repetitive and make it more extensible
    for event in events {
        match event {
            TestInput::Held { pos } => {
                state.event(&Event::MouseMove { pos });
                state.event(&Event::MouseButtonChanged {
                    pos,
                    button: MouseButton::Primary,
                    down: true,
                    modifiers: Modifiers::NONE,
                });
            }

            TestInput::Click { pos } => {
                state.event(&Event::MouseMove { pos });
                state.event(&Event::MouseButtonChanged {
                    pos,
                    button: MouseButton::Primary,
                    down: true,
                    modifiers: Modifiers::NONE,
                });
                state.event(&Event::MouseButtonChanged {
                    pos,
                    button: MouseButton::Primary,
                    down: false,
                    modifiers: Modifiers::NONE,
                });
            }
            TestInput::MouseMove { pos } => {
                state.event(&Event::MouseMove { pos });
            }
            TestInput::Drag { start, delta } => {
                state.event(&Event::MouseMove { pos: start });
                state.event(&Event::MouseButtonChanged {
                    pos: start,
                    button: MouseButton::Primary,
                    down: true,
                    modifiers: Modifiers::NONE,
                });
                state.event(&Event::MouseMove { pos: start + delta });
            }
        }
    }

    let response = state.build(Rect::from_min_size(Pos2::ZERO, vec2(80, 25)), show);

    let mut debug = DebugRasterizer::default();
    state.render(&mut debug);

    TestOutput {
        response,
        shapes: debug.paint_list,
    }
}
