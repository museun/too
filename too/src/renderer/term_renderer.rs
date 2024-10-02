use std::io::{BufWriter, Write as _};

use super::Renderer;
use crate::{math::Pos2, Attribute, Backend, Rgba};

/// Renders to a `Backend` using ANSI escape sequences
pub struct TermRenderer<'a, B: Backend + 'a> {
    out: BufWriter<B::Out<'a>>,
}

impl<'a, B: Backend> TermRenderer<'a, B> {
    pub fn new(term: &'a mut B) -> Self {
        let size = term.size();
        let estimate = size.x as usize * size.y as usize * 21;

        // TODO maybe cache this
        Self {
            out: BufWriter::with_capacity(estimate, term.writer()),
        }
    }
}

macro_rules! csi {
    ($($lit:literal),*) => {
        concat!($("\x1b[", $lit),*).as_bytes()
    };
}

impl<'a, B: Backend> Renderer for TermRenderer<'a, B> {
    #[inline(always)]
    fn begin(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?2026h"))
    }

    #[inline(always)]
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

    #[inline(always)]
    fn write(&mut self, ch: char) -> std::io::Result<()> {
        self.out.write_all(ch.encode_utf8(&mut [0; 4]).as_bytes())
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
            self.out.write_all(csi!("[0m"))?;
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

    fn clear_screen(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("2J"))
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
