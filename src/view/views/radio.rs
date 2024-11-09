use crate::{
    view::{style::StyleKind, Adhoc, Palette, Response},
    Rgba,
};

use super::label::{label, LabelStyle};

pub type RadioClass = fn(&Palette, bool) -> RadioStyle;

#[derive(Debug)]
pub struct RadioStyle {
    pub text_color: Rgba,

    pub background: Rgba,
    pub selected_background: Rgba,

    pub hovered_text: Option<Rgba>,
    pub hovered_background: Option<Rgba>,
}

impl RadioStyle {
    pub fn default(palette: &Palette, selected: bool) -> Self {
        Self {
            text_color: palette.foreground,
            background: palette.surface,
            selected_background: palette.primary,
            hovered_text: None,
            hovered_background: None,
        }
    }

    pub fn hovered(palette: &Palette, selected: bool) -> Self {
        Self {
            hovered_text: Some(palette.surface),
            hovered_background: Some(palette.secondary),
            ..Self::default(palette, selected)
        }
    }
}

pub struct Radio<'a, V> {
    value: V,
    existing: &'a mut V,
    label: &'a str,
    class: StyleKind<RadioClass, RadioStyle>,
}

impl<'v, V> Adhoc<'v> for Radio<'v, V>
where
    V: PartialEq,
{
    type Output = Response<bool>;

    fn show(self, ui: &crate::view::Ui) -> Self::Output {
        let resp = ui
            .mouse_area(|ui| {
                let style = match self.class {
                    StyleKind::Deferred(style) => {
                        (style)(&ui.palette(), self.value == *self.existing)
                    }
                    StyleKind::Direct(style) => style,
                };

                let hovered = ui.is_hovered();
                let fill = match (hovered, self.value == *self.existing) {
                    (false, true) => style.selected_background,
                    (false, false) => style.background,
                    (true, true) => style
                        .hovered_background
                        .unwrap_or(style.selected_background),
                    (true, false) => style.hovered_background.unwrap_or(style.background),
                };

                let text = if hovered {
                    style.hovered_text.unwrap_or(style.text_color)
                } else {
                    style.text_color
                };

                ui.background(fill, |ui| {
                    ui.show(label(self.label).style(LabelStyle { foreground: text }))
                });
            })
            .flatten_left();

        if resp.clicked() {
            *self.existing = self.value;
        }
        resp.map(|c| c.clicked())
    }
}

pub fn radio<'a, V>(value: V, existing: &'a mut V, label: &'a str) -> Radio<'a, V>
where
    V: PartialEq,
{
    Radio {
        value,
        existing,
        label,
        class: StyleKind::deferred(RadioStyle::default),
    }
}
