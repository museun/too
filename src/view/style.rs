use std::ops::Deref;

use crate::{helpers::short_name, renderer::Rgba};

use super::{builder::ViewMarker, Render, Ui};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct StyleOptions<T = ()> {
    pub hovered: bool,
    pub focused: bool,
    pub selected: bool,
    pub interactive: bool,
    pub state: StyleState,
    pub args: T,
}

impl<T> Deref for StyleOptions<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl StyleOptions<()> {
    pub const fn with_args<T>(self, args: T) -> StyleOptions<T> {
        StyleOptions {
            args,
            state: self.state,
            hovered: self.hovered,
            focused: self.focused,
            selected: self.selected,
            interactive: self.interactive,
        }
    }
}

impl<T> StyleOptions<T> {
    pub fn resolve_color(&self, palette: &Palette, or_default: fn(&Palette) -> Rgba) -> Rgba {
        match self.state {
            StyleState::Hoverable if self.hovered => palette.accent,
            StyleState::Disabled => palette.outline,
            _ => or_default(palette),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum StyleState {
    Hoverable,
    Disabled,
    #[default]
    None,
}

impl StyleState {
    pub const fn is_hoverable(&self) -> bool {
        matches!(self, Self::Hoverable)
    }

    pub const fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

pub trait Style: Sized + Clone + 'static {
    type Args: 'static + ViewMarker;
    fn default(palette: &Palette, options: StyleOptions<Self::Args>) -> Self;
    fn indirect() -> impl FnOnce(&Palette, StyleOptions<Self::Args>) -> Self {
        move |palette, options| Self::default(palette, options)
    }
}

impl Style for () {
    type Args = ();
    fn default(palette: &Palette, options: StyleOptions<Self::Args>) -> Self {
        _ = palette;
        _ = options;
    }
}

enum ApplicableStyleKind<S>
where
    S: Style + ViewMarker,
{
    Direct(S),
    #[allow(clippy::type_complexity)]
    // we cannot type alias this because it was an associated type from the generic 'S'
    Indirect(Box<dyn Fn(&Palette, StyleOptions<S::Args>) -> S>),
}

pub struct ApplicableStyle<S = ()>
where
    S: Style + ViewMarker,
{
    kind: ApplicableStyleKind<S>,
    pub state: StyleState,
}

impl<S> Default for ApplicableStyle<S>
where
    S: Style + ViewMarker,
{
    fn default() -> Self {
        Self::new(S::default)
    }
}

impl<S> ApplicableStyle<S>
where
    S: Style + ViewMarker,
{
    pub fn new<T>(make: T) -> Self
    where
        T: Fn(&Palette, StyleOptions<S::Args>) -> S + 'static + ViewMarker,
    {
        Self {
            kind: ApplicableStyleKind::Indirect(Box::new(move |p, o| make(p, o))),
            state: StyleState::None,
        }
    }

    pub const fn value(value: S) -> Self {
        Self {
            kind: ApplicableStyleKind::Direct(value),
            state: StyleState::None,
        }
    }

    pub fn deferred() -> Self {
        Self {
            kind: ApplicableStyleKind::Indirect(Box::new(move |p, o| S::indirect()(p, o))),
            state: StyleState::None,
        }
    }

    pub const fn merge(mut self, other: &Self) -> Self {
        self.state = other.state;
        self
    }

    pub const fn with_style_state(mut self, state: StyleState) -> Self {
        self.state = state;
        self
    }

    pub fn apply(
        &self,
        applicator: impl StyleApplicator,
        map: impl FnOnce(StyleOptions<()>) -> StyleOptions<S::Args>,
    ) -> S {
        match &self.kind {
            ApplicableStyleKind::Direct(value) => value.clone(),
            ApplicableStyleKind::Indirect(indirect) => applicator
                .apply(self, move |palette, options| {
                    indirect(palette, map(options))
                }),
        }
    }
}

pub trait StyleApplicator {
    fn apply<S>(
        &self,
        style: &ApplicableStyle<S>,
        f: impl FnOnce(&Palette, StyleOptions<()>) -> S,
    ) -> S
    where
        S: Style;
}

impl StyleApplicator for &Render<'_, '_> {
    fn apply<S: Style>(
        &self,
        style: &ApplicableStyle<S>,
        f: impl FnOnce(&Palette, StyleOptions<()>) -> S,
    ) -> S {
        f(self.palette, self.style_options(style))
    }
}

impl StyleApplicator for &Ui<'_> {
    fn apply<S>(
        &self,
        style: &ApplicableStyle<S>,
        f: impl FnOnce(&Palette, StyleOptions<()>) -> S,
    ) -> S
    where
        S: Style,
    {
        f(&self.palette(), self.current_style_options(style))
    }
}

impl<S> std::fmt::Debug for ApplicableStyle<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NoQuote<'a>(&'a str);
        impl std::fmt::Debug for NoQuote<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.0)
            }
        }
        use std::any::type_name;
        f.debug_struct("ApplicableStyle")
            .field("state", &self.state)
            .field("args", &NoQuote(&short_name(type_name::<S::Args>())))
            .field("style", &NoQuote(&short_name(type_name::<S>())))
            .finish()
    }
}

/// A color palette used by the common [`crate::views`]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    /// The background color
    pub background: Rgba,
    /// The foreground color
    pub foreground: Rgba,
    /// A color close to the background, but more visible
    pub surface: Rgba,
    /// A color used to outline things.
    ///
    /// This is generally like surface, but even more visible
    pub outline: Rgba,
    /// A color used to contrast something against the background
    pub contrast: Rgba,
    /// A color used for a primary action
    ///
    /// e.g. the default interaction color
    pub primary: Rgba,
    /// A color used for a secondary action
    ///
    /// e.g an interaction color that is different from the primary color
    pub secondary: Rgba,
    /// A accent color used to differentiate something from a primary and secondary color
    pub accent: Rgba,
    /// A color representing that something is dangerous
    pub danger: Rgba,
    /// A color representing that something is successful
    pub success: Rgba,
    /// A color representing that something is potentially dangerous
    pub warning: Rgba,
    /// A coloe representing that something should be noted
    pub info: Rgba,
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark()
    }
}

impl Palette {
    /// Is this background's luminosity considered 'dark'?
    pub fn is_dark(&self) -> bool {
        self.background.is_dark()
    }

    /// Is this background's luminosity considered 'light'?
    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }

    /// A default "dark" palette
    ///
    /// # A visualization of this palette
    /// | Color | Visualization |
    /// | --- | --- |
    /// | foreground | <span style="color: #FFFFFF; background-color: #131313;">#FFFFFF</span> |
    /// | surface | <span style="color: #232323; background-color: #131313;">#232323</span> |
    /// | outline | <span style="color: #4D4D4D; background-color: #131313;">#4D4D4D</span> |
    /// | contrast | <span style="color: #A9E9E9; background-color: #131313;">#A9E9E9</span> |
    /// | primary | <span style="color: #55B1F0; background-color: #131313;">#55B1F0</span> |
    /// | secondary | <span style="color: #8C8BED; background-color: #131313;">#8C8BED</span> |
    /// | accent | <span style="color: #F4A151; background-color: #131313;">#F4A151</span> |
    /// | danger | <span style="color: #F05D61; background-color: #131313;">#F05D61</span> |
    /// | success | <span style="color: #9AF07A; background-color: #131313;">#9AF07A</span> |
    /// | warning | <span style="color: #F9F35F; background-color: #131313;">#F9F35F</span> |
    /// | info | <span style="color: #6A7DDA; background-color: #131313;">#6A7DDA</span> |
    ///
    /// (All text is on the `Palette::background` color)
    pub const fn dark() -> Self {
        Self {
            background: Rgba::hex("#131313"),
            foreground: Rgba::hex("#FFFFFF"),
            surface: Rgba::hex("#232323"),
            outline: Rgba::hex("#4D4D4D"),
            contrast: Rgba::hex("#A9E9E9"),
            primary: Rgba::hex("#55B1F0"),
            secondary: Rgba::hex("#8C8BED"),
            accent: Rgba::hex("#F4A151"),
            danger: Rgba::hex("#F05D61"),
            success: Rgba::hex("#9AF07A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#6A7DDA"),
        }
    }

    /// A default "light" palette
    ///
    /// # A visualization of this palette
    /// | Color | Visualization |
    /// | --- | --- |
    /// | foreground | <span style="color: #000000; background-color: #E0E0E0;">#000000</span> |
    /// | surface | <span style="color: #C3C5C8; background-color: #E0E0E0;">#C3C5C8</span> |
    /// | outline | <span style="color: #9D9099; background-color: #E0E0E0;">#9D9099</span> |
    /// | contrast | <span style="color: #663696; background-color: #E0E0E0;">#663696</span> |
    /// | primary | <span style="color: #8175DF; background-color: #E0E0E0;">#8175DF</span> |
    /// | secondary | <span style="color: #28758D; background-color: #E0E0E0;">#28758D</span> |
    /// | accent | <span style="color: #776BC2; background-color: #E0E0E0;">#776BC2</span> |
    /// | danger | <span style="color: #C7343B; background-color: #E0E0E0;">#C7343B</span> |
    /// | success | <span style="color: #33D17A; background-color: #E0E0E0;">#33D17A</span> |
    /// | warning | <span style="color: #F9F35F; background-color: #E0E0E0;">#F9F35F</span> |
    /// | info | <span style="color: #0077C2; background-color: #E0E0E0;">#0077C2</span> |
    ///
    /// (All text is on the `Palette::background` color)
    pub const fn light() -> Self {
        Self {
            background: Rgba::hex("#E0E0E0"),
            foreground: Rgba::hex("#000000"),
            surface: Rgba::hex("#C3C5C8"),
            outline: Rgba::hex("#9D9099"),
            contrast: Rgba::hex("#663696"),
            primary: Rgba::hex("#8175DF"),
            secondary: Rgba::hex("#28758D"),
            accent: Rgba::hex("#776BC2"),
            danger: Rgba::hex("#C7343B"),
            success: Rgba::hex("#33D17A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#0077C2"),
        }
    }
}

/// Useful elements for drawing a TUI
pub struct Elements;
impl Elements {
    pub const LARGE_RECT: char = '█';
    pub const MEDIUM_RECT: char = '■';
    pub const SMALL_RECT: char = '▮';

    pub const CIRCLE: char = '●';
    pub const DIAMOND: char = '◆';

    pub const HORIZONTAL_LINE: char = '─';
    pub const THICK_HORIZONTAL_LINE: char = '━';
    pub const DASH_HORIZONTAL_LINE: char = '╌';
    pub const THICK_DASH_HORIZONTAL_LINE: char = '╍';
    pub const DOUBLE_HORIZONATAL_LINE: char = '═';

    pub const VERTICAL_LINE: char = '│';
    pub const THICK_VERTICAL_LINE: char = '┃';
    pub const DASH_VERTICAL_LINE: char = '╎';
    pub const THICK_DASH_VERTICAL_LINE: char = '╏';
    pub const DOUBLE_VERTICAL_LINE: char = '║';
}
