use crate::{
    renderer::Rgba,
    view::{ApplicableStyle, Builder, Palette, Style, StyleOptions, Ui, View, ViewExt},
    Str,
};

use super::label::{label, LabelStyle};

// TODO make this simpler
#[derive(Debug, Copy, Clone)]
pub struct SelectedStyle {
    pub text_color: Rgba,

    pub background: Rgba,
    pub selected_background: Rgba,

    pub hovered_text: Option<Rgba>,
    pub hovered_background: Option<Rgba>,
}

impl Style for SelectedStyle {
    type Args = bool;
    fn default(palette: &Palette, _args: StyleOptions<bool>) -> Self {
        Self {
            text_color: palette.foreground,
            background: palette.outline,
            selected_background: palette.primary,
            hovered_text: None,
            hovered_background: None,
        }
    }
}

impl SelectedStyle {
    pub fn hovered(palette: &Palette, args: StyleOptions<bool>) -> Self {
        Self {
            hovered_text: Some(palette.surface),
            hovered_background: Some(palette.secondary),
            ..Self::default(palette, args)
        }
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Selected<'a> {
    value: &'a mut bool,
    label: Str,
    style: ApplicableStyle<SelectedStyle>,
}

impl<'v> Builder<'v> for Selected<'v> {
    type View = SelectedView;
    type Style = SelectedStyle;

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
    }
}

#[derive(Debug)]
pub struct SelectedView {
    label: Str,
    style: ApplicableStyle<SelectedStyle>,
}

impl View for SelectedView {
    type Args<'v> = Selected<'v>;
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

                let hovered = ui.is_hovered();
                let fill = match (hovered, *args.value) {
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
                    ui.show(label(&self.label).style(LabelStyle { foreground: text }))
                });
            })
            .flatten_left();

        *args.value ^= resp.clicked();
        resp.clicked()
    }
}

pub fn selected(value: &mut bool, label: impl Into<Str>) -> Selected<'_> {
    Selected {
        value,
        label: label.into(),
        style: ApplicableStyle::default(),
    }
}
