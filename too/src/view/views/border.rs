use crate::{
    layout::Align,
    view::{
        geom::{Margin, Size, Space, Vector},
        text::Text,
        view::Context,
        DrawCtx, LayoutCtx, UpdateCtx, View, ViewExt,
    },
    Attribute, MeasureText, Pixel,
};

#[derive(Copy, Clone)]
pub struct Border {
    pub left_top: char,
    pub top: char,
    pub right_top: char,

    pub left: char,
    pub right: char,

    pub left_bottom: char,
    pub bottom: char,
    pub right_bottom: char,
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

pub struct BorderTitle<'a> {
    pub title: &'a str,
    pub align: Align,
}

impl<'a> BorderTitle<'a> {
    pub const fn title(title: &'a str) -> Self {
        Self {
            title,
            align: Align::Center,
        }
    }

    pub const fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

// more closures
struct BorderParams<T: 'static> {
    title: Option<for<'a> fn(&'a T) -> BorderTitle<'a>>,
    style: Border,
}

impl<T: 'static> Copy for BorderParams<T> {}

impl<T: 'static> Clone for BorderParams<T> {
    fn clone(&self) -> Self {
        *self
    }
}

struct BorderView<T: 'static> {
    params: BorderParams<T>,
}

impl<T: 'static> View<T> for BorderView<T> {
    type Args<'a> = BorderParams<T>;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let style = &self.params.style;

        const fn has(c: char) -> bool {
            c != ' '
        }

        let margin = Margin::new(
            (has(style.left_top) && has(style.left) && has(style.left_bottom)) as u8 as f32,
            (self.params.title.is_some()
                || (has(style.left_top) && has(style.top) && has(style.right_top)))
                as u8 as f32,
            (has(style.right_top) && has(style.right) && has(style.right_bottom)) as u8 as f32,
            (has(style.left_bottom) && has(style.bottom) && has(style.right_bottom)) as u8 as f32,
        );

        let sum = margin.sum();
        let offset = margin.left_top().to_point();
        let space = Space {
            min: (space.min - sum).max(Size::ZERO),
            max: (space.max - sum).max(Size::ZERO),
        };

        let mut size = Size::ZERO;
        for &child in ctx.children {
            size = ctx.compute_layout(child, space) + sum;
            ctx.translate_pos(child, offset);
        }
        space.min.max(size.max(sum))
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let w = ctx.rect.width() - 1.0;
        let h = ctx.rect.height() - 1.0;

        let fg = ctx.theme.outline;

        let pixel = Pixel::new(self.params.style.top).fg(fg);
        ctx.surface.horizontal_fill((1.0, w), pixel);

        let pixel = Pixel::new(self.params.style.bottom).fg(fg);
        ctx.surface.horizontal_fill_offset((1.0, w), h, pixel);

        let pixel = Pixel::new(self.params.style.left).fg(fg);
        ctx.surface.vertical_fill((1.0, h), pixel);

        let pixel = Pixel::new(self.params.style.right).fg(fg);
        ctx.surface.vertical_fill_offset((1.0, h), w, pixel);

        let pixel = Pixel::new(self.params.style.left_top).fg(fg);
        ctx.surface.set((0.0, 0.0), pixel);

        let pixel = Pixel::new(self.params.style.right_top).fg(fg);
        ctx.surface.set((w, 0.0), pixel);

        let pixel = Pixel::new(self.params.style.left_bottom).fg(fg);
        ctx.surface.set((0.0, h), pixel);

        let pixel = Pixel::new(self.params.style.right_bottom).fg(fg);
        ctx.surface.set((w, h), pixel);

        if let Some(get) = self.params.title {
            let title = get(&ctx.state);
            let tw = MeasureText::measure(&title.title) as f32;

            let x = match title.align {
                Align::Min => 1.0,
                Align::Center => (w - tw) / 2.0,
                Align::Max => w - tw,
            };

            Text {
                data: title.title,
                fg: ctx.theme.foreground,
                bg: crate::Color::Reuse,
                attribute: Attribute::RESET,
            }
            .draw(ctx.rect + Vector::new(x, 0.0), ctx.surface.surface_raw());
        }

        self.default_draw(ctx);
    }
}

pub fn border<T: 'static, R>(
    ctx: &mut Context<T>,
    style: Border,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let (_, resp) = BorderView::show_children(BorderParams { title: None, style }, ctx, show);
    resp
}

pub fn frame<T: 'static, R>(
    ctx: &mut Context<T>,
    style: Border,
    // TODO has to be a closure. every time its a closure
    title: for<'a> fn(&'a T) -> BorderTitle<'a>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let args = BorderParams {
        title: Some(title),
        style,
    };
    let (_, resp) = BorderView::show_children(args, ctx, show);
    resp
}
