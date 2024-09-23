use std::{borrow::Cow, cell::Ref, char, rc::Rc, sync::Arc};

use too_math::{vec2, Vec2};

pub trait Label: std::fmt::Debug {
    type Static: Label + 'static;
    fn into_static(self) -> Self::Static
    where
        Self: Sized;

    fn size(&self) -> Vec2;
    // FIXME we should not decompose into `char`
    // instead we should just use stack-allocated strings
    // and use those
    fn chars(&self) -> impl Iterator<Item = char>;
}

impl Label for () {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        std::iter::empty()
    }
}

impl Label for bool {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        vec2(if *self { 4 } else { 5 }, 1)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        match *self {
            true => "true",
            false => "false",
        }
        .chars()
    }
}

impl Label for char {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        vec2(1, 1)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        std::iter::once(*self)
    }
}

// impl Label for &'static str {
//     type Static = Self;

//     fn into_static(self) -> Self::Static {
//         self
//     }

//     fn size(&self) -> Vec2 {
//         size_of_str(self)
//     }

//     fn chars(&self) -> impl Iterator<Item = char> {
//         let s = unicode_segmentation::UnicodeSegmentation::graphemes(*self, false);
//         s.flat_map(<str>::chars)
//     }
// }

fn flat_map_graphemes(s: &str) -> impl Iterator<Item = char> + '_ {
    unicode_segmentation::UnicodeSegmentation::graphemes(s, false).flat_map(<str>::chars)
}

impl<'a> Label for &'a str {
    type Static = String;

    fn into_static(self) -> Self::Static {
        self.to_string()
    }

    fn size(&self) -> Vec2 {
        size_of_str(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        flat_map_graphemes(self)
    }
}

impl Label for String {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        size_of_str(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        flat_map_graphemes(self.as_str())
    }
}

impl<'a> Label for Cow<'a, str> {
    type Static = Cow<'static, str>;

    fn into_static(self) -> Self::Static {
        match self {
            Cow::Borrowed(s) => Cow::Owned(s.to_owned()),
            Cow::Owned(s) => Cow::Owned(s),
        }
    }

    fn size(&self) -> Vec2 {
        size_of_str(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        flat_map_graphemes(self)
    }
}

impl Label for i32 {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        vec2(count_signed_digits(*self as isize) as i32, 1)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        signed_digits(*self as isize)
    }
}

impl Label for usize {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        vec2(count_digits(*self) as i32, 1)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        digits(*self)
    }
}

impl<T: Label + 'static> Label for Rc<T> {
    type Static = Rc<T>;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        <T as Label>::size(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        <T as Label>::chars(self)
    }
}

impl<T: Label + 'static> Label for Arc<T> {
    type Static = Arc<T>;

    fn into_static(self) -> Self::Static {
        self
    }

    fn size(&self) -> Vec2 {
        <T as Label>::size(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        <T as Label>::chars(self)
    }
}

impl<'a> Label for Ref<'a, str> {
    type Static = String;

    fn into_static(self) -> Self::Static {
        self.to_string()
    }

    fn size(&self) -> Vec2 {
        size_of_str(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        flat_map_graphemes(self)
    }
}

impl<T: Label + Clone> Label for &T {
    type Static = T::Static;

    fn into_static(self) -> Self::Static {
        <T as Label>::into_static(self.clone())
    }

    fn size(&self) -> Vec2 {
        <T as Label>::size(self)
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        <T as Label>::chars(self)
    }
}

// TODO make this better
fn size_of_str(s: &str) -> Vec2 {
    let mut size = vec2(0, 1);
    let mut max_x = 0;
    for ch in s.chars() {
        if ch == '\n' {
            size.y += 1;
            size.x = std::mem::take(&mut max_x).max(size.x);
            continue;
        }
        max_x += 1;
    }
    size.x = size.x.max(max_x);
    size
}

fn count_signed_digits(d: isize) -> usize {
    let signed = d.is_negative() as usize;
    let len = count_digits(d.unsigned_abs());
    len + signed
}

fn signed_digits(d: isize) -> impl Iterator<Item = char> {
    d.is_negative()
        .then_some('-')
        .into_iter()
        .chain(digits(d.unsigned_abs()))
}

fn count_digits(d: usize) -> usize {
    let (mut len, mut n) = (1, 1);
    while len < 20 {
        n *= 10;
        if n > d {
            return len;
        }
        len += 1;
    }
    len
}

fn digits(mut d: usize) -> impl Iterator<Item = char> {
    let x = count_digits(d) as u32 - 1;
    let mut mag = 10usize.pow(x);
    if d < mag {
        mag /= 10
    }

    let mut is_zero = d == 0;
    std::iter::from_fn(move || {
        if std::mem::take(&mut is_zero) {
            return Some(0);
        }
        if mag == 0 {
            return None;
        }
        let n = d / mag;
        d %= mag;
        mag /= 10;
        Some(n as u8)
    })
    .map(int_to_char)
}

const fn int_to_char(c: u8) -> char {
    (c + b'0') as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what() {
        let a: i32 = -10;

        eprintln!("{:?}", a.size());
        eprintln!("{:?}", a.chars().collect::<Vec<_>>());
    }
}
