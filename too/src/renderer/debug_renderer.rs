use super::Renderer;
use crate::{Attribute, Pos2, Rgba};

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
