use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    layout::Align,
    view::{
        geom::{Margin, Size, Space},
        Builder, Layout, Render, View,
    },
    Grapheme, Pixel,
};

// TODO move this out into the main module
#[derive(Copy, Clone, PartialEq)]
pub struct Border {
    pub left_top: char,
    pub top: char,
    pub right_top: char,
    pub right: char,
    pub right_bottom: char,
    pub bottom: char,
    pub left_bottom: char,
    pub left: char,
}

impl std::fmt::Debug for Border {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Border")
            .field("left_top", &format_args!("0x{:04X}", self.left_top as u32))
            .field("top", &format_args!("0x{:04X}", self.top as u32))
            .field(
                "right_top",
                &format_args!("0x{:04X}", self.right_top as u32),
            )
            .field("right", &format_args!("0x{:04X}", self.right as u32))
            .field(
                "right_bottom",
                &format_args!("0x{:04X}", self.right_bottom as u32),
            )
            .field("bottom", &format_args!("0x{:04X}", self.bottom as u32))
            .field(
                "left_bottom",
                &format_args!("0x{:04X}", self.left_bottom as u32),
            )
            .field("left", &format_args!("0x{:04X}", self.left as u32))
            .finish()
    }
}

impl Border {
    pub const fn without_top(self) -> Self {
        Self {
            left_top: self.left,
            right_top: self.right,
            top: ' ',
            ..self
        }
    }

    pub const fn without_bottom(self) -> Self {
        Self {
            left_bottom: self.left,
            right_bottom: self.right,
            bottom: ' ',
            ..self
        }
    }

    pub const fn without_left(self) -> Self {
        Self {
            left_top: self.top,
            left: ' ',
            left_bottom: self.bottom,
            ..self
        }
    }

    pub const fn without_right(self) -> Self {
        Self {
            right_top: self.top,
            right: ' ',
            right_bottom: self.bottom,
            ..self
        }
    }

    pub const fn without_corners(self) -> Self {
        Self {
            left_top: self.top,
            right_top: self.top,
            left_bottom: self.bottom,
            right_bottom: self.bottom,
            ..self
        }
    }

    const fn has(c: char) -> bool {
        c != ' '
    }

    pub const fn has_left(&self) -> bool {
        Self::has(self.left) | (Self::has(self.left_top) && Self::has(self.left_bottom))
    }

    pub const fn has_top(&self) -> bool {
        Self::has(self.top) | (Self::has(self.left_top) && Self::has(self.right_top))
    }

    pub const fn has_right(&self) -> bool {
        Self::has(self.right) | (Self::has(self.right_top) && Self::has(self.right_bottom))
    }

    pub const fn has_bottom(&self) -> bool {
        Self::has(self.bottom) | (Self::has(self.left_bottom) && Self::has(self.right_bottom))
    }

    pub fn as_margin(&self) -> Margin {
        Margin::new(
            self.has_left() as i32,
            self.has_top() as i32,
            self.has_right() as i32,
            self.has_bottom() as i32,
        )
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::THICK
    }
}

impl Border {
    pub const EMPTY: Self = Self {
        left_top: ' ',
        top: ' ',
        right_top: ' ',
        right: ' ',
        right_bottom: ' ',
        bottom: ' ',
        left_bottom: ' ',
        left: ' ',
    };

    pub const THIN: Self = Self {
        left_top: '┌',
        top: '─',
        right_top: '┐',
        right: '│',
        right_bottom: '┘',
        bottom: '─',
        left_bottom: '└',
        left: '│',
    };

    pub const THIN_WIDE: Self = Self {
        left_top: '▁',
        top: '▁',
        right_top: '▁',
        right: '▕',
        right_bottom: '▔',
        bottom: '▔',
        left_bottom: '▔',
        left: '▏',
    };

    pub const ROUNDED: Self = Self {
        left_top: '╭',
        top: '─',
        right_top: '╮',
        right: '│',
        right_bottom: '╯',
        bottom: '─',
        left_bottom: '╰',
        left: '│',
    };

    pub const DOUBLE: Self = Self {
        left_top: '╔',
        top: '═',
        right_top: '╗',
        right: '║',
        right_bottom: '╝',
        bottom: '═',
        left_bottom: '╚',
        left: '║',
    };

    pub const THICK: Self = Self {
        left_top: '┏',
        top: '━',
        right_top: '┓',
        right: '┃',
        right_bottom: '┛',
        bottom: '━',
        left_bottom: '┗',
        left: '┃',
    };

    pub const THICK_TALL: Self = Self {
        left_top: '▛',
        top: '▀',
        right_top: '▜',
        right: '▐',
        right_bottom: '▟',
        bottom: '▄',
        left_bottom: '▙',
        left: '▌',
    };

    pub const THICK_WIDE: Self = Self {
        left_top: '▗',
        top: '▄',
        right_top: '▖',
        right: '▌',
        right_bottom: '▘',
        bottom: '▀',
        left_bottom: '▝',
        left: '▐',
    };
}

use super::measure_text;

// TODO 'hoverable' and 'focusable'
#[derive(Default, Debug, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct BorderView {
    border: Border,
    title: Option<CompactString>,
    align: Align,
}

impl BorderView {
    pub const fn style(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    pub fn title(mut self, title: impl ToCompactString) -> Self {
        self.title = Some(title.to_compact_string());
        self
    }

    pub const fn title_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl<'v> Builder<'v> for BorderView {
    type View = Self;
}

impl View for BorderView {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let mut margin = self.border.as_margin();
        if margin.top == 0 && self.title.is_some() {
            margin.top = 1;
        }

        let sum = margin.size();

        let offset = margin.left_top();
        let child_space = space.shrink(sum);

        let node = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &node.children {
            size = layout.compute(child, child_space) + sum;
            layout.set_position(child, offset);
        }

        let title_size = self
            .title
            .as_deref()
            .map(measure_text)
            .unwrap_or(Size::ZERO);

        // TODO this needs some padding for the title_size
        size.max(title_size) + sum
    }

    // TODO refactor this out so its on Border, so a Surface can draw a border natively
    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();
        let (w, h) = (rect.width() - 1, rect.height() - 1);

        // TODO property for this
        // let fg = if render.is_focused() {
        //     render.theme.accent
        // } else {
        //     render.theme.outline
        // };

        let fg = render.theme.outline;

        let pixel = Pixel::new(self.border.top).fg(fg);
        for x in 1..=w {
            render.surface.set((x, 0), pixel);
        }

        let pixel = Pixel::new(self.border.bottom).fg(fg);
        for x in 1..=w {
            render.surface.set((x, h), pixel);
        }

        let pixel = Pixel::new(self.border.left).fg(fg);
        for y in 1..=h {
            render.surface.set((0, y), pixel);
        }

        let pixel = Pixel::new(self.border.right).fg(fg);
        for y in 1..=h {
            render.surface.set((w, y), pixel);
        }

        let pixel = Pixel::new(self.border.left_top).fg(fg);
        render.surface.set((0, 0), pixel);

        let pixel = Pixel::new(self.border.right_top).fg(fg);
        render.surface.set((w, 0), pixel);

        let pixel = Pixel::new(self.border.left_bottom).fg(fg);
        render.surface.set((0, h), pixel);

        let pixel = Pixel::new(self.border.right_bottom).fg(fg);
        render.surface.set((w, h), pixel);

        if let Some(title) = &self.title {
            let tw = measure_text(title);

            let w = w as f32;
            let x = match self.align {
                Align::Min => 1.0,
                Align::Center => (w - tw.width) / 2.0,
                Align::Max => w - tw.width,
            };

            let mut start = 0.0;
            let fg = render.theme.foreground;
            for grapheme in title.graphemes(true) {
                if grapheme.chars().all(|c| c.is_whitespace()) {
                    start += grapheme.width() as f32;
                    continue;
                }
                let cell = Grapheme::new(grapheme).fg(fg);
                render.surface.set((start + x, 0.0), cell);
                start += grapheme.width() as f32
            }
        }

        self.default_draw(render);
    }
}

pub fn border() -> BorderView {
    BorderView::default()
}

pub fn frame(title: impl ToCompactString) -> BorderView {
    BorderView::default().title(title)
}
