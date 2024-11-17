use std::ops::Deref;

use compact_str::{CompactString, ToCompactString};

/// A semver wrapper around a [`CompactString`](https://docs.rs/compact_str/0.8.0/compact_str/index.html)
///
/// You would normally not need to name this type, anything that implements [`ToCompactString`](https://docs.rs/compact_str/0.8.0/compact_str/trait.ToCompactString.html) can be turned into this type.
///
/// You can use [`format_str!`][crate::format_str] like [`std::format!`] to make this type
/// - or `Str::from(&str)`
/// - or `Str::from(String)`
/// - or `Str::from(usize)`
/// - etc
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Str(CompactString);

impl Str {
    /// Create a new [`Str`] at compile time
    pub const fn new(str: &'static str) -> Self {
        Self(CompactString::const_new(str))
    }

    /// Turn this [`Str`] into a [`CompactString`]
    pub fn into_inner(self) -> CompactString {
        self.0
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<&Str> for Str {
    fn from(value: &Str) -> Self {
        value.clone()
    }
}

impl<T> From<T> for Str
where
    T: ToCompactString,
{
    fn from(value: T) -> Self {
        Self(value.to_compact_string())
    }
}

// basically just https://github.com/ParkMyCar/compact_str/blob/193d13eaa5a92b3c39c2f7289dc44c95f37c80d1/compact_str/src/macros.rs#L28
// but semver-safe
/// Like [`std::format!`] but for a [`Str`]
#[macro_export]
macro_rules! format_str {
    ($($arg:tt)*) => {
        $crate::Str::from($crate::__dont_use_this_because_semver!($($arg)*))
    }
}
