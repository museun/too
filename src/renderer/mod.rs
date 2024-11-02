mod surface;

pub use surface::Surface;

mod cell;
pub use cell::{Attribute, Cell, Color, Grapheme, Pixel, Underline};

pub mod rgba;
pub use rgba::Rgba;

mod gradient;
pub use gradient::Gradient;

use crate::math::{self};

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

mod term_renderer;
pub use term_renderer::TermRenderer;

mod debug_renderer;
pub use debug_renderer::DebugRenderer;

mod dummy_renderer;
pub use dummy_renderer::DummyRenderer;

// TODO TestRenderer
