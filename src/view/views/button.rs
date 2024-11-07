use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    view::{
        geom::{Margin, Size, Space},
        style::StyleKind,
        views::measure_text,
        Builder, EventCtx, Handled, Interest, Layout, Palette, Render, Ui, View, ViewEvent,
    },
    Grapheme, Rgba,
};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum State {
    Hovered,
    Held,
    Clicked,
    Disabled,
    #[default]
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct ButtonStyle {
    pub text_color: Rgba,
    pub background: Rgba,
}

impl ButtonStyle {
    fn common(palette: &Palette, state: State, primary: Rgba, mut text_color: Rgba) -> ButtonStyle {
        let background = primary;
        let is_dark = background.is_dark();

        let background = match state {
            State::Hovered => palette.accent,
            State::Held => palette.secondary,
            State::Clicked => palette.primary,
            State::Disabled => {
                text_color = palette.outline;
                palette.surface
            }
            State::None => background,
        };

        ButtonStyle {
            text_color,
            background,
        }
    }

    pub fn default(palette: &Palette, state: State) -> Self {
        Self::common(palette, state, palette.outline, palette.foreground)
    }

    pub fn success(palette: &Palette, state: State) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.success, fg)
    }

    pub fn info(palette: &Palette, state: State) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.info, fg)
    }

    pub fn warning(palette: &Palette, state: State) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.warning, fg)
    }

    pub fn danger(palette: &Palette, state: State) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.danger, fg)
    }
}

pub type ButtonClass = fn(&Palette, State) -> ButtonStyle;

pub fn button(label: impl ToCompactString) -> Button {
    Button::new(label)
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Button {
    label: CompactString,
    margin: Margin,
    state: State,
    class: StyleKind<ButtonClass, ButtonStyle>,
}

impl Button {
    pub fn new(label: impl ToCompactString) -> Self {
        Button {
            label: label.to_compact_string(),
            margin: Margin::ZERO,
            state: State::None,
            class: StyleKind::Deferred(ButtonStyle::default),
        }
    }

    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.state = State::Disabled;
        self
    }

    pub const fn class(mut self, class: ButtonClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: ButtonStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v> Builder<'v> for Button {
    type View = Self;
}

impl View for Button {
    type Args<'v> = Self;
    type Response = Response;

    fn create(builder: Self::Args<'_>) -> Self {
        builder
    }

    fn update(&mut self, builder: Self::Args<'_>, _: &Ui) -> Self::Response {
        self.label = builder.label;
        self.class = builder.class;
        self.margin = builder.margin;

        let state = self.state;
        if let State::Clicked = self.state {
            self.state = State::Hovered
        }

        Response { state }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        if matches!(self.state, State::Disabled) {
            return Handled::Bubble;
        }

        self.state = match event {
            ViewEvent::MouseClicked { inside: true, .. } => State::Clicked,
            ViewEvent::MouseHeld { inside: true, .. } => State::Held,
            ViewEvent::MouseEntered => State::Hovered,
            ViewEvent::MouseLeave => State::None,
            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        space.fit(measure_text(&self.label) + self.margin)
    }

    fn draw(&mut self, mut render: Render) {
        let style = match self.class {
            StyleKind::Deferred(class) => (class)(render.palette, self.state),
            StyleKind::Direct(style) => style,
        };

        render.surface.fill(style.background);

        let mut surface = render.surface.shrink(self.margin);
        let mut start = 0;
        // TODO get the default text color
        for grapheme in self.label.graphemes(true) {
            surface.set((start, 0), Grapheme::new(grapheme).fg(style.text_color));
            start += grapheme.width() as i32
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Response {
    state: State,
}

impl Response {
    pub const fn clicked(&self) -> bool {
        matches!(self.state, State::Clicked)
    }

    pub const fn hovered(&self) -> bool {
        matches!(self.state, State::Hovered)
    }

    pub const fn held(&self) -> bool {
        matches!(self.state, State::Held)
    }

    pub const fn disabled(&self) -> bool {
        matches!(self.state, State::Disabled)
    }
}
