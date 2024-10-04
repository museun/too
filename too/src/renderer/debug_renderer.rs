use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::Renderer;
use crate::{math::Pos2, Attribute, Rgba};

/// A renderer that explains the actions a render phase would take
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

use std::{borrow::Cow, fmt::Write as _};

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

    fn write_str(&mut self, data: &str) -> std::io::Result<()> {
        if !self.incomplete {
            self.out.push_str("    ");
        }
        self.incomplete = true;

        for cluster in data.graphemes(true) {
            let cluster = if cluster.chars().all(|c| c.is_whitespace()) {
                Cow::from("▪".repeat(cluster.width()))
            } else {
                Cow::from(cluster)
            };

            _ = write!(&mut self.out, "{cluster}");
        }
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
