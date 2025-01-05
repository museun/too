#[allow(deprecated)]
use crate::view::measure_text;

use crate::{
    layout::Align,
    math::{Margin, Size, Space},
    renderer::{Rgba, TextShape},
    view::{
        ApplicableStyle, Builder, EventCtx, Handled, Interest, Layout, Palette, Render, Style, Ui,
        View, ViewEvent,
    },
    Str,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum ButtonState {
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

impl Style for ButtonStyle {
    type Args = ButtonState;
    fn default(palette: &Palette, args: Self::Args) -> Self {
        Self::common(palette, args, palette.outline, palette.foreground)
    }
}

impl ButtonStyle {
    pub fn success(palette: &Palette, state: ButtonState) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.success, fg)
    }

    pub fn info(palette: &Palette, state: ButtonState) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.info, fg)
    }

    pub fn warning(palette: &Palette, state: ButtonState) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.warning, fg)
    }

    pub fn danger(palette: &Palette, state: ButtonState) -> Self {
        let fg = if palette.is_dark() {
            palette.background
        } else {
            palette.foreground
        };
        Self::common(palette, state, palette.danger, fg)
    }

    fn common(palette: &Palette, state: ButtonState, primary: Rgba, mut text_color: Rgba) -> Self {
        let background = match state {
            ButtonState::Hovered => palette.accent,
            ButtonState::Held => palette.secondary,
            ButtonState::Clicked => palette.primary,
            ButtonState::Disabled => {
                // why?
                text_color = palette.outline;
                palette.surface
            }
            ButtonState::None => primary,
        };

        Self {
            text_color,
            background,
        }
    }
}

pub fn button(label: impl Into<Str>) -> Button {
    Button::new(label)
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Button {
    label: Str,
    margin: Margin,
    state: ButtonState,
    disabled: bool,
    main: Align,
    cross: Align,
    style: ApplicableStyle<ButtonStyle>,
}

impl Button {
    pub fn new(label: impl Into<Str>) -> Self {
        Self {
            label: label.into(),
            margin: Margin::symmetric(1, 0),
            state: ButtonState::None,
            disabled: false,
            main: Align::Min,
            cross: Align::Min,
            style: ApplicableStyle::default(),
        }
    }

    pub const fn text_horizontal_align(mut self, justify: Align) -> Self {
        self.main = justify;
        self
    }

    pub const fn text_vertical_align(mut self, justify: Align) -> Self {
        self.cross = justify;
        self
    }

    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    pub const fn disabled_if(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.state = if disabled {
            ButtonState::Disabled
        } else {
            ButtonState::None
        };
        self
    }
}

impl Builder<'_> for Button {
    type View = Self;
    type Style = ButtonStyle;

    fn style(mut self, style: Self::Style) -> Self {
        self.style = ApplicableStyle::value(style);
        self
    }

    fn class(mut self, style: impl Fn(&Palette, ButtonState) -> Self::Style + 'static) -> Self {
        self.style = ApplicableStyle::new(style);
        self
    }
}

impl View for Button {
    type Args<'v> = Self;
    type Response = ButtonResponse;

    fn create(builder: Self::Args<'_>) -> Self {
        builder
    }

    fn interactive(&self) -> bool {
        true
    }

    fn update(&mut self, builder: Self::Args<'_>, _: &Ui) -> Self::Response {
        // TODO splat this

        self.label = builder.label;
        self.style = builder.style;
        self.margin = builder.margin;
        self.disabled = builder.disabled;
        self.main = builder.main;
        self.cross = builder.cross;

        let state = self.state;
        if self.state == ButtonState::Clicked {
            self.state = ButtonState::Hovered
        }

        if self.disabled {
            self.state = ButtonState::Disabled
        } else if !self.disabled && matches!(self.state, ButtonState::Disabled) {
            self.state = ButtonState::None
        }

        ButtonResponse { state }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    fn event(&mut self, event: ViewEvent, _ctx: EventCtx) -> Handled {
        if matches!(self.state, ButtonState::Disabled) {
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

    fn layout(&mut self, _layout: Layout, space: Space) -> Size {
        // FIXME this should use this function
        #[allow(deprecated)]
        space.fit(measure_text(&self.label) + self.margin)
    }

    fn draw(&mut self, mut render: Render) {
        let style = self.style.apply(render.palette, self.state);

        render
            .fill_bg(style.background)
            .shrink(self.margin, |render| {
                render.text(TextShape::new(&self.label).fg(style.text_color));
            });
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

    pub const fn disabled(&self) -> bool {
        matches!(self.state, ButtonState::Disabled)
    }
}
