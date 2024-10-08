use crate::Rgba;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme {
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

impl Theme {
    pub const fn light() -> Self {
        Self {
            background: Rgba::hex("#E0E0E0"),
            foreground: Rgba::hex("#000000"),
            surface: Rgba::hex("#A3A5A8"),
            outline: Rgba::hex("#9DA2A8"),
            contrast: Rgba::hex("#161616"),
            primary: Rgba::hex("#8175DF"),
            secondary: Rgba::hex("#B8A52D"),
            accent: Rgba::hex("#776BC2"),
            danger: Rgba::hex("#C7343B"),
            success: Rgba::hex("#33D17A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#0077C2"),
        }
    }

    pub const fn dark() -> Self {
        Self {
            background: Rgba::hex("#131313"),
            foreground: Rgba::hex("#FFFFFF"),
            surface: Rgba::hex("#343434"),
            outline: Rgba::hex("#4D4D4D"),
            contrast: Rgba::hex("#F9E9E9"),
            primary: Rgba::hex("#55B1F0"),
            secondary: Rgba::hex("#8C8BED"),
            accent: Rgba::hex("#F4A151"),
            danger: Rgba::hex("#F05D61"),
            success: Rgba::hex("#9AF07A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#6A7DDA"),
        }
    }
}
