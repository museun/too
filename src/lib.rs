//! too -- a different kind of tui library
//!
//! # Feature flags
//! | Flag | Description | Default |
//! | --- | --- | -- |
//! |`terminal` | enable the terminal backend | `true` |
//! |`sync` | enable `Send`+`Sync` wrappers | `false` |
//! |`profile` | enable [`profiling`](https://docs.rs/profiling/1.0.16/profiling/index.html) support | `false` |
//!
//! # Simple examples
//! ## Centering some text:
//! ```no_run
//! fn main() -> std::io::Result<()> {
//!     too::run(|ui| {
//!         ui.center(|ui| ui.label("hello world"));
//!     })
//! }
//! ```
//! ## A pair of buttons to increment and decrement a counter
//! ```no_run
//! fn main() -> std::io::Result<()> {
//!     let mut counter = 0;
//!     too::run(|ui| {
//!         ui.vertical(|ui|{
//!             ui.horizontal(|ui|{
//!                 if ui.button("add 1").clicked() {
//!                     counter += 1;
//!                 }
//!                 if ui.button("subtract 1").clicked() {
//!                     counter -= 1;
//!                 }
//!             });
//!             ui.label(counter)
//!         });
//!     })
//! }
//! ```
//! ## Storing state in a struct
//! ```no_run
//! use too::view::Ui;
//!
//! #[derive(Default)]
//! struct App {
//!     value: f32
//! }
//!
//! impl App {
//!     fn view(&mut self, ui: &Ui) {
//!         ui.slider(&mut self.value);
//!     }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     let mut app = App::default();
//!     too::run(|ui| app.view(ui))
//! }
//! ```
//! ## Storing state seperately from an application
//! ```no_run
//! use too::view::Ui;
//!
//! #[derive(Default)]
//! struct State {
//!     value: f32
//! }
//!
//! struct App;
//!
//! impl App {
//!     fn view(&self, state: &mut State, ui: &Ui) {
//!         ui.slider(&mut state.value);
//!     }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     let app = App;
//!     let mut state = State::default();
//!     too::run(|ui| app.view(&mut state, ui))
//! }
//! ```
//!
//! Some pre-made views are provided in: [`too::views`](crate::views)
//!
pub mod animation;
pub mod backend;
pub mod layout;
pub mod math;
pub mod renderer;

pub mod view;
pub mod views;

pub mod lock;

#[cfg(feature = "terminal")]
pub mod term;

mod hasher;
pub mod helpers;

#[macro_use]
mod str;
pub use str::Str;

#[doc(hidden)]
pub use compact_str::format_compact as ඞ_dont_use_this_because_semver;

#[cfg(feature = "terminal")]
mod run;
#[cfg(feature = "terminal")]
pub use run::{application, run, RunConfig};

#[doc(hidden)]
pub fn ඞrun_in_docs<R: 'static>(app: impl FnMut(&crate::view::Ui) -> R) -> std::io::Result<()> {
    view::State::default().build(math::rect(math::vec2(80, 25)), app);
    Ok(())
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDocTests;
