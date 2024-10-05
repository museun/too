//! Typed key support

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
}

#[inline(always)]
const fn hash_fnv_1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }
    hash
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

/// [`Hasher`](std::hash::Hash) for [`Index`]
#[derive(Default)]
pub struct IndexHasher(u64);

impl std::hash::Hasher for IndexHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }
}

/// [`BuildHasher`](std::hash::BuildHasher) for [`Index`]
#[derive(Default)]
pub struct BuildIndexHasher;

impl std::hash::BuildHasher for BuildIndexHasher {
    type Hasher = IndexHasher;
    fn build_hasher(&self) -> Self::Hasher {
        IndexHasher::default()
    }
}
