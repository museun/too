use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    view::{
        geom::{Margin, Size, Space},
        views::measure_text,
        Builder, EventCtx, Handled, Interest, Layout, Render, Ui, View, ViewEvent,
    },
    Grapheme,
};

pub fn button(label: impl ToCompactString) -> Button {
    Button {
        label: label.to_compact_string(),
        margin: Margin::ZERO,
        disabled: false,
        state: ButtonState::None,
    }
}

#[derive(Clone, PartialEq, Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Button {
    label: CompactString,
    margin: Margin,
    disabled: bool,
    state: ButtonState,
}

impl Button {
    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'v> Builder<'v> for Button {
    type View = Self;
}

impl View for Button {
    type Args<'v> = Self;
    type Response = ButtonResponse;

    fn create(builder: Self::Args<'_>) -> Self {
        builder
    }

    fn update(&mut self, builder: Self::Args<'_>, _: &Ui) -> Self::Response {
        self.label = builder.label;
        self.disabled = builder.disabled;
        self.margin = builder.margin;

        let state = self.state;
        if let ButtonState::Clicked = self.state {
            self.state = ButtonState::Hovered
        }

        ButtonResponse { state }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        if self.disabled {
            return Handled::Bubble;
        }

        self.state = match event {
            ViewEvent::MouseClicked { inside: true, .. } => ButtonState::Clicked,
            ViewEvent::MouseHeld { inside: true, .. } => ButtonState::Held,
            ViewEvent::MouseEntered => ButtonState::Hovered,
            ViewEvent::MouseLeave => ButtonState::None,
            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        space.fit(measure_text(&self.label) + self.margin)
    }

    fn draw(&mut self, mut render: Render) {
        let fg = if !self.disabled {
            render.theme.foreground
        } else {
            render.theme.outline
        };

        let bg = match self.state {
            ButtonState::Hovered if !self.disabled => render.theme.accent,
            ButtonState::Held if !self.disabled => render.theme.primary,
            ButtonState::Clicked if !self.disabled => render.theme.success,
            ButtonState::None if !self.disabled => render.theme.surface,
            _ => render.theme.surface,
        };

        // TODO get bg from the state
        render.surface.fill(bg);

        let mut surface = render.surface.shrink(self.margin);
        let mut start = 0;
        // TODO get the default text color
        for grapheme in self.label.graphemes(true) {
            surface.set((start, 0), Grapheme::new(grapheme).fg(fg));
            start += grapheme.width() as i32
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
enum ButtonState {
    Hovered,
    Held,
    Clicked,
    #[default]
    None,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ButtonResponse {
    state: ButtonState,
}

impl ButtonResponse {
    pub const fn clicked(&self) -> bool {
        matches!(self.state, ButtonState::Clicked)
    }

    pub const fn hovered(&self) -> bool {
        matches!(self.state, ButtonState::Hovered)
    }

    pub const fn held(&self) -> bool {
        matches!(self.state, ButtonState::Held)
    }
}
