//! Abstraction for providing interior mutability over for a type.
//!
//! By default, the `unsync` types are used. The `sync` flag switches to the `Send + Sync` variants
//!
//! | Type | unsync (default) | sync|
//! | -- | -- | -- |
//! | [`Lock<T>`] | [`RefCell<T>`](std::cell::RefCell) | [`parking_lot::RwLock<T>`](https://docs.rs/parking_lot/0.12.3/parking_lot/type.RwLock.html) |
//! | [`Ref<T>`] | [`Ref<T>`](std::cell::Ref) | [`parking_lot::RwLockReadGuard<T>`](https://docs.rs/parking_lot/0.12.3/parking_lot/type.RwLockReadGuard.html) |
//! | [`RefMut<T>`] | [`RefMut<T>`](std::cell::RefMut) | [`parking_lot::RwLockWriteGuard<T>`](https://docs.rs/parking_lot/0.12.3/parking_lot/type.RwLockWriteGuard.html) |
//! | [`RefMapped<T>`] | [`Ref<T>`](std::cell::Ref) | [`parking_lot::MappedRwLockReadGuard<T>`](https://docs.rs/parking_lot/0.12.3/parking_lot/type.MappedRwLockReadGuard.html) |
//! | [`RefMutMapped<T>`] | [`RefMut<T>`](std::cell::RefMut) | [`parking_lot::MappedRwLockWriteGuard<T>`](https://docs.rs/parking_lot/0.12.3/parking_lot/type.MappedRwLockWriteGuard.html) |
//! | [`Shared<T>`] | [`Rc<T>`](std::rc::Rc) | [`Arc<T>`](std::sync::Arc) |
//!
//!
//! ***NOTE***: [`RefMapped`] and [`RefMutMapped`] are not a type alias but rather
//! new-type wrappers to match the behavior of [**parking_lot**](https://docs.rs/parking_lot/0.12.3/parking_lot/index.html).
//!
//! So [`Ref::map()`] returns a [`RefMapped`] rather than a [`Ref`]

#[cfg(not(feature = "sync"))]
mod unsync;

#[cfg(feature = "sync")]
mod sync;

#[cfg(not(feature = "sync"))]
pub use unsync::*;

#[cfg(feature = "sync")]
pub use sync::*;
