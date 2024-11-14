use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    layout::Align,
    math::{Size, Space},
    measure_text,
    view::{Builder, Interest, Layout, Palette, Render, StyleKind, View},
    Border, Grapheme, Pixel, Rgba,
};

pub type BorderClass = fn(&Palette, bool, bool) -> BorderStyle;

#[derive(Copy, Clone, Debug)]
pub struct BorderStyle {
    pub title: Rgba,
    pub border: Rgba,
    pub border_focused: Option<Rgba>,
    pub border_hovered: Option<Rgba>,
}

impl BorderStyle {
    pub fn default(palette: &Palette, hovered: bool, focused: bool) -> Self {
        Self {
            title: palette.foreground,
            border: palette.outline,
            border_focused: None,
            border_hovered: None,
        }
    }

    pub fn interactive(palette: &Palette, hovered: bool, focused: bool) -> Self {
        Self {
            border_focused: Some(palette.contrast),
            border_hovered: Some(palette.secondary),
            ..Self::default(palette, hovered, focused)
        }
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct BorderView {
    border: Border,
    title: Option<CompactString>,
    align: Align,
    class: StyleKind<BorderClass, BorderStyle>,
}

impl std::fmt::Debug for BorderView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorderView")
            .field("title", &self.title)
            .field("align", &self.align)
            .field("class", &self.class)
            .finish()
    }
}

impl BorderView {
    pub const fn border(mut self, border: Border) -> Self {
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

    pub const fn class(mut self, class: BorderClass) -> Self {
        self.class = StyleKind::deferred(class);
        self
    }

    pub const fn style(mut self, style: BorderStyle) -> Self {
        self.class = StyleKind::direct(style);
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

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let mut margin = self.border.as_margin();
        if margin.top == 0 && self.title.is_some() {
            margin.top = 1;
        }

        let sum = margin.sum();
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

        let max = size.max(title_size) + Size::new(1.0, 0.0);
        space.fit(max)
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();
        let (w, h) = (rect.width() - 1, rect.height() - 1);

        let is_hovered = render.is_hovered();
        let is_focused = render.is_focused();

        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, is_hovered, is_focused),
            StyleKind::Direct(style) => style,
        };

        let color = match (is_focused, is_hovered) {
            (true, true) => style
                .border_focused
                .unwrap_or(style.border_hovered.unwrap_or(style.border)),
            (true, false) => style.border_focused.unwrap_or(style.border),
            (false, true) => style.border_hovered.unwrap_or(style.border),
            (false, false) => style.border,
        };

        let pixel = Pixel::new(self.border.top).fg(color);
        for x in 1..=w {
            render.surface.set((x, 0), pixel);
        }

        let pixel = Pixel::new(self.border.bottom).fg(color);
        for x in 1..=w {
            render.surface.set((x, h), pixel);
        }

        let pixel = Pixel::new(self.border.left).fg(color);
        for y in 1..=h {
            render.surface.set((0, y), pixel);
        }

        let pixel = Pixel::new(self.border.right).fg(color);
        for y in 1..=h {
            render.surface.set((w, y), pixel);
        }

        let pixel = Pixel::new(self.border.left_top).fg(color);
        render.surface.set((0, 0), pixel);

        let pixel = Pixel::new(self.border.right_top).fg(color);
        render.surface.set((w, 0), pixel);

        let pixel = Pixel::new(self.border.left_bottom).fg(color);
        render.surface.set((0, h), pixel);

        let pixel = Pixel::new(self.border.right_bottom).fg(color);
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
            let fg = style.title;
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

pub fn border(border: Border) -> BorderView {
    BorderView {
        border,
        title: None,
        align: Align::Min,
        class: StyleKind::deferred(BorderStyle::default),
    }
}

pub fn frame(border: Border, title: impl ToCompactString) -> BorderView {
    BorderView {
        border,
        title: Some(title.to_compact_string()),
        align: Align::Min,
        class: StyleKind::deferred(BorderStyle::default),
    }
}
