use std::borrow::Cow;

use crate::{
    view::{Adhoc, Palette, Response, Ui},
    Key,
};

pub trait DropDownItem: PartialEq {
    fn short(&self) -> Cow<'static, str>;
    fn long(&self) -> Cow<'static, str> {
        self.short()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DropDownResponse {
    selected: Option<usize>,
}

impl DropDownResponse {
    pub fn resolve<'a, V: DropDownItem>(&self, values: &'a [V]) -> Option<&'a V> {
        self.selected.and_then(|index| values.get(index))
    }

    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }
}

pub struct DropDownState {
    pub expanded: bool,
    pub index: usize,
    pub hovered: Option<usize>,
}

impl Default for DropDownState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl DropDownState {
    pub const fn new(index: usize) -> Self {
        Self {
            expanded: false,
            index,
            hovered: None,
        }
    }

    pub fn from_items<V: DropDownItem>(selected: &V, items: &[V]) -> Self {
        Self::new(items.iter().position(|c| c == selected).unwrap_or(0))
    }
}

pub struct DropDown<'a, V> {
    values: &'a [V],
    state: &'a mut DropDownState,
}

impl<'v, V> Adhoc<'v> for DropDown<'v, V>
where
    V: DropDownItem,
{
    type Output = Response<DropDownResponse>;

    fn show(self, ui: &Ui) -> Self::Output {
        let mut selected = None;

        let bg = Palette::current().surface;
        let resp = ui.background(bg, |ui| {
            ui.margin((1, 0), |ui| {
                let resp = ui.mouse_area(|ui| {
                    let resp = ui.key_area(|ui| {
                        ui.vertical(|ui| {
                            Self::draw_drop_list(ui, &mut selected, self.values, self.state)
                        })
                    });
                    ui.set_focus(resp.id());
                    let resp = resp.flatten_left();

                    if resp.key_pressed(Key::Up) {
                        self.state.index = self
                            .state
                            .index
                            .checked_sub(1)
                            .unwrap_or(self.values.len().saturating_sub(1));
                        selected = Some(self.state.index);
                    }
                    if resp.key_pressed(Key::Down) {
                        self.state.index = (self.state.index + 1) % self.values.len();
                        selected = Some(self.state.index);
                    }

                    if resp.key_pressed(Key::Right) {
                        self.state.expanded = true;
                    }

                    if resp.key_pressed(Key::Escape)
                        || resp.key_pressed(Key::Left)
                        || resp.key_pressed(Key::Enter)
                    {
                        self.state.expanded = false;
                    }
                });
                if resp.flatten_left().clicked() {
                    self.state.expanded = !self.state.expanded
                }
            })
        });

        resp.map(|_| DropDownResponse { selected })
    }
}

impl<'a, V: DropDownItem> DropDown<'a, V> {
    fn draw_drop_list(
        ui: &Ui,
        selected: &mut Option<usize>,
        values: &[V],
        state: &mut DropDownState,
    ) {
        if !state.expanded {
            ui.horizontal(|ui| {
                ui.label('▶');
                ui.label(&*values[state.index].short())
            });
            return;
        }

        let (primary, secondary) = {
            let p = Palette::current();
            (p.primary, p.secondary)
        };

        for (i, item) in values.iter().enumerate() {
            let resp = ui.mouse_area(|ui| {
                ui.horizontal(|ui| {
                    ui.label(item.short());

                    if Some(i) == state.hovered {
                        ui.background(primary, |ui| ui.label(item.long()));
                    } else if i == state.index {
                        ui.background(secondary, |ui| ui.label(item.long()));
                    } else {
                        ui.label(item.long());
                    };
                });
            });

            let resp = resp.flatten_left();
            if resp.hovered() {
                state.hovered = Some(i)
            }

            if resp.clicked() {
                state.index = i;
                *selected = Some(i);
                state.hovered = None
            }
        }
    }
}

pub fn drop_down<'a, V: DropDownItem>(
    values: &'a [V],
    state: &'a mut DropDownState,
) -> DropDown<'a, V> {
    DropDown { values, state }
}
