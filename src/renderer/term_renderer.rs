use std::io::Write;

use super::Renderer;
use crate::{math::Pos2, Attribute, Rgba};

/// Renders to a `Backend` using ANSI escape sequences
pub struct TermRenderer<W: Write> {
    out: W,
}

impl<W: Write> TermRenderer<W> {
    pub const fn new(out: W) -> Self {
        Self { out }
    }
}

macro_rules! csi {
    ($($lit:literal),*) => {
        concat!($("\x1b[", $lit),*).as_bytes()
    };
}

impl<W: Write> Renderer for TermRenderer<W> {
    #[inline(always)]
    fn begin(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?2026h"))
    }

    #[inline(always)]
    #[profiling::function]
    fn end(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?2026l"))?;
        self.out.flush()
    }

    #[inline(always)]
    fn move_to(&mut self, pos: Pos2) -> std::io::Result<()> {
        const FIXUP: Pos2 = Pos2::splat(1);
        // terminals are 1-based, but we use 0-based indexing
        let Pos2 { x, y } = pos + FIXUP;
        write!(self.out, "\x1b[{y};{x};H")
    }

    // #[inline(always)]
    // fn write_underline_color(
    //     &mut self,
    //     underline: super::Underline,
    //     color: Rgba,
    // ) -> std::io::Result<()> {
    //     fn iter(data: u8) -> impl Iterator<Item = u8> {
    //         let mut pos = 0;
    //         std::iter::from_fn(move || loop {
    //             if pos >= u8::BITS {
    //                 return None;
    //             }

    //             let set = (data & (1 << pos)) != 0;
    //             pos += 1;
    //             if set {
    //                 return Some(pos as _);
    //             }
    //         })
    //     }

    //     for mode in iter(underline.0) {
    //         write!(self.out, "\x1b[4:{mode}m")?;
    //     }

    //     let Rgba(r, g, b, ..) = color;
    //     write!(self.out, "\x1b[58:2::{r}:{g}:{b}m")
    // }

    #[inline(always)]
    fn write_str(&mut self, data: &str) -> std::io::Result<()> {
        write!(self.out, "{data}")
    }

    #[inline(always)]
    fn set_fg(&mut self, rgb: Rgba) -> std::io::Result<()> {
        let Rgba(r, g, b, ..) = rgb;
        write!(self.out, "\x1b[38;2;{r};{g};{b}m")
    }

    #[inline(always)]
    fn set_bg(&mut self, rgb: Rgba) -> std::io::Result<()> {
        let Rgba(r, g, b, ..) = rgb;
        write!(self.out, "\x1b[48;2;{r};{g};{b}m")
    }

    #[inline(always)]
    fn set_attr(&mut self, attr: Attribute) -> std::io::Result<()> {
        fn iter(data: u16) -> impl Iterator<Item = u8> {
            let mut pos = 0;
            std::iter::from_fn(move || loop {
                if pos >= u16::BITS {
                    return None;
                }

                let set = (data & (1 << pos)) != 0;
                pos += 1;
                if set {
                    return Some(pos as _);
                }
            })
        }

        let mut seen = false;
        for i in iter(attr.0) {
            seen = true;
            write!(self.out, "\x1b[{i}m")?;
        }

        if !seen {
            self.out.write_all(csi!("0m"))?;
        }

        Ok(())
    }

    #[inline(always)]
    fn reset_fg(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("39m"))
    }

    #[inline(always)]
    fn reset_bg(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("49m"))
    }

    #[inline(always)]
    fn reset_attr(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("0m"))
    }

    fn set_title(&mut self, title: &str) -> std::io::Result<()> {
        write!(self.out, "\x1b]2;{title}\x07")
    }

    fn switch_to_alt_screen(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?1049h"))?;
        self.out.flush()
    }

    fn switch_to_main_screen(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?1049l"))?;
        self.out.flush()
    }
}
