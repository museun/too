use too::{Key, Keybind, Modifiers};

use crate::{
    view::Context, Event, EventCtx, Handled, Interest, NoArgs, Response, UpdateCtx, View, ViewExt,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KeyAreaResponse {
    // TODO maybe a `Keybind`
    pub key: Option<Key>,
    pub modifiers: Option<Modifiers>,
}

impl KeyAreaResponse {
    pub fn is_keybind(&self, keybind: impl Into<Keybind>) -> bool {
        let Some(key) = self.key else {
            return false;
        };

        let modifiers = self.modifiers.unwrap_or_default();

        let expected: Keybind = keybind.into();
        if matches!(expected.key, Key::Char(..))
            && expected.modifiers.is_none()
            && modifiers.is_none()
        {
            return key == expected.key;
        }
        Keybind::new(key, modifiers) == expected
    }
}

#[derive(Debug)]
struct KeyArea {
    key: Option<Key>,
    modifiers: Option<Modifiers>,
}

impl<T: 'static> View<T> for KeyArea {
    type Args<'a> = NoArgs;
    type Response = KeyAreaResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            key: None,
            modifiers: None,
        }
    }

    fn interest(&self) -> Interest {
        Interest::KEY_INPUT
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        Self::Response {
            key: std::mem::take(&mut self.key),
            modifiers: std::mem::take(&mut self.modifiers),
        }
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        if let Event::KeyInput(ev) = event {
            self.key = Some(ev.key);
            self.modifiers = Some(ev.modifiers);
        }
        Handled::Bubble
    }
}

pub fn key_area<T: 'static, R>(
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<KeyAreaResponse, R> {
    KeyArea::show_children((), ctx, show)
}

pub fn hot_key<T: 'static, R>(
    keybind: impl Into<Keybind>,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<bool, R> {
    let resp = key_area(ctx, show);
    let pressed = resp.is_keybind(keybind);
    Response::new(resp.view_id(), pressed, resp.into_inner())
}

pub fn key_press<const N: usize, T: 'static, R>(
    keys: [Keybind; N],
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<[bool; N], R> {
    let resp = key_area(ctx, show);
    let mut out = [false; N];
    for (key, result) in keys.into_iter().zip(&mut out) {
        *result = resp.is_keybind(key)
    }
    Response::new(resp.view_id(), out, resp.into_inner())
}
