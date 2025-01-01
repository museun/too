use crate::{
    renderer::Rgba,
    view::{ApplicableStyle, Builder, Palette, Style, Ui, View},
    Str,
};

use super::label::{label, LabelStyle};

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
    fn default(palette: &Palette, _args: Self::Args) -> Self {
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
    pub fn hovered(palette: &Palette, selected: bool) -> Self {
        Self {
            hovered_text: Some(palette.surface),
            hovered_background: Some(palette.secondary),
            ..Self::default(palette, selected)
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
    fn style(mut self, style: Self::Style) -> Self {
        self.style = ApplicableStyle::value(style);
        self
    }

    fn class(mut self, class: impl Fn(&Palette, bool) -> Self::Style + 'static) -> Self {
        self.style = ApplicableStyle::new(class);
        self
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
        let resp = ui
            .mouse_area(|ui| {
                let style = self.style.apply(&ui.palette(), *args.value);

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
