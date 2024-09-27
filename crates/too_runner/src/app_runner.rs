use too_events::EventReader;
use too_layout::LinearLayout;
use too_math::vec2;
use too_renderer::{Backend, SurfaceMut};
use too_shapes::Text;

use crate::{App, Overlay, Runner};

/// A trait to run your application
///
/// It is implemented for all types that implement [`App`].
///
/// # Example:
/// ```rust
/// use too_runner::{AppRunner as _, SurfaceMut};
///
/// struct Demo {
///     state: i32
/// }
///
/// impl Demo {
///     fn new(state: i32) -> Self {
///         Self { state }
///     }
/// }
///
/// impl too_runner::App for Demo {
///     fn render(&mut self, surface: &mut SurfaceMut) {}
/// }
///
/// # fn get_backend() -> std::io::Result<too_runner::dummy::Dummy> { Ok(too_runner::dummy::Dummy) }
/// fn main() -> std::io::Result<()> {
///     let backend = get_backend()?;
///     Demo::new(1234).run(backend)
/// }
/// ```
pub trait AppRunner: App + Sealed + Sized {
    /// Run the [`App`] with the provided [`Backend`] and [`EventReader`]
    fn run(self, term: impl Backend + EventReader) -> std::io::Result<()> {
        Runner::new()
            .min_ups(Self::min_ups)
            .max_ups(Self::max_ups)
            .init(Self::initial_size)
            .event(Self::event)
            .update(Self::update)
            .render(Self::render)
            .post_render(draw_default_overlay)
            .run(self, term)
    }
}

/// Draws the default fps overlay
///
/// This does not check whether its shown -- thats up to you
pub fn draw_default_fps_overlay(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
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

/// Draws the default debug overlay
///
/// This does not check whether its shown -- thats up to you
pub fn draw_default_debug_overlay(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
    let mut alloc = LinearLayout::new(overlay.debug.axis)
        .anchor(overlay.debug.anchor)
        .wrap(true)
        .spacing(vec2(1, 0))
        .layout(surface.rect());

    let (fg, bg) = (overlay.debug.fg, overlay.debug.bg);

    for msg in overlay.debug.queue.drain(..).rev() {
        let part = Text::new(msg).fg(fg).bg(bg);
        if let Some(rect) = alloc.allocate(part.size()) {
            surface.crop(rect).draw(part);
        }
    }
}

/// A default overlay draw function.
///
/// If you use a custom [`Runner`] you can use this for [`Runner::post_render`]
pub fn draw_default_overlay(_: &mut impl App, overlay: &mut Overlay, mut surface: SurfaceMut<'_>) {
    if overlay.fps.show {
        draw_default_fps_overlay(overlay, &mut surface);
    }

    if overlay.debug.show {
        draw_default_debug_overlay(overlay, &mut surface);
    }
}

#[doc(hidden)]
pub trait Sealed {}

impl<T> Sealed for T {}
impl<T: App + Sealed> AppRunner for T {}
