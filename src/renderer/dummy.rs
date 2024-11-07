use super::Renderer;

pub struct DummyRenderer;

impl Renderer for DummyRenderer {
    fn begin(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn end(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn move_to(&mut self, pos: crate::math::Pos2) -> std::io::Result<()> {
        Ok(())
    }

    fn write_str(&mut self, data: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn set_fg(&mut self, rgb: super::Rgba) -> std::io::Result<()> {
        Ok(())
    }

    fn set_bg(&mut self, rgb: super::Rgba) -> std::io::Result<()> {
        Ok(())
    }

    fn set_attr(&mut self, attr: super::Attribute) -> std::io::Result<()> {
        Ok(())
    }

    fn reset_fg(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn reset_bg(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn reset_attr(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
