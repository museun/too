use too_events::Event;
use too_renderer::SurfaceMut;
use too_runner::Context;

/// Trait for defining an application to run
pub trait App {
    /// The initial surface size that you can use to compute some internal state
    fn initial_size(&mut self, ctx: Context<'_>) {
        _ = ctx
    }

    /// An [`Event`] was sent from the backend
    ///
    /// This provides a [`Context`], and the current surface `size`
    ///
    /// This [`Context`] will let you send commands to the backend and configure the overlays
    fn event(&mut self, event: Event, ctx: Context<'_>) {
        _ = event;
        _ = ctx;
    }

    /// Update allows you to interpolate state
    ///
    /// `dt` is the delta-time that can be used for interpolation
    /// `size` is the current surface size
    fn update(&mut self, dt: f32, ctx: Context<'_>) {
        _ = dt;
        _ = ctx;
    }

    /// Min UPS is the minimum UPS the runner should perform
    ///
    /// The default is 10 updates/s
    fn min_ups(&self) -> f32 {
        10.0
    }

    /// Max UPS is the maximum UPS the runner should perform
    ///
    /// The default is 60 updates/s
    fn max_ups(&self) -> f32 {
        60.0
    }

    /// Render your application
    ///
    /// This provides you with a [`SurfaceMut`] that allows you to draw onto
    ///
    /// The draw order are back-to-front. Later draw calls will be drawn over earlier calls
    fn render(&mut self, surface: SurfaceMut, ctx: Context<'_>);
}
