use crate::{
    view::{Adhoc, Palette, Response, StyleKind},
    Rgba, Str,
};

use super::label::{label, LabelStyle};

#[derive(Debug)]
pub struct SelectedStyle {
    pub text_color: Rgba,

    pub background: Rgba,
    pub selected_background: Rgba,

    pub hovered_text: Option<Rgba>,
    pub hovered_background: Option<Rgba>,
}

impl SelectedStyle {
    pub fn default(palette: &Palette, _selected: bool) -> Self {
        Self {
            text_color: palette.foreground,
            background: palette.outline,
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

pub type SelectedClass = fn(&Palette, bool) -> SelectedStyle;

#[derive(Debug)]
#[must_use = "a view does nothing unless `ui.adhoc()` is called"]
pub struct Selected<'a> {
    value: &'a mut bool,
    label: Str,
    class: StyleKind<SelectedClass, SelectedStyle>,
}

impl<'a> Selected<'a> {
    pub const fn class(mut self, class: SelectedClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: SelectedStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v> Adhoc<'v> for Selected<'v> {
    type Output = Response<bool>;

    fn show(self, ui: &crate::view::Ui) -> Self::Output {
        let resp = ui
            .mouse_area(|ui| {
                let style = match self.class {
                    StyleKind::Deferred(style) => (style)(&ui.palette(), *self.value),
                    StyleKind::Direct(style) => style,
                };

                let hovered = ui.is_hovered();
                let fill = match (hovered, *self.value) {
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

        *self.value ^= resp.clicked();
        resp.map(|c| c.clicked())
    }
}

pub fn selected(value: &mut bool, label: impl Into<Str>) -> Selected<'_> {
    Selected {
        value,
        label: label.into(),
        class: StyleKind::Deferred(SelectedStyle::default),
    }
}
