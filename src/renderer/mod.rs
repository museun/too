//! Things that can be drawn to a surface
mod surface;
pub use surface::Surface;

mod cell;
pub use cell::{Attribute, Cell, Color, Grapheme, Pixel};

mod rgba;
pub use rgba::Rgba;

mod gradient;
pub use gradient::Gradient;

mod border;
pub use border::Border;

mod rasterizer;
pub use rasterizer::{Rasterizer, Shape, TextShape};

use crate::math;

/// Abstraction for rendering to a surface.
pub trait Renderer {
    fn begin(&mut self) -> std::io::Result<()>;
    fn end(&mut self) -> std::io::Result<()>;
    fn move_to(&mut self, pos: math::Pos2) -> std::io::Result<()>;
    fn write_str(&mut self, data: &str) -> std::io::Result<()>;
    fn set_fg(&mut self, rgb: Rgba) -> std::io::Result<()>;
    fn set_bg(&mut self, rgb: Rgba) -> std::io::Result<()>;
    fn set_attr(&mut self, attr: Attribute) -> std::io::Result<()>;
    fn reset_fg(&mut self) -> std::io::Result<()>;
    fn reset_bg(&mut self) -> std::io::Result<()>;
    fn reset_attr(&mut self) -> std::io::Result<()>;

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

mod term;
pub use term::TermRenderer;

mod debug;
pub use debug::DebugRenderer;

mod dummy;
pub use dummy::DummyRenderer;

// TODO TestRenderer
