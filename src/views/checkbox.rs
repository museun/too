use std::borrow::Cow;

use crate::{
    renderer::Rgba,
    view::{ApplicableStyle, Builder, Palette, Style, StyleOptions, Ui, View, ViewExt},
    Str,
};

use super::label::LabelStyle;

#[derive(Debug, Clone)]
pub struct CheckboxStyle {
    pub checked: Cow<'static, str>,
    pub unchecked: Cow<'static, str>,
    pub text_color: Rgba,
    pub hovered_color: Option<Rgba>,
}

impl Style for CheckboxStyle {
    type Args = bool;

    fn default(palette: &Palette, _options: StyleOptions<bool>) -> Self {
        Self {
            checked: Cow::Borrowed("🗹"),
            unchecked: Cow::Borrowed("☐"),
            text_color: palette.foreground,
            hovered_color: Some(palette.contrast),
        }
    }
}

impl CheckboxStyle {
    pub fn markdown(palette: &Palette, options: StyleOptions<bool>) -> Self {
        Self {
            checked: Cow::Borrowed("[X]"),
            unchecked: Cow::Borrowed("[ ]"),
            ..Self::default(palette, options)
        }
    }

    pub fn ascii(palette: &Palette, options: StyleOptions<bool>) -> Self {
        Self::default(palette, options)
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

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
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
        self.label = args.label;
        self.style = args.style;

        let resp = ui
            .mouse_area(|ui| {
                let style = self.style.apply(ui, |s| s.with_args(*args.value));

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
