use crate::{
    view::{Adhoc, Palette, Response, StyleKind, Ui},
    Border,
};

use super::border::BorderStyle;

pub type CollapsibleClass = fn(&Palette, bool) -> CollapsibleStyle;

#[derive(Debug)]
pub struct CollapsibleStyle {
    pub expanded: &'static str,
    pub collapsed: &'static str,
    pub border: Border,
    pub border_style: BorderStyle,
}

impl CollapsibleStyle {
    pub fn default(palette: &Palette, _expanded: bool) -> Self {
        Self {
            expanded: "▼",
            collapsed: "▶",
            border: Border::THICK,
            border_style: BorderStyle::default(palette, false, false),
        }
    }

    pub fn interactive(palette: &Palette, _expanded: bool) -> Self {
        Self {
            expanded: "▼",
            collapsed: "▶",
            border: Border::THICK,
            border_style: BorderStyle::interactive(palette, true, true),
        }
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `ui.adhoc()` is called"]
pub struct Collapsible<'a, F> {
    state: &'a mut bool,
    title: &'a str,
    show: F,
    class: StyleKind<CollapsibleClass, CollapsibleStyle>,
}

impl<'a, F> Collapsible<'a, F> {
    pub const fn class(mut self, class: CollapsibleClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: CollapsibleStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v, F, R: 'static> Adhoc<'v> for Collapsible<'v, F>
where
    F: FnOnce(&Ui) -> R,
{
    type Output = Response<Option<R>>;

    fn show(self, ui: &Ui) -> Self::Output {
        let style = match self.class {
            StyleKind::Deferred(style) => (style)(&ui.palette(), *self.state),
            StyleKind::Direct(style) => style,
        };

        ui.vertical(|ui| {
            let resp = ui.mouse_area(|ui| {
                if *self.state {
                    let inner = ui.show_children(
                        // TODO make frame take a label so we can change the color of the text, but not the marker
                        super::frame(style.border, format!("{} {}", style.expanded, self.title))
                            .style(style.border_style),
                        self.show,
                    );
                    return Some(inner);
                }

                ui.show_children(
                    super::border(style.border).style(style.border_style),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label(style.collapsed);
                            ui.label(self.title);
                        })
                    },
                );
                None
            });
            *self.state ^= resp.0.clicked();
            resp.flatten_right()
        })
        .into_inner()
        .map(|c| c.map(|c| c.flatten_right().into_inner()))
    }
}

pub fn collapsible<'a, R: 'static>(
    state: &'a mut bool,
    title: &'a str,
    show: impl FnOnce(&Ui) -> R,
) -> Collapsible<'a, impl FnOnce(&Ui) -> R> {
    Collapsible {
        state,
        title,
        show,
        class: StyleKind::Deferred(CollapsibleStyle::default),
    }
}
