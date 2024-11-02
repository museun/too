use crate::{
    view::{Builder, EventCtx, Handled, Interest, Ui, View, ViewEvent},
    Key, Keybind, Modifiers,
};

#[derive(Debug)]
pub struct KeyArea {
    key: Option<Key>,
    modifiers: Option<Modifiers>,
}

impl<'v> Builder<'v> for KeyArea {
    type View = Self;
}

impl View for KeyArea {
    type Args<'v> = Self;
    type Response = KeyAreaResponse;

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        Self::Response {
            key: self.key.take(),
            modifiers: self.modifiers.take(),
        }
    }

    fn interests(&self) -> Interest {
        Interest::FOCUS_INPUT
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        let ViewEvent::KeyInput { key, modifiers } = event else {
            return Handled::Bubble;
        };

        self.key = Some(key);
        self.modifiers = Some(modifiers);

        Handled::Sink
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct KeyAreaResponse {
    key: Option<Key>,
    modifiers: Option<Modifiers>,
}

impl KeyAreaResponse {
    pub fn key_binding(&self) -> Option<Keybind> {
        self.key
            .and_then(|k| self.modifiers.map(|m| (k, m)))
            .map(|(k, m)| Keybind::new(k, m))
    }

    pub fn key_pressed(&self, key: impl Into<Keybind>) -> bool {
        let Some(got) = self.key else {
            return false;
        };

        let got = Keybind::new(got, self.modifiers.unwrap_or_default());
        got == key.into()
    }

    pub fn is_shift(&self) -> bool {
        self.modifiers.filter(|m| m.is_shift()).is_some()
    }

    pub fn is_ctrl(&self) -> bool {
        self.modifiers.filter(|m| m.is_ctrl()).is_some()
    }

    pub fn is_alt(&self) -> bool {
        self.modifiers.filter(|m| m.is_alt()).is_some()
    }

    pub fn is_shift_only(&self) -> bool {
        self.modifiers.filter(|m| m.is_shift_only()).is_some()
    }

    pub fn is_ctrl_only(&self) -> bool {
        self.modifiers.filter(|m| m.is_ctrl_only()).is_some()
    }

    pub fn is_alt_only(&self) -> bool {
        self.modifiers.filter(|m| m.is_alt_only()).is_some()
    }
}

pub const fn key_area() -> KeyArea {
    KeyArea {
        key: None,
        modifiers: None,
    }
}
