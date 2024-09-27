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

fn draw_default_overlay(_: &mut impl App, overlay: &mut Overlay, mut surface: SurfaceMut<'_>) {
    if overlay.fps.show {
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

#[doc(hidden)]
pub trait Sealed {}

impl<T> Sealed for T {}
impl<T: App + Sealed> AppRunner for T {}
