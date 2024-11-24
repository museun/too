use std::marker::PhantomData;

use crate::{
    renderer::Rgba,
    view::{Builder, Palette, StyleKind, Ui, View},
    Str,
};

use super::label::{label, LabelStyle};

pub type RadioClass = fn(&Palette, bool) -> RadioStyle;

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

impl RadioStyle {
    pub fn default(palette: &Palette, _selected: bool) -> Self {
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

    pub fn hovered(palette: &Palette, selected: bool) -> Self {
        Self {
            hovered_text: Some(palette.surface),
            hovered_background: Some(palette.secondary),
            ..Self::default(palette, selected)
        }
    }
}

pub struct Radio<'a, V> {
    value: V,
    existing: &'a mut V,
    label: Str,
    class: StyleKind<RadioClass, RadioStyle>,
}

impl<'v, V: PartialEq + 'static> Builder<'v> for Radio<'v, V> {
    type View = RadioView<V>;
}

pub struct RadioView<V>
where
    V: PartialEq + 'static,
{
    label: Str,
    class: StyleKind<RadioClass, RadioStyle>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: PartialEq> std::fmt::Debug for RadioView<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadioView")
            .field("label", &self.label)
            .field("class", &self.class)
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
            class: args.class,
            _marker: PhantomData,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        let resp = ui
            .mouse_area(|ui| {
                let style = match self.class {
                    StyleKind::Deferred(style) => {
                        (style)(&ui.palette(), args.value == *args.existing)
                    }
                    StyleKind::Direct(style) => style,
                };

                let hovered = ui.is_hovered();
                let fill = match (hovered, args.value == *args.existing) {
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
                    ui.show(label(&self.label).style(LabelStyle { foreground }))
                });
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
        class: StyleKind::deferred(RadioStyle::default),
    }
}
