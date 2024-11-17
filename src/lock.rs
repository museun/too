#[cfg(not(feature = "sync"))]
mod unsync;

#[cfg(feature = "sync")]
mod sync;

#[cfg(not(feature = "sync"))]
pub use unsync::*;

#[cfg(feature = "sync")]
pub use sync::*;
