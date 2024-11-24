use crate::renderer::Rgba;

/// Views have the ability to style themselves.
///
/// This type allow a view to have a pre-computed style,
///
/// Or allows it to compute the style when it needs it
///
/// ## Convention
///
/// Conventional types:
/// ```ignore
/// struct Style;
/// type Class = fn(&Palette) -> Style;
/// fn class(mut self, class: Class) -> Self;
/// fn style(mut self, style: Style) -> Self;
/// ```
///
/// 3## Description
/// When a view wants to style itself, it has to define a few 'by-convention' types.
///
/// Generally this means making a struct `YourViewStyle` and then a type aliasing for a way to construct this style.
///
/// The deferred style is generally called `YourViewClass` and its just a function pointer that takes a palette, some arguments and produces your style.
///
/// And on your builder, you implement 2 builder methods:
///
/// `class(YourClass)`
///
/// and
///
/// `style(YouStyle)`
///
/// ### Example
/// A simple example:
/// ```rust
/// use too::{
///     layout::Axis,
///     renderer::{Rgba, Pixel},
///     view::{Palette, StyleKind, Builder, View, Render}
/// };
///
/// #[derive(Copy, Clone, Debug)]
/// struct MyStyle {
///     background: Rgba,
///     fill: char,
/// }
///
/// impl MyStyle {
///     // by convention, styles provide a default 'class'
///     fn default(palette: &Palette, axis: Axis) -> Self {
///         Self {
///             background: palette.background,
///             // when in horizontal, we should use a ?, otherwise a !
///             fill: axis.main(('?', '!')),
///         }
///     }
///
///     // we can delegate to the default style
///     fn hash_at(palette: &Palette, axis: Axis) -> Self {
///         Self {
///             fill: axis.main(('#', '@')),
///             ..Self::default(palette, axis)
///         }
///     }
/// }
///
/// /// A `class` is just a function pointer that takes in a &Palette, some
/// /// of your args and returns your Style
/// type MyClass = fn(&Palette, Axis) -> MyStyle;
///
/// fn builder() -> MyBuilder {
///     MyBuilder {
///         class: StyleKind::Deferred(MyStyle::default),
///     }
/// }
///
/// struct MyBuilder {
///     /// We make the StyleKind mapping
///     class: StyleKind<MyClass, MyStyle>,
/// }
///
/// impl MyBuilder {
///     const fn class(mut self, class: MyClass) -> Self {
///         self.class = StyleKind::Deferred(class);
///         self
///     }
///
///     const fn style(mut self, style: MyStyle) -> Self {
///         self.class = StyleKind::Direct(style);
///         self
///     }
/// }
///
/// impl<'v> Builder<'v> for MyBuilder {
///     type View = MyView;
/// }
///
/// #[derive(Debug)]
/// struct MyView {
///     class: StyleKind<MyClass, MyStyle>,
/// }
///
/// impl View for MyView {
///     type Args<'v> = MyBuilder;
///     type Response = ();
///
///     fn create(args: Self::Args<'_>) -> Self {
///         Self { class: args.class }
///     }
///
///     fn draw(&mut self, mut render: Render) {
///         // Then when we want to 'resolve' the style, we simply match on
///         // it, and pass in the palette + some arguments
///         let style = match self.class {
///             StyleKind::Deferred(style) => (style)(render.palette, Axis::Horizontal),
///             StyleKind::Direct(style) => style,
///         };
///
///         render.fill_with(Pixel::new(style.fill).bg(style.background));
///     }
/// }
///```
/// Then a user can do:
/// ```ignore
/// // for a deferred style:
/// ui.show(builder().class(MyStyle::hash_at));
/// // or for a pre-computed style
/// ui.show(builder().style(MyStyle {
///     background: Rgba::hex("#123"),
///     ..MyStyle::hash_at(&ui.palette(), Axis::Horizontal)
/// }));
/// ```
#[derive(Copy, Clone, Debug)]
pub enum StyleKind<Class, Style> {
    /// Compute the style on use
    Deferred(Class),
    /// A pre-computed styled
    Direct(Style),
}

impl<F, T> StyleKind<F, T> {
    pub const fn deferred(class: F) -> Self {
        Self::Deferred(class)
    }

    pub const fn direct(style: T) -> Self {
        Self::Direct(style)
    }
}

/// A color palette used by the common [`crate::views`]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Palette {
    /// The background color
    pub background: Rgba,
    /// The foreground color
    pub foreground: Rgba,
    /// A color close to the background, but more visible
    pub surface: Rgba,
    /// A color used to outline things. This is generally like surface, but even more visible
    pub outline: Rgba,
    /// A color used to contrast something against the background
    pub contrast: Rgba,
    /// A color used for a primary action -- e.g. the default interaction color
    pub primary: Rgba,
    /// A color used for a secondary action -- e.g an interaction color that is different from the primary color
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
        Palette {
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
        Palette {
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
