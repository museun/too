use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

// TODO determine if we're read-heavy and switch to RwLock if we are
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

#[derive(Clone, Default)]
pub struct Shared<T>
where
    T: ?Sized,
{
    inner: Arc<T>,
}

impl<T> Deref for Shared<T> {
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
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(this: &Self) -> Self {
        Self {
            inner: Arc::clone(&this.inner),
        }
    }
}

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

impl<'a, T: std::fmt::Display> std::fmt::Display for Ref<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

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
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: MappedRwLockReadGuard::map(this.inner, |inner| map(inner)),
        }
    }

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

impl<'a, T: std::fmt::Display> std::fmt::Display for RefMapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, T> Deref for RefMapped<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> Ref<'a, T>
where
    T: ?Sized,
{
    pub fn map<U>(this: Self, map: impl FnOnce(&T) -> &U) -> RefMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMapped {
            inner: RwLockReadGuard::map(this.inner, |inner| map(inner)),
        }
    }

    pub fn filter_map<U>(this: Self, map: impl FnOnce(&T) -> Option<&U>) -> Option<RefMapped<'a, U>>
    where
        U: ?Sized,
    {
        RwLockReadGuard::try_map(this.inner, map)
            .map(|inner| RefMapped { inner })
            .ok()
    }
}

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

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<'a, T> RefMut<'a, T>
where
    T: ?Sized,
{
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: RwLockWriteGuard::map(this.inner, |inner| map(inner)),
        }
    }

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
    pub fn map<U>(this: Self, map: impl FnOnce(&mut T) -> &mut U) -> RefMutMapped<'a, U>
    where
        U: ?Sized,
    {
        RefMutMapped {
            inner: MappedRwLockWriteGuard::map(this.inner, |inner| map(inner)),
        }
    }

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

impl<'a, T> Deref for RefMutMapped<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for RefMutMapped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

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
    pub const fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        Ref {
            inner: self.inner.read(),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut {
            inner: self.inner.write(),
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}
