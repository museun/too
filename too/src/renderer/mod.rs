mod buffer;
pub use buffer::Buffer;

mod surface;
pub use surface::{Surface, SurfaceMut};

mod shape;
pub use shape::{anonymous, anonymous_ctx, Shape};

mod pixel;
pub use pixel::{Attribute, Color, Pixel};

mod rgba;
pub use rgba::Rgba;

mod gradient;
pub use gradient::Gradient;

pub mod shapes;

pub trait Renderer {
    fn begin(&mut self) -> std::io::Result<()>;
    fn end(&mut self) -> std::io::Result<()>;
    fn move_to(&mut self, pos: crate::Pos2) -> std::io::Result<()>;
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

mod term_renderer;
pub use term_renderer::TermRenderer;

mod debug_renderer;
pub use debug_renderer::DebugRenderer;

// TODO TestRenderer
