use crate::{
    renderer::Rgba,
    view::{ApplicableStyle, Builder, Palette, Style, Ui, View},
    Str,
};

use super::label::LabelStyle;

#[derive(Debug, Copy, Clone)]
pub struct CheckboxStyle {
    pub checked: &'static str,
    pub unchecked: &'static str,
    pub text_color: Rgba,
    pub hovered_color: Option<Rgba>,
}

impl Style for CheckboxStyle {
    type Args = bool;

    fn default(palette: &Palette, _args: Self::Args) -> Self {
        Self {
            checked: "🗹",
            unchecked: "☐",
            text_color: palette.foreground,
            hovered_color: Some(palette.contrast),
        }
    }
}

impl CheckboxStyle {
    pub fn markdown(palette: &Palette, checked: bool) -> Self {
        Self {
            checked: "[X]",
            unchecked: "[ ]",
            ..Self::default(palette, checked)
        }
    }

    pub fn ascii(palette: &Palette, checked: bool) -> Self {
        Self::default(palette, checked)
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Checkbox<'a> {
    value: &'a mut bool,
    label: Str,
    style: ApplicableStyle<CheckboxStyle>,
}

impl<'v> Builder<'v> for Checkbox<'v> {
    type View = CheckboxView;
    type Style = CheckboxStyle;

    fn style(mut self, style: Self::Style) -> Self {
        self.style = ApplicableStyle::value(style);
        self
    }

    fn class(mut self, style: impl Fn(&Palette, bool) -> Self::Style + 'static) -> Self {
        self.style = ApplicableStyle::new(style);
        self
    }
}

#[derive(Debug)]
pub struct CheckboxView {
    label: Str,
    style: ApplicableStyle<CheckboxStyle>,
}

impl View for CheckboxView {
    type Args<'v> = Checkbox<'v>;
    type Response = bool;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            label: args.label,
            style: args.style,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        let resp = ui
            .mouse_area(|ui| {
                let style = self.style.apply(&ui.palette(), *args.value);

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
        style: ApplicableStyle::default(),
    }
}
