use crate::{App, Backend, EventReader, Overlay, Runner};

/// A trait to run your application
///
/// It is implemented for all types that implement [`App`].
///
/// # Example:
/// ```rust
/// use too::{SurfaceMut, Context, AppRunner as _};
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
/// impl too::App for Demo {
///     fn render(&mut self, surface: SurfaceMut, ctx: Context) {}
/// }
///
/// # fn get_backend() -> std::io::Result<too::DummyBackend> { Ok(too::DummyBackend) }
/// fn main() -> std::io::Result<()> {
///     let backend = get_backend()?;
///     Demo::new(1234).run(backend)
/// }
/// ```
pub trait AppRunner: App + Sealed + Sized {
    /// Run the [`App`] with the provided [`Backend`] and [`EventReader`]
    fn run(self, backend: impl Backend + EventReader) -> std::io::Result<()> {
        Runner::new()
            .min_ups(Self::min_ups)
            .max_ups(Self::max_ups)
            .init(Self::initial_size)
            .event(Self::event)
            .update(Self::update)
            .render(Self::render)
            .post_render(|ctx, overlay, surface| Overlay::default_draw(ctx, overlay, surface))
            .run(self, backend)
    }
}

#[doc(hidden)]
pub trait Sealed {}

impl<T> Sealed for T {}
impl<T: App + Sealed> AppRunner for T {}
