use compact_str::CompactString;
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

#[allow(deprecated)]
use crate::view::measure_text;

use crate::{
    layout::Align,
    math::{pos2, Size, Space},
    renderer::{Border, Grapheme, Pixel, Rgba},
    view::{ApplicableStyle, Builder, Interest, Layout, Palette, Render, Style, View},
    Str,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BorderStyleArgs {
    pub hovered: bool,
    pub focused: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct BorderStyle {
    pub title: Rgba,
    pub border: Rgba,
    pub border_focused: Option<Rgba>,
    pub border_hovered: Option<Rgba>,
}

impl Style for BorderStyle {
    type Args = BorderStyleArgs;
    fn default(palette: &Palette, _args: Self::Args) -> Self {
        Self {
            title: palette.foreground,
            border: palette.outline,
            border_focused: None,
            border_hovered: None,
        }
    }
}

impl BorderStyle {
    pub fn interactive(palette: &Palette, hovered: bool, focused: bool) -> Self {
        Self {
            border_focused: Some(palette.contrast),
            border_hovered: Some(palette.secondary),
            ..Self::default(palette, BorderStyleArgs { hovered, focused })
        }
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Frame {
    border: Border,
    title: Option<CompactString>,
    align: Align,
    style: ApplicableStyle<BorderStyle>,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorderView")
            .field("title", &self.title)
            .field("align", &self.align)
            .field("style", &self.style)
            .finish()
    }
}

impl Frame {
    pub const fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    pub fn title(mut self, title: impl Into<Str>) -> Self {
        self.title = Some(title.into().into_inner());
        self
    }

    pub const fn title_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl<'v> Builder<'v> for Frame {
    type View = Self;
    type Style = BorderStyle;

    fn style(mut self, style: Self::Style) -> Self {
        self.style = ApplicableStyle::value(style);
        self
    }

    fn class(mut self, style: impl Fn(&Palette, BorderStyleArgs) -> Self::Style + 'static) -> Self {
        self.style = ApplicableStyle::new(style);
        self
    }
}

impl View for Frame {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    // TODO frames should have a 'click to focus' option

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let mut margin = self.border.as_margin();
        if margin.top == 0 && self.title.is_some() {
            margin.top = 1;
        }

        let sum = margin.sum();
        let offset = margin.left_top();
        let child_space = space.shrink(sum).loosen();

        let node = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &node.children {
            size = layout.compute(child, child_space);
            layout.set_position(child, offset);
        }

        #[allow(deprecated)]
        let title_size = self
            .title
            .as_deref()
            .map_or(Size::ZERO, |c| measure_text(c) + Size::new(2.0, 0.0));

        space.fit((size + sum).max(title_size))
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.rect();
        let (w, h) = (rect.width() - 1, rect.height() - 1);

        let is_hovered = render.is_hovered();
        let is_focused = render.is_focused();

        let style = self.style.apply(
            render.palette,
            BorderStyleArgs {
                hovered: is_hovered,
                focused: is_focused,
            },
        );

        let color = match (is_focused, is_hovered) {
            (true, true) => style
                .border_focused
                .unwrap_or(style.border_hovered.unwrap_or(style.border)),
            (true, false) => style.border_focused.unwrap_or(style.border),
            (false, true) => style.border_hovered.unwrap_or(style.border),
            (false, false) => style.border,
        };

        render
            .horizontal_line(0, 1..=w, Pixel::new(self.border.top).fg(color))
            .horizontal_line(h, 1..=w, Pixel::new(self.border.bottom).fg(color))
            .vertical_line(0, 1..=h, Pixel::new(self.border.left).fg(color))
            .vertical_line(w, 1..=h, Pixel::new(self.border.right).fg(color))
            .set(pos2(0, 0), Pixel::new(self.border.left_top).fg(color))
            .set(pos2(w, 0), Pixel::new(self.border.right_top).fg(color))
            .set(pos2(0, h), Pixel::new(self.border.left_bottom).fg(color))
            .set(pos2(w, h), Pixel::new(self.border.right_bottom).fg(color));

        // XXX this is actually a valid use of `measure_text`
        // we don't really want to delegate to the label type because we do that
        // weird intersperse border-behind-title things
        if let Some(title) = &self.title {
            #[allow(deprecated)]
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
                render.set((start + x, 0.0), cell);
                start += grapheme.width() as f32
            }
        }

        self.default_draw(render);
    }
}

pub fn border(border: Border) -> Frame {
    Frame {
        border,
        title: None,
        align: Align::Min,
        style: ApplicableStyle::default(),
    }
}

pub fn frame(border: Border, title: impl Into<Str>) -> Frame {
    Frame {
        border,
        title: Some(title.into().into_inner()),
        align: Align::Min,
        style: ApplicableStyle::default(),
    }
}
