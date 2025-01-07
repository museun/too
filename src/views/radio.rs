use std::marker::PhantomData;

use crate::{
    renderer::Rgba,
    view::{ApplicableStyle, Builder, Palette, Style, StyleOptions, Ui, View, ViewExt},
    Str,
};

use super::label::{label, LabelStyle};

// TODO make this simpler
#[derive(Debug, Copy, Clone)]
pub struct RadioStyle {
    pub selected: Option<&'static str>,
    pub unselected: Option<&'static str>,

    pub text_color: Rgba,

    pub background: Rgba,
    pub selected_background: Rgba,

    pub hovered_text: Option<Rgba>,
    pub hovered_background: Option<Rgba>,
}

impl Style for RadioStyle {
    type Args = bool;
    fn default(palette: &Palette, _args: StyleOptions<bool>) -> Self {
        Self {
            selected: None,
            unselected: None,
            text_color: palette.foreground,
            background: palette.surface,
            selected_background: palette.primary,
            hovered_text: None,
            hovered_background: None,
        }
    }
}

impl RadioStyle {
    pub fn hovered(palette: &Palette, args: StyleOptions<bool>) -> Self {
        Self {
            hovered_text: Some(palette.surface),
            hovered_background: Some(palette.secondary),
            ..Self::default(palette, args)
        }
    }
}

pub struct Radio<'a, V> {
    value: V,
    existing: &'a mut V,
    label: Str,
    style: ApplicableStyle<RadioStyle>,
}

impl<'v, V: PartialEq + 'static> Builder<'v> for Radio<'v, V> {
    type View = RadioView<V>;
    type Style = RadioStyle;

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
    }
}

pub struct RadioView<V>
where
    V: PartialEq + 'static,
{
    label: Str,
    style: ApplicableStyle<RadioStyle>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: PartialEq> std::fmt::Debug for RadioView<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadioView")
            .field("label", &self.label)
            .field("class", &self.style)
            .finish()
    }
}

impl<V> View for RadioView<V>
where
    V: PartialEq + 'static,
{
    type Args<'v> = Radio<'v, V>;
    type Response = bool;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            label: args.label,
            style: args.style,
            _marker: PhantomData,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        let resp = ui
            .mouse_area(|ui| {
                let selected = args.value == *args.existing;
                let style = self.style.apply(ui, |s| s.with_args(selected));

                let hovered = ui.is_hovered();
                let fill = match (hovered, selected) {
                    (false, true) => style.selected_background,
                    (false, false) => style.background,
                    (true, true) => style
                        .hovered_background
                        .unwrap_or(style.selected_background),
                    (true, false) => style.hovered_background.unwrap_or(style.background),
                };

                let foreground = if hovered {
                    style.hovered_text.unwrap_or(style.text_color)
                } else {
                    style.text_color
                };

                ui.background(fill, |ui| {
                    let left = if selected {
                        style.selected
                    } else {
                        style.unselected
                    };

                    ui.horizontal(|ui| {
                        if let Some(left) = left {
                            ui.label(left);
                        }
                        ui.show(label(&self.label).style(LabelStyle { foreground }));
                    });
                })
            })
            .flatten_left();

        let clicked = resp.clicked();
        if clicked {
            *args.existing = args.value;
        }
        clicked
    }
}

pub fn radio<V>(value: V, existing: &mut V, label: impl Into<Str>) -> Radio<'_, V>
where
    V: PartialEq,
{
    Radio {
        value,
        existing,
        label: label.into(),
        style: ApplicableStyle::default(),
    }
}
