use crate::{pixel::Attribute, Backend, Rgba};
use std::io::{BufWriter, Write as _};
use too_math::Pos2;

pub trait Renderer {
    fn begin(&mut self) -> std::io::Result<()>;
    fn end(&mut self) -> std::io::Result<()>;
    fn move_to(&mut self, pos: Pos2) -> std::io::Result<()>;
    fn write(&mut self, ch: char) -> std::io::Result<()>;
    fn set_fg(&mut self, rgb: Rgba) -> std::io::Result<()>;
    fn set_bg(&mut self, rgb: Rgba) -> std::io::Result<()>;
    fn set_attr(&mut self, attr: Attribute) -> std::io::Result<()>;
    fn reset_fg(&mut self) -> std::io::Result<()>;
    fn reset_bg(&mut self) -> std::io::Result<()>;
    fn reset_attr(&mut self) -> std::io::Result<()>;
    fn clear_screen(&mut self) -> std::io::Result<()>;

    fn set_title(&mut self, title: &str) -> std::io::Result<()> {
        _ = title;
        Ok(())
    }

    fn switch_to_alt_screen(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn switch_to_main_screen(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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
        [
            attr.is_reset(),
            attr.is_bold(),
            attr.is_faint(),
            attr.is_italic(),
            attr.is_underline(),
            attr.is_blink(),
            false, // placeholder
            attr.is_reverse(),
            false, // placeholder
            attr.is_strikeout(),
        ]
        .into_iter()
        .enumerate()
        .filter(|(_, c)| *c)
        .try_for_each(|(n, _)| write!(self.out, "\x1b[{n}m"))
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
        self.out.write_all(csi!("?1049h"))
    }

    fn switch_to_main_screen(&mut self) -> std::io::Result<()> {
        self.out.write_all(csi!("?1049l"))
    }
}

#[derive(Default)]
pub struct DebugRenderer {
    out: String,
    incomplete: bool,
}

impl DebugRenderer {
    pub const fn new() -> Self {
        Self {
            out: String::new(),
            incomplete: false,
        }
    }

    fn next_entry(&mut self) {
        if self.incomplete {
            self.out.push('\n');
            self.incomplete = !self.incomplete
        }
    }
}

impl std::fmt::Display for DebugRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out.fmt(f)
    }
}

use std::fmt::Write as _;

impl Renderer for DebugRenderer {
    fn begin(&mut self) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "begin");
        Ok(())
    }

    fn end(&mut self) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "end");
        Ok(())
    }

    fn move_to(&mut self, pos: Pos2) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  move to {pos:?}");
        Ok(())
    }

    fn write(&mut self, ch: char) -> std::io::Result<()> {
        if !self.incomplete {
            self.out.push_str("    ");
        }
        self.incomplete = true;
        let ch = match ch {
            ' ' => '▒',
            d => d,
        };
        _ = write!(&mut self.out, "{ch}");
        Ok(())
    }

    fn set_fg(&mut self, rgb: Rgba) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  set_fg: {rgb:?}");
        Ok(())
    }

    fn set_bg(&mut self, rgb: Rgba) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  set_bg: {rgb:?}");
        Ok(())
    }

    fn set_attr(&mut self, attr: Attribute) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  set_attr: {attr:?}");
        Ok(())
    }

    fn reset_fg(&mut self) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  reset_fg");
        Ok(())
    }

    fn reset_bg(&mut self) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  reset_bg");
        Ok(())
    }

    fn reset_attr(&mut self) -> std::io::Result<()> {
        self.next_entry();
        _ = writeln!(&mut self.out, "  reset_attr");
        Ok(())
    }

    fn clear_screen(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
