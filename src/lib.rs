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
pub use compact_str::format_compact as __dont_use_this_because_semver;

#[cfg(feature = "terminal")]
mod run;
#[cfg(feature = "terminal")]
pub use run::{application, run, Config as RunConfig};
