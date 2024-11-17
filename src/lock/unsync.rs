use std::{
    cell::RefCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// A (cheaply) clonable pointer
#[derive(Clone, Default)]
pub struct Shared<T>
where
    T: ?Sized,
{
    inner: Rc<T>,
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
            inner: Rc::new(value),
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Clone this [`Shared`]
    pub fn clone(this: &Self) -> Self {
        Self {
            inner: Rc::clone(&this.inner),
        }
    }
}

/// An immutable reference
pub struct Ref<'a, T>
where
    T: ?Sized,
{
    inner: std::cell::Ref<'a, T>,
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

impl<'a, T: std::fmt::Display> std::fmt::Display for Ref<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// An immutable reference
pub struct RefMapped<'a, T>
where
    T: ?Sized,
{
    inner: std::cell::Ref<'a, T>,
}

impl<'a, T> RefMapped<'a, T>
where
    T: ?Sized,
{
    /// See [`std::cell::Ref::map`]
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: std::cell::Ref::map(this.inner, map),
        }
    }

    /// See [`std::cell::Ref::filter_map`]
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(this: Self, map: impl FnOnce(&T) -> Option<&U>) -> Option<RefMapped<'a, U>>
    where
        U: ?Sized,
    {
        std::cell::Ref::filter_map(this.inner, map)
            .map(|inner| RefMapped { inner })
            .ok()
    }
}

impl<'a, T: Debug> Debug for RefMapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl<'a, T: std::fmt::Display> std::fmt::Display for RefMapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
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
    /// See [`std::cell::Ref::map`]
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: std::cell::Ref::map(this.inner, map),
        }
    }

    /// See [`std::cell::Ref::filter_map`]
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(this: Self, map: impl FnOnce(&T) -> Option<&U>) -> Option<RefMapped<'a, U>>
    where
        U: ?Sized,
    {
        std::cell::Ref::filter_map(this.inner, map)
            .map(|inner| RefMapped { inner })
            .ok()
    }
}

/// A mutable reference
pub struct RefMut<'a, T>
where
    T: ?Sized,
{
    inner: std::cell::RefMut<'a, T>,
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

/// A mutable reference
pub struct RefMutMapped<'a, T>
where
    T: ?Sized,
{
    inner: std::cell::RefMut<'a, T>,
}

impl<'a, T> RefMutMapped<'a, T>
where
    T: ?Sized,
{
    /// See [`std::cell::RefMut::map`]
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: std::cell::RefMut::map(this.inner, map),
        }
    }

    /// See [`std::cell::RefMut::filter_map`]
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(
        this: Self,
        map: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<RefMutMapped<'a, U>>
    where
        U: ?Sized,
    {
        std::cell::RefMut::filter_map(this.inner, map)
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

impl<'a, T> RefMut<'a, T>
where
    T: ?Sized,
{
    /// See [`std::cell::RefMut::map`]
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: std::cell::RefMut::map(this.inner, map),
        }
    }

    /// See [`std::cell::RefMut::filter_map`]
    ///
    /// ***NOTE*** this returns an Option instead of the original reference
    pub fn filter_map<U>(
        this: Self,
        map: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<RefMutMapped<'a, U>>
    where
        U: ?Sized,
    {
        std::cell::RefMut::filter_map(this.inner, map)
            .map(|inner| RefMutMapped { inner })
            .ok()
    }
}

/// An interior mutable container for a type
#[derive(Default)]
pub struct Lock<T>
where
    T: ?Sized,
{
    inner: RefCell<T>,
}

impl<T: Debug> Debug for Lock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lock")
            .field("inner", &*self.inner.borrow())
            .finish()
    }
}

impl<T> Lock<T> {
    /// Create a new Lock for this value
    pub const fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// Gets immutable borrow to the internal data
    ///
    /// ***WARNING:*** This'll panic if any mutable borrows are outstanding
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref {
            inner: self.inner.borrow(),
        }
    }

    /// Gets mutable borrow to the internal data
    ///
    /// ***WARNING:*** This'll panic if any immutable or mutable borrows are outstanding
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut {
            inner: self.inner.borrow_mut(),
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
