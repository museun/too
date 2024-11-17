use crate::{
    renderer::{Attribute, Rgba},
    view::{Adhoc, Palette, Response, StyleKind},
    Str,
};

use super::label::LabelStyle;

pub type TodoClass = fn(&Palette, bool) -> TodoStyle;

#[derive(Debug)]
pub struct TodoStyle {
    pub selected: Attribute,
    pub text_color: Rgba,
    pub hovered_color: Option<Rgba>,
}

impl TodoStyle {
    pub fn default(palette: &Palette, _selected: bool) -> Self {
        Self {
            selected: Attribute::STRIKEOUT | Attribute::FAINT,
            text_color: palette.foreground,
            hovered_color: Some(palette.contrast),
        }
    }
}

#[must_use = "a view does nothing unless `ui.adhoc()` is called"]
pub struct TodoValue<'a> {
    value: &'a mut bool,
    label: Str,
    class: StyleKind<TodoClass, TodoStyle>,
}

impl<'a> TodoValue<'a> {
    pub const fn class(mut self, class: TodoClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: TodoStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'a> Adhoc<'a> for TodoValue<'a> {
    type Output = Response<bool>;

    fn show(self, ui: &crate::view::Ui) -> Self::Output {
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

                let attr = if *self.value {
                    style.selected
                } else {
                    Attribute::RESET
                };

                ui.horizontal(|ui| {
                    ui.show(
                        super::label(self.label)
                            .style(LabelStyle { foreground })
                            .attribute(attr),
                    );
                });
            })
            .flatten_left();

        *self.value ^= resp.clicked();
        resp.map(|c| c.clicked())
    }
}

pub fn todo_value(value: &mut bool, label: impl Into<Str>) -> TodoValue<'_> {
    TodoValue {
        value,
        label: label.into(),
        class: StyleKind::Deferred(TodoStyle::default),
    }
}
