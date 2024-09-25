pub struct Dummy;

impl too_renderer::Backend for Dummy {
    type Out<'a> = std::io::Empty;

    fn size(&self) -> too_math::Vec2 {
        too_math::vec2(80, 25)
    }

    fn should_draw(&self) -> bool {
        true
    }

    fn command(&mut self, _cmd: too_renderer::Command) {}

    fn writer(&mut self) -> Self::Out<'_> {
        std::io::empty()
    }
}

impl too_events::EventReader for Dummy {
    fn try_read_event(&mut self) -> Option<too_events::Event> {
        Some(too_events::Event::Quit)
    }
}
