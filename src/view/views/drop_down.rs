use std::borrow::Cow;

use crate::{
    view::{Response, Ui},
    Key,
};

use super::key_area;

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
    const fn new(index: usize) -> Self {
        Self {
            expanded: false,
            index,
            hovered: None,
        }
    }

    fn from_items<V: DropDownItem>(selected: &V, items: &[V]) -> Self {
        Self::new(items.iter().position(|c| c == selected).unwrap_or(0))
    }
}

pub fn drop_down<V: DropDownItem>(
    ui: &Ui,
    values: &[V],
    state: &mut DropDownState,
) -> Response<DropDownResponse> {
    let mut selected = None;

    let resp = ui.background(ui.theme.surface, |ui| {
        ui.margin((1, 0), |ui| {
            let resp = ui.mouse_area(|ui| {
                let resp = ui.show_children(key_area(), |ui| {
                    ui.vertical(|ui| draw_drop_list(ui, &mut selected, values, state))
                });
                ui.set_focus(resp.id());
                let resp = resp.flatten_left();

                if resp.key_pressed(Key::Up) {
                    state.index = state
                        .index
                        .checked_sub(1)
                        .unwrap_or(values.len().saturating_sub(1));
                    selected = Some(state.index);
                }
                if resp.key_pressed(Key::Down) {
                    state.index = (state.index + 1) % values.len();
                    selected = Some(state.index);
                }

                if resp.key_pressed(Key::Right) {
                    state.expanded = true;
                }

                if resp.key_pressed(Key::Escape)
                    || resp.key_pressed(Key::Left)
                    || resp.key_pressed(Key::Enter)
                {
                    state.expanded = false;
                }
            });
            if resp.flatten_left().clicked() {
                state.expanded = !state.expanded
            }
        })
    });

    resp.map(|_| DropDownResponse { selected })
}

fn draw_drop_list<V: DropDownItem>(
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

    for (i, item) in values.iter().enumerate() {
        let resp = ui.mouse_area(|ui| {
            ui.horizontal(|ui| {
                ui.label(item.short());
                if Some(i) == state.hovered {
                    ui.background(ui.theme.primary, |ui| ui.label(item.long()));
                } else if i == state.index {
                    ui.background(ui.theme.secondary, |ui| ui.label(item.long()));
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
