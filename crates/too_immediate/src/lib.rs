mod app;
pub use app::App;

mod app_runner;
pub use app_runner::AppRunner;

// Hide this from the docs
// #[cfg(doctests)] doesn't work as expected here
#[doc(hidden)]
pub mod dummy;
