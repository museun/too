use crate::{
    renderer::{Attribute, Rgba},
    view::{ApplicableStyle, Builder, Palette, Style, StyleOptions, Ui, View, ViewExt},
    Str,
};

use super::label::LabelStyle;

#[derive(Debug, Copy, Clone)]
pub struct TodoStyle {
    pub selected: Attribute,
    pub text_color: Rgba,
    pub hovered_color: Option<Rgba>,
}

impl Style for TodoStyle {
    type Args = bool;

    fn default(palette: &Palette, _args: StyleOptions<bool>) -> Self {
        Self {
            selected: Attribute::STRIKEOUT | Attribute::FAINT,
            text_color: palette.foreground,
            hovered_color: Some(palette.contrast),
        }
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct TodoValue<'a> {
    value: &'a mut bool,
    label: Str,
    style: ApplicableStyle<TodoStyle>,
}

impl<'v> Builder<'v> for TodoValue<'v> {
    type View = TodoValueView;
    type Style = TodoStyle;

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
    }
}

#[derive(Debug)]
pub struct TodoValueView {
    label: Str,
    style: ApplicableStyle<TodoStyle>,
}

impl View for TodoValueView {
    type Args<'v> = TodoValue<'v>;
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

                let attr = if *args.value {
                    style.selected
                } else {
                    Attribute::RESET
                };

                ui.horizontal(|ui| {
                    ui.show(
                        super::label(&self.label)
                            .style(LabelStyle { foreground })
                            .attribute(attr),
                    );
                });
            })
            .flatten_left();

        *args.value ^= resp.clicked();
        resp.clicked()
    }
}

pub fn todo_value(value: &mut bool, label: impl Into<Str>) -> TodoValue<'_> {
    TodoValue {
        value,
        label: label.into(),
        style: ApplicableStyle::default(),
    }
}
