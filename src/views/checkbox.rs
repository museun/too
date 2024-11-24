use crate::{
    renderer::Rgba,
    view::{Builder, Palette, StyleKind, Ui, View},
    Str,
};

use super::label::LabelStyle;

pub type CheckboxClass = fn(&Palette, bool) -> CheckboxStyle;

#[derive(Debug, Copy, Clone)]
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

    pub fn ascii(palette: &Palette, _checked: bool) -> Self {
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
    label: Str,
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

impl<'v> Builder<'v> for Checkbox<'v> {
    type View = CheckboxView;
}

#[derive(Debug)]
pub struct CheckboxView {
    label: Str,
    class: StyleKind<CheckboxClass, CheckboxStyle>,
}

impl View for CheckboxView {
    type Args<'v> = Checkbox<'v>;
    type Response = bool;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            label: args.label,
            class: args.class,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        let resp = ui
            .mouse_area(|ui| {
                let style = match self.class {
                    StyleKind::Deferred(style) => (style)(&ui.palette(), *args.value),
                    StyleKind::Direct(style) => style,
                };

                let foreground = if ui.is_hovered() {
                    style.hovered_color.unwrap_or(style.text_color)
                } else {
                    style.text_color
                };

                ui.horizontal(|ui| {
                    let marker = if *args.value {
                        style.checked
                    } else {
                        style.unchecked
                    };
                    ui.label(marker);
                    ui.show(super::label(&self.label).style(LabelStyle { foreground }));
                });
            })
            .flatten_left();

        *args.value ^= resp.clicked();
        resp.clicked()
    }
}

pub fn checkbox(value: &mut bool, label: impl Into<Str>) -> Checkbox<'_> {
    Checkbox {
        value,
        label: label.into(),
        class: StyleKind::deferred(CheckboxStyle::ascii),
    }
}
