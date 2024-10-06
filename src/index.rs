//! Typed key support

use crate::hasher::hash_fnv_1a;

// TODO rename this
/// Typed key for some data.
///
/// This turns a `&'static str` into something more compact and efficient to
/// compare, at compile time
///
/// You can turn a `&str` into this via [`Index::from`]
///
/// If you use namespace separation, remember that the parts are separated by a `.`
///
/// e.g. `namespace.key.sub_key.another_sub_key`
pub struct Index<T: ?Sized = ()> {
    pub(crate) key: u64,
    _marker: std::marker::PhantomData<fn(&T)>,
}

// impl From<ViewId> for Index<()> {
//     fn from(value: ViewId) -> Self {
//         Self::from_id(value)
//     }
// }

// impl Index<()> {
//     /// Construct a key from a [`ViewId`]
//     pub const fn from_id(id: ViewId) -> Self {
//         unimplemented!()
//         // Self::from_raw(id.0.to_bits())
//     }
// }

impl<T: ?Sized> Index<T> {
    /// Create a new key.
    ///
    /// Keys can have namespaces.
    ///
    /// The form is: `namespace.key.sub_key`
    pub const fn namespace(key: &str) -> Self {
        Self {
            key: hash_fnv_1a(key.as_bytes()),
            _marker: std::marker::PhantomData,
        }
    }

    /// Add a key to this namespace
    pub const fn with(mut self, key: &str) -> Self {
        self.key ^= hash_fnv_1a(key.as_bytes());
        self
    }

    /// Construct a key from a raw unique identifier
    pub const fn from_raw(raw: u64) -> Self {
        Self {
            key: raw,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for Index<T> {}
impl<T: ?Sized> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: ?Sized> Eq for Index<T> {}

impl<T: ?Sized> PartialOrd for Index<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ?Sized> Ord for Index<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

// TODO decide if we want to implement Hash

impl<T: ?Sized> std::fmt::Debug for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NoQuote<'a>(&'a str);
        impl<'a> std::fmt::Debug for NoQuote<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.0)
            }
        }
        f.debug_struct("Index")
            .field("type", &NoQuote(std::any::type_name::<T>()))
            .field("key", &self.key)
            .finish()
    }
}

impl From<&str> for Index {
    fn from(value: &str) -> Self {
        match value.split_once('.') {
            Some((head, tail)) => tail.split('.').fold(Self::namespace(head), Self::with),
            None => Self::namespace(value),
        }
    }
}
