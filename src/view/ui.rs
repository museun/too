use std::cell::{Ref, RefCell};

use crate::{
    layout::Align2,
    math::{Pos2, Rect},
    view::views::shorthands::todo_value,
    Border, Rgba,
};
use compact_str::ToCompactString;

use super::{
    geom::{Flex, Margin},
    input::InputState,
    state::{LayoutNodes, ViewNodes},
    view::Adhoc,
    views::{
        self, button, checkbox::checkbox, radio::radio, selected::selected, shorthands,
        TextInputResponse, ToggleResponse,
    },
    Builder, Palette, Response, State, View, ViewId,
};

pub struct Ui<'a> {
    pub(in crate::view) nodes: &'a ViewNodes,
    pub(in crate::view) layout: &'a LayoutNodes,
    pub(in crate::view) input: &'a InputState,
    pub(in crate::view) palette: &'a RefCell<Palette>,
}

impl<'a> Ui<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            nodes: &state.nodes,
            layout: &state.layout,
            input: &state.input,
            palette: &state.palette,
        }
    }

    pub fn adhoc<'v, A>(&self, view: A) -> A::Output
    where
        A: Adhoc<'v>,
    {
        view.show(self)
    }

    pub fn show<'v, B>(&self, args: B) -> Response<<B::View as View>::Response>
    where
        B: Builder<'v>,
    {
        self.show_children(args, |_| {}).flatten_left()
    }

    pub fn show_children<'v, B, R>(
        &self,
        args: B,
        show: impl FnOnce(&Self) -> R,
    ) -> Response<(<B::View as View>::Response, R)>
    where
        B: Builder<'v>,
        R: 'static,
    {
        let (id, resp) = self.nodes.begin_view::<B::View>(args, self);
        let inner = show(self);
        self.nodes.end_view(id);
        Response::new(id, (resp, inner))
    }
}

impl<'a> Ui<'a> {
    pub fn current_available_rect(&self) -> Rect {
        let parent = self.nodes.parent();
        self.layout.get(parent).map(|c| c.rect).unwrap_or_default()
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn cursor_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    pub fn palette(&self) -> Ref<'_, Palette> {
        self.palette.borrow()
    }

    pub fn set_palette(&self, palette: Palette) {
        *self.palette.borrow_mut() = palette
    }

    pub fn set_focus(&self, id: impl Into<Option<ViewId>>) {
        self.input.set_focus(id.into());
    }

    pub fn scope<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Scope, show).flatten_right()
    }

    pub fn layer<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Layer, show).flatten_right()
    }

    pub fn clip<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Clip, show).flatten_right()
    }

    pub fn float<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Float, show).flatten_right()
    }
}

impl<'a> Ui<'a> {
    // pub fn set_debug_mode(&self, mode: DebugMode) {
    //     self.debug.mode.set(mode);
    // }

    pub fn debug(msg: impl ToCompactString) {
        super::state::debug(msg);
    }
}

impl<'a> Ui<'a> {
    pub fn is_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.current())
    }

    pub fn is_parent_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.parent())
    }
}

impl<'a> Ui<'a> {
    pub fn center<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.aligned(Align2::CENTER_CENTER, show)
    }

    pub fn aligned<R>(&self, align: Align2, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::aligned(align), show)
            .flatten_right()
    }

    pub fn margin<R>(&self, margin: impl Into<Margin>, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Margin::new(margin), show)
            .flatten_right()
    }

    pub fn background<R>(&self, bg: impl Into<Rgba>, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Background::new(bg), show)
            .flatten_right()
    }

    pub fn offset<R>(&self, offset: impl Into<Pos2>, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Offset::new(offset), show)
            .flatten_right()
    }

    pub fn constrain<R>(
        &self,
        constrain: views::Constrain,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(constrain, show).flatten_right()
    }

    pub fn unconstrained<R>(
        &self,
        unconstrained: views::Unconstrained,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(unconstrained, show).flatten_right()
    }

    pub fn draggable<R>(
        &self,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(Option<views::Dragging>, R)>
    where
        R: 'static,
    {
        self.mouse_area(show).map(|(m, r)| (m.dragged(), r))
    }

    pub fn mouse_area<R>(
        &self,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(views::MouseAreaResponse, R)>
    where
        R: 'static,
    {
        self.show_children(shorthands::mouse_area(), show)
    }

    pub fn key_area<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<(views::KeyAreaResponse, R)>
    where
        R: 'static,
    {
        self.show_children(shorthands::key_area(), show)
    }

    pub fn progress(&self, value: f32) -> Response {
        self.show(shorthands::progress(value))
    }

    pub fn text_input(&self, focus: bool) -> Response<TextInputResponse> {
        let resp = self.show(shorthands::text_input());
        if focus {
            self.set_focus(resp.id());
        }
        resp
    }

    pub fn slider(&self, value: &mut f32) -> Response {
        self.show(shorthands::slider(value))
    }

    pub fn toggle_switch(&self, value: &mut bool) -> Response<ToggleResponse> {
        self.show(shorthands::toggle_switch(value))
    }

    pub fn button(&self, label: &str) -> Response<button::Response> {
        self.show(shorthands::button(label).margin((1, 0)))
    }

    pub fn checkbox(&self, value: &mut bool, label: &str) -> Response<bool> {
        self.adhoc(checkbox(value, label))
    }

    pub fn todo_value(&self, value: &mut bool, label: &str) -> Response<bool> {
        self.adhoc(todo_value(value, label))
    }

    pub fn selected(&self, value: &mut bool, label: &str) -> Response<bool> {
        self.adhoc(selected(value, label))
    }

    pub fn radio<V>(&self, value: V, existing: &mut V, label: &str) -> Response<bool>
    where
        V: PartialEq,
    {
        self.adhoc(radio(value, existing, label))
    }

    pub fn label(&self, data: impl ToCompactString) -> Response {
        self.show(shorthands::label(data))
    }

    pub fn expand<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Flex::new(Flex::Tight(1.0)), show)
            .flatten_right()
    }

    pub fn flex<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Flex::new(Flex::Loose(1.0)), show)
            .flatten_right()
    }

    pub fn vertical_wrap<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::vertical_wrap(), show)
            .flatten_right()
    }

    pub fn horizontal_wrap<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::horizontal_wrap().row_gap(1), show)
            .flatten_right()
    }

    pub fn expander(&self) -> Response {
        self.show(views::Expander)
    }

    // TODO make this work better
    // pub fn separator(&self) -> Response {
    //     self.show(views::Separator)
    // }

    pub fn vertical<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::list().vertical(), show)
            .flatten_right()
    }

    pub fn horizontal<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::list().horizontal().gap(1), show)
            .flatten_right()
    }

    pub fn border<R>(&self, border: Border, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::border(border), show)
            .flatten_right()
    }

    pub fn frame<R>(
        &self,
        border: Border,
        title: impl ToCompactString,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(shorthands::frame(border, title), show)
            .flatten_right()
    }

    pub fn collapsible<R: 'static>(
        &self,
        state: &mut bool,
        title: &str,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<Option<R>> {
        self.adhoc(shorthands::collapsible(state, title, show))
    }
}
