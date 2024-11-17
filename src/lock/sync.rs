use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

// TODO determine if we're read-heavy and switch to RwLock if we are
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

/// A (cheaply) clonable pointer
#[derive(Clone, Default)]
pub struct Shared<T>
where
    T: ?Sized,
{
    inner: Arc<T>,
}

impl<T> Deref for Shared<T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: std::fmt::Debug> Debug for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Shared<T> {
    /// Create a new [`Shared`] from a value
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Clone this [`Shared`]
    pub fn clone(this: &Self) -> Self {
        Self {
            inner: Arc::clone(&this.inner),
        }
    }
}

/// An immutable reference
pub struct Ref<'a, T>
where
    T: ?Sized,
{
    inner: RwLockReadGuard<'a, T>,
}

impl<'a, T: Debug> Debug for Ref<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl<'a, T> Deref for Ref<'a, T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// An immutable reference
pub struct RefMapped<'a, T>
where
    T: ?Sized,
{
    inner: MappedRwLockReadGuard<'a, T>,
}

impl<'a, T> RefMapped<'a, T>
where
    T: ?Sized,
{
    /// See [`RwLock::map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.MappedRwLockReadGuard.html#method.map)
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: MappedRwLockReadGuard::map(this.inner, |inner| map(inner)),
        }
    }

    /// See [`RwLock::try_map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.MappedRwLockReadGuard.html#method.try_map)
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(this: Self, map: impl FnOnce(&T) -> Option<&U>) -> Option<RefMapped<'a, U>>
    where
        U: ?Sized,
    {
        MappedRwLockReadGuard::try_map(this.inner, map)
            .map(|inner| RefMapped { inner })
            .ok()
    }
}

impl<'a, T: Debug> Debug for RefMapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl<'a, T> Deref for RefMapped<'a, T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> Ref<'a, T>
where
    T: ?Sized,
{
    /// See [`RwLock::map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.RwLockReadGuard.html#method.map)
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: RwLockReadGuard::map(this.inner, |inner| map(inner)),
        }
    }

    /// See [`RwLock::try_map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.RwLockReadGuard.html#method.try_map)
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(this: Self, map: impl FnOnce(&T) -> Option<&U>) -> Option<RefMapped<'a, U>>
    where
        U: ?Sized,
    {
        RwLockReadGuard::try_map(this.inner, map)
            .map(|inner| RefMapped { inner })
            .ok()
    }
}

/// A mutable reference
pub struct RefMut<'a, T>
where
    T: ?Sized,
{
    inner: RwLockWriteGuard<'a, T>,
}

impl<'a, T: Debug> Debug for RefMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl<'a, T> Deref for RefMut<'a, T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for RefMut<'a, T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<'a, T> RefMut<'a, T>
where
    T: ?Sized,
{
    /// See [`RwLock::map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.RwLockWriteGuard.html#method.map)
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: RwLockWriteGuard::map(this.inner, |inner| map(inner)),
        }
    }

    /// See [`RwLock::try_map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.RwLockWriteGuard.html#method.try_map)
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(
        this: Self,
        map: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<RefMutMapped<'a, U>>
    where
        U: ?Sized,
    {
        RwLockWriteGuard::try_map(this.inner, map)
            .map(|inner| RefMutMapped { inner })
            .ok()
    }
}

/// A mutable reference
pub struct RefMutMapped<'a, T>
where
    T: ?Sized,
{
    inner: MappedRwLockWriteGuard<'a, T>,
}

impl<'a, T> RefMutMapped<'a, T>
where
    T: ?Sized,
{
    /// See [`RwLock::map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.MappedRwLockWriteGuard.html#method.map)
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: MappedRwLockWriteGuard::map(this.inner, |inner| map(inner)),
        }
    }

    /// See [`RwLock::try_map`](https://docs.rs/lock_api/0.4.12/lock_api/struct.MappedRwLockWriteGuard.html#method.try_map)
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(
        this: Self,
        map: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<RefMutMapped<'a, U>>
    where
        U: ?Sized,
    {
        MappedRwLockWriteGuard::try_map(this.inner, map)
            .map(|inner| RefMutMapped { inner })
            .ok()
    }
}

impl<'a, T: Debug> Debug for RefMutMapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl<'a, T> Deref for RefMutMapped<'a, T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for RefMutMapped<'a, T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

/// An interior mutable container for a type
#[derive(Default)]
pub struct Lock<T> {
    inner: RwLock<T>,
}

impl<T: Debug> Debug for Lock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lock")
            .field("inner", &*self.inner.read())
            .finish()
    }
}

impl<T> Lock<T> {
    /// Create a new Lock for this value
    pub const fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
        }
    }

    /// Gets immutable borrow to the internal data
    ///
    /// ***WARNING:*** This'll deadlock if any mutable borrows are outstanding
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref {
            inner: self.inner.read(),
        }
    }

    /// Gets mutable borrow to the internal data
    ///
    /// ***WARNING:*** This'll deadlock if any immutable or mutable borrows are outstanding
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut {
            inner: self.inner.write(),
        }
    }

    /// Gets mutable to the internal data.
    ///
    /// This will not panic because we have &mut access to the lock
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// Consumes the lock, returning the internal data
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}
