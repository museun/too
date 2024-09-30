use std::collections::VecDeque;

use crate::{
    layout::{Anchor2, Axis, LinearLayout},
    math::vec2,
    shapes::Text,
    Overlay, Rgba, SurfaceMut,
};

/// Allows you to show messages ontop of everything
pub struct DebugOverlay {
    /// Should we show this overlay?
    pub show: bool,
    /// What axis should it be rendered on
    pub axis: Axis,
    /// Where should the elements be anchored on the axis
    pub anchor: Anchor2,

    /// Foreground color over the text
    pub fg: Rgba,
    /// Background color over the text
    pub bg: Rgba,

    /// How many messages to buffer
    pub limit: usize,

    queue: VecDeque<String>,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show: false,
            axis: Axis::Vertical,
            anchor: Anchor2::RIGHT_TOP,
            fg: Rgba::hex("#F00"),
            bg: Rgba::hex("#000"),
            limit: 100,
            queue: VecDeque::new(),
        }
    }
}

impl DebugOverlay {
    /// Push a new message into the overlay
    pub fn push(&mut self, msg: impl ToString) {
        while self.queue.len() > self.limit {
            self.queue.pop_back();
        }
        self.queue.push_front(msg.to_string());
    }

    /// Toggle showing this overlay
    pub fn toggle(&mut self) {
        self.show = !self.show
    }

    /// Drain all of the queued debug messages
    pub fn drain(&mut self) -> impl ExactSizeIterator<Item = String> + DoubleEndedIterator + '_ {
        self.queue.drain(..)
    }

    /// Draws the default debug overlay
    ///
    /// This does not check whether its shown -- thats up to you
    pub fn default_draw(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
        let mut alloc = LinearLayout::new(overlay.debug.axis)
            .anchor(overlay.debug.anchor)
            .wrap(true)
            .spacing(vec2(1, 0))
            .layout(surface.rect());

        let (fg, bg) = (overlay.debug.fg, overlay.debug.bg);

        for msg in overlay.debug.drain().rev() {
            let part = Text::new(msg).fg(fg).bg(bg);
            if let Some(rect) = alloc.allocate(part.size()) {
                surface.crop(rect).draw(part);
            }
        }
    }
}
