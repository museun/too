use std::{any::Any, cell::RefCell, collections::HashMap, ops::Deref};

use crate::{
    hasher::{hash_fnv_1a, DefaultIntHasher},
    layout::Axis,
    Rgba,
};

use super::ViewId;

pub struct AxisProperty<T: 'static> {
    pub horizontal: T,
    pub vertical: T,
}

impl<T> From<AxisProperty<T>> for (T, T) {
    fn from(value: AxisProperty<T>) -> Self {
        (value.horizontal, value.vertical)
    }
}

impl<T: 'static> AxisProperty<T> {
    pub const fn new(horizontal: T, vertical: T) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub const fn same(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            horizontal: value,
            vertical: value,
        }
    }
}

impl<T: 'static + Copy> AxisProperty<T> {
    pub const fn resolve(&self, axis: Axis) -> T {
        match axis {
            Axis::Horizontal => self.horizontal,
            Axis::Vertical => self.vertical,
        }
    }
}

impl<T: 'static + Copy> Copy for AxisProperty<T> {}
impl<T: 'static + Copy> Clone for AxisProperty<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// FIXME this should be constrained by some sealed trait so 'stylesheet' isn't
// used as a generic any map

// this should also be an enum, so 'default()' can be lazy
// and so we specialize for Style<Style<Rgba>> (i.e. deferred to the Theme)
pub struct Styled<T: 'static> {
    key: u64,
    default: T,
}

impl<T: 'static> PartialEq for Styled<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: 'static + Copy> Copy for Styled<T> {}
impl<T: 'static + Copy> Clone for Styled<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Styled<T> {
    pub const fn new(key: &'static str, default: T) -> Self {
        Self {
            key: hash_fnv_1a(key.as_bytes()),
            default,
        }
    }

    pub const fn default(&self) -> T
    where
        T: Copy,
    {
        self.default
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub surface: Rgba,
    pub outline: Rgba,
    pub contrast: Rgba,
    pub primary: Rgba,
    pub secondary: Rgba,
    pub accent: Rgba,
    pub danger: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub info: Rgba,
}

impl Theme {
    pub const DEFAULT: Self = Self::DARK;

    pub const DARK: Self = Self::dark();
    pub const LIGHT: Self = Self::light();

    pub const BACKGROUND: Styled<Rgba> =
        Styled::new("too.theme.background", Self::DEFAULT.background);
    pub const FOREGROUND: Styled<Rgba> =
        Styled::new("too.theme.foreground", Self::DEFAULT.foreground);
    pub const SURFACE: Styled<Rgba> = //
        Styled::new("too.theme.surface", Self::DEFAULT.surface);
    pub const OUTLINE: Styled<Rgba> = //
        Styled::new("too.theme.outline", Self::DEFAULT.outline);
    pub const CONTRAST: Styled<Rgba> = //
        Styled::new("too.theme.contrast", Self::DEFAULT.contrast);
    pub const PRIMARY: Styled<Rgba> = //
        Styled::new("too.theme.primary", Self::DEFAULT.primary);
    pub const SECONDARY: Styled<Rgba> = //
        Styled::new("too.theme.secondary", Self::DEFAULT.secondary);
    pub const ACCENT: Styled<Rgba> = //
        Styled::new("too.theme.accent", Self::DEFAULT.accent);
    pub const DANGER: Styled<Rgba> = //
        Styled::new("too.theme.danger", Self::DEFAULT.danger);
    pub const SUCCESS: Styled<Rgba> = //
        Styled::new("too.theme.success", Self::DEFAULT.success);
    pub const WARNING: Styled<Rgba> = //
        Styled::new("too.theme.warning", Self::DEFAULT.warning);
    pub const INFO: Styled<Rgba> = //
        Styled::new("too.theme.info", Self::DEFAULT.info);
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    pub const fn light() -> Self {
        Self {
            background: Rgba::hex("#E0E0E0"),
            foreground: Rgba::hex("#000000"),
            surface: Rgba::hex("#A3A5A8"),
            outline: Rgba::hex("#9D7278"),
            contrast: Rgba::hex("#763636"),
            primary: Rgba::hex("#8175DF"),
            secondary: Rgba::hex("#B8A52D"),
            accent: Rgba::hex("#776BC2"),
            danger: Rgba::hex("#C7343B"),
            success: Rgba::hex("#33D17A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#0077C2"),
        }
    }

    pub const fn dark() -> Self {
        Self {
            background: Rgba::hex("#131313"),
            foreground: Rgba::hex("#FFFFFF"),
            surface: Rgba::hex("#232323"),
            outline: Rgba::hex("#4D4D4D"),
            contrast: Rgba::hex("#A9E9E9"),
            primary: Rgba::hex("#55B1F0"),
            secondary: Rgba::hex("#8C8BED"),
            accent: Rgba::hex("#F4A151"),
            danger: Rgba::hex("#F05D61"),
            success: Rgba::hex("#9AF07A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#6A7DDA"),
        }
    }
}

pub struct Knob;
impl Knob {
    pub const LARGE: char = Elements::LARGE_RECT;
    pub const MEDIUM: char = Elements::MEDIUM_RECT;
    pub const SMALL: char = Elements::SMALL_RECT;
    pub const ROUND: char = Elements::CIRCLE;
    pub const DIAMOND: char = Elements::DIAMOND;
}

pub struct Elements;
impl Elements {
    pub const LARGE_RECT: char = '█';
    pub const MEDIUM_RECT: char = '■';
    pub const SMALL_RECT: char = '▮';

    pub const CIRCLE: char = '●';
    pub const DIAMOND: char = '◆';

    pub const HORIZONTAL_LINE: char = '─';
    pub const THICK_HORIZONTAL_LINE: char = '━';
    pub const DASH_HORIZONTAL_LINE: char = '╌';
    pub const THICK_DASH_HORIZONTAL_LINE: char = '╍';

    pub const VERTICAL_LINE: char = '│';
    pub const THICK_VERTICAL_LINE: char = '┃';
    pub const DASH_VERTICAL_LINE: char = '╎';
    pub const THICK_DASH_VERTICAL_LINE: char = '╏';
}

#[derive(Default)]
pub struct Stylesheet {
    map: HashMap<u64, Box<dyn Any>, DefaultIntHasher>,
    scoped: RefCell<HashMap<ViewId, Vec<ScopedStyle>, DefaultIntHasher>>,
}

struct ScopedStyle {
    key: u64,
    value: Box<dyn Any>,
}

impl std::fmt::Debug for ScopedStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopedStyle")
            .field("key", &self.key)
            .field("type_id", &self.value.deref().type_id())
            .field("addr", &format_args!("{:p}", self.value))
            .finish()
    }
}

impl Stylesheet {
    pub(in crate::view) fn swap(&mut self, id: ViewId) {
        let Some(list) = self.scoped.get_mut().get_mut(&id) else {
            return;
        };

        for ScopedStyle { key, value } in list {
            if let Some(old) = self.map.get_mut(&*key) {
                std::mem::swap(value, old);
            }
        }
    }

    pub(in crate::view) fn reset(&mut self) {
        self.scoped.get_mut().clear();
    }

    pub(in crate::view) fn reset_style<T>(&self, id: ViewId, key: Styled<T>)
    where
        T: 'static + Copy,
    {
        self.scoped
            .borrow_mut()
            .entry(id)
            .or_default()
            .retain(|style| style.key != key.key);
    }

    pub(in crate::view) fn replace<T>(&self, id: ViewId, key: Styled<T>, new_value: T)
    where
        T: 'static + Copy,
    {
        let value = ScopedStyle {
            key: key.key,
            value: Box::new(new_value),
        };
        self.scoped.borrow_mut().entry(id).or_default().push(value);
    }

    pub fn get<T>(&self, key: Styled<T>) -> Option<T>
    where
        T: 'static + Copy,
    {
        self.map
            .get(&key.key)
            .and_then(|d| d.downcast_ref::<T>())
            .copied()
    }

    pub fn get_or_default<T>(&mut self, key: Styled<T>) -> T
    where
        T: 'static + Copy,
    {
        self.get_or_insert_with(key, move || key.default)
    }

    pub fn get_or_insert_with<T>(&mut self, key: Styled<T>, insert: impl FnOnce() -> T) -> T
    where
        T: 'static + Copy,
    {
        *self
            .map
            .entry(key.key)
            .or_insert_with(|| Box::new(insert()))
            .downcast_ref::<T>()
            .unwrap()
    }
}
