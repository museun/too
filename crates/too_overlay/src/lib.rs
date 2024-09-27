//! Overlays are (optionally) drawn ontop of the surface
use too_renderer::SurfaceMut;

mod ema_window;
pub use ema_window::{EmaWindow, WindowStats};

#[derive(Default)] // don't mem swap me
#[non_exhaustive]
/// Overlays are drawn ontop of all over renders
///
/// These are things like an ***FPS*** display
pub struct Overlay {
    /// An FPS overlay
    pub fps: FpsOverlay,
    /// A Debug message overlay
    pub debug: DebugOverlay,
}

impl Overlay {
    /// A default overlay draw function.
    pub fn default_draw<T>(_: T, overlay: &mut Overlay, mut surface: SurfaceMut<'_>) {
        if overlay.fps.show {
            FpsOverlay::default_draw(overlay, &mut surface);
        }

        if overlay.debug.show {
            DebugOverlay::default_draw(overlay, &mut surface);
        }
    }
}

mod debug_overlay;
pub use debug_overlay::DebugOverlay;

mod fps_overlay;
pub use fps_overlay::FpsOverlay;
