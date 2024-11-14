use crate::Rgba;

#[derive(Copy, Clone, Debug)]
pub enum StyleKind<Class, Style> {
    Deferred(Class),
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Palette {
    pub background: Rgba,
    pub foreground: Rgba,
    pub surface: Rgba,
    pub outline: Rgba,
    pub contrast: Rgba,
    pub primary: Rgba,
    pub secondary: Rgba,
    pub accent: Rgba,
    pub danger: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub info: Rgba,
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark()
    }
}

impl Palette {
    pub fn is_dark(&self) -> bool {
        self.background.is_dark()
    }

    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }

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

    pub const fn light() -> Self {
        Palette {
            background: Rgba::hex("#E0E0E0"),
            foreground: Rgba::hex("#000000"),
            surface: Rgba::hex("#A3A5A8"),
            outline: Rgba::hex("#9D7278"),
            contrast: Rgba::hex("#569636"),
            primary: Rgba::hex("#8175DF"),
            secondary: Rgba::hex("#B8A52D"),
            accent: Rgba::hex("#776BC2"),
            danger: Rgba::hex("#C7343B"),
            success: Rgba::hex("#33D17A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#0077C2"),
        }
    }
}

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
