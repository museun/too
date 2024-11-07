use crate::{
    view::{style::StyleKind, view::Adhoc, Palette, Response, Ui},
    Rgba,
};

use super::{label::LabelStyle, shorthands};

pub type CheckboxClass = fn(&Palette, bool) -> CheckboxStyle;

#[derive(Debug)]
pub struct CheckboxStyle {
    pub checked: &'static str,
    pub unchecked: &'static str,
    pub text_color: Rgba,
    pub hovered_color: Option<Rgba>,
}

impl CheckboxStyle {
    pub fn markdown(palette: &Palette, checked: bool) -> Self {
        Self {
            checked: "[X]",
            unchecked: "[ ]",
            ..Self::ascii(palette, checked)
        }
    }

    pub fn ascii(palette: &Palette, checked: bool) -> Self {
        Self {
            checked: "🗹",
            unchecked: "☐",
            text_color: palette.foreground,
            hovered_color: Some(palette.contrast),
        }
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `ui.adhoc()` is called"]
pub struct Checkbox<'a> {
    value: &'a mut bool,
    label: &'a str,
    class: StyleKind<CheckboxClass, CheckboxStyle>,
}

impl<'a> Checkbox<'a> {
    pub const fn class(mut self, class: CheckboxClass) -> Self {
        self.class = StyleKind::deferred(class);
        self
    }

    pub const fn style(mut self, style: CheckboxStyle) -> Self {
        self.class = StyleKind::direct(style);
        self
    }
}

impl<'a> Adhoc<'a> for Checkbox<'a> {
    // TODO make this an opaque response
    type Output = Response<bool>;

    fn show(self, ui: &Ui) -> Self::Output {
        let resp = ui
            .mouse_area(|ui| {
                let style = match self.class {
                    StyleKind::Deferred(style) => (style)(&ui.palette(), *self.value),
                    StyleKind::Direct(style) => style,
                };

                let foreground = if ui.is_hovered() {
                    style.hovered_color.unwrap_or(style.text_color)
                } else {
                    style.text_color
                };

                ui.horizontal(|ui| {
                    let marker = if *self.value {
                        style.checked
                    } else {
                        style.unchecked
                    };
                    ui.label(marker);
                    ui.show(shorthands::label(self.label).style(LabelStyle { foreground }));
                });
            })
            .flatten_left();

        *self.value ^= resp.clicked();
        resp.map(|c| c.clicked())
    }
}

pub fn checkbox<'a>(value: &'a mut bool, label: &'a str) -> Checkbox<'a> {
    Checkbox {
        value,
        label,
        class: StyleKind::deferred(CheckboxStyle::ascii),
    }
}
