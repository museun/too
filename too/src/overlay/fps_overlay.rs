use crate::{
    layout::{Anchor2, Axis, LinearLayout},
    math::vec2,
    shapes::Text,
    EmaWindow, Overlay, Rgba, SurfaceMut, WindowStats,
};

/// An FPS overlay
pub struct FpsOverlay {
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

    pub(crate) window: EmaWindow<32>,
}

impl FpsOverlay {
    /// Get the current [`WindowStats`] of the EMA window
    pub fn get_current_stats(&self) -> WindowStats {
        self.window.get()
    }

    /// Push a new frame timing
    pub fn push(&mut self, frame_time: f32) {
        self.window.push(frame_time);
    }

    /// Toggle showing this overlay
    pub fn toggle(&mut self) {
        self.show = !self.show
    }

    /// Draws the default fps overlay
    ///
    /// This does not check whether its shown -- thats up to you
    pub fn default_draw(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
        let frame_stats = overlay.fps.get_current_stats();

        let mut alloc = LinearLayout::new(overlay.fps.axis)
            .anchor(overlay.fps.anchor)
            .wrap(true)
            .spacing(vec2(1, 0))
            .layout(surface.rect());

        let (fg, bg) = (overlay.fps.fg, overlay.fps.bg);
        for part in [
            format!("min: {:.2}", frame_stats.min),
            format!("max: {:.2}", frame_stats.max),
            format!("avg: {:.2}", frame_stats.avg),
        ] {
            let part = Text::new(part).fg(fg).bg(bg);
            if let Some(rect) = alloc.allocate(part.size()) {
                surface.crop(rect).draw(part);
            }
        }
    }
}

impl Default for FpsOverlay {
    fn default() -> Self {
        Self {
            show: false,
            window: EmaWindow::new(),
            axis: Axis::Horizontal,
            anchor: Anchor2::LEFT_TOP,
            fg: Rgba::hex("#F00"),
            bg: Rgba::hex("#000"),
        }
    }
}
