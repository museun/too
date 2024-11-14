use std::cell::{Ref, RefCell};

use crate::{
    layout::{Align2, Flex},
    math::{Margin, Pos2, Rect, Size},
    views::{self, Constrain},
    Border, Rgba, Str,
};

use super::{
    input::InputState, internal_views, Adhoc, Builder, LayoutNodes, Palette, Response, State, View,
    ViewId, ViewNodes,
};

pub struct Ui<'a> {
    pub(in crate::view) nodes: &'a ViewNodes,
    pub(in crate::view) layout: &'a LayoutNodes,
    pub(in crate::view) input: &'a InputState,
    pub(in crate::view) palette: &'a RefCell<Palette>,
    pub(in crate::view) client_rect: Rect,
    pub(in crate::view) frame_count: u64,
    pub(in crate::view) dt: f32,
}

impl<'a> Ui<'a> {
    pub(super) fn new(state: &'a mut State, client_rect: Rect) -> Self {
        Self {
            nodes: &state.nodes,
            layout: &state.layout,
            input: &state.input,
            palette: &state.palette,
            client_rect,
            frame_count: state.frame_count,
            dt: state.dt,
        }
    }
}

impl<'a> Ui<'a> {
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
    pub fn client_rect(&self) -> Rect {
        self.client_rect
    }

    pub fn current_available_rect(&self) -> Rect {
        let parent = self.nodes.parent();
        self.layout.get(parent).map(|c| c.rect).unwrap_or_default()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn dt(&self) -> f32 {
        self.dt
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
        self.show_children(internal_views::Scope, show)
            .flatten_right()
    }

    pub fn layer<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(internal_views::Layer, show)
            .flatten_right()
    }

    pub fn clip<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(internal_views::Clip, show)
            .flatten_right()
    }

    pub fn float<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(internal_views::Float, show)
            .flatten_right()
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
        self.show_children(views::aligned(align), show)
            .flatten_right()
    }

    pub fn margin<R>(&self, margin: impl Into<Margin>, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::MarginView::new(margin), show)
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

    pub fn exact_size<R>(&self, size: impl Into<Size>, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.constrain(Constrain::exact_size(size), show)
    }

    pub fn exact_height<R>(&self, height: i32, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.constrain(Constrain::exact_height(height), show)
    }

    pub fn exact_width<R>(&self, width: i32, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.constrain(Constrain::exact_width(width), show)
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
    ) -> Response<(Option<views::DraggingResponse>, R)>
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
        self.show_children(views::mouse_area(), show)
    }

    pub fn key_area<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<(views::KeyAreaResponse, R)>
    where
        R: 'static,
    {
        self.show_children(views::key_area(), show)
    }

    pub fn progress(&self, value: f32) -> Response {
        self.show(views::progress(value))
    }

    pub fn text_input(&self, focus: bool) -> Response<views::TextInputResponse> {
        let resp = self.show(views::text_input());
        if focus {
            self.set_focus(resp.id());
        }
        resp
    }

    pub fn slider(&self, value: &mut f32) -> Response {
        self.show(views::slider(value))
    }

    pub fn toggle_switch(&self, value: &mut bool) -> Response<views::ToggleResponse> {
        self.show(views::toggle_switch(value))
    }

    pub fn button(&self, label: impl Into<Str>) -> Response<views::ButtonResponse> {
        self.show(views::button(label).margin((1, 0)))
    }

    pub fn checkbox(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.adhoc(views::checkbox(value, label))
    }

    pub fn todo_value(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.adhoc(views::todo_value(value, label))
    }

    pub fn selected(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.adhoc(views::selected(value, label))
    }

    pub fn radio<V>(&self, value: V, existing: &mut V, label: impl Into<Str>) -> Response<bool>
    where
        V: PartialEq,
    {
        self.adhoc(views::radio(value, existing, label))
    }

    pub fn label(&self, data: impl Into<Str>) -> Response {
        self.show(views::label(data))
    }

    pub fn expand<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::FlexView::new(Flex::Tight(1.0)), show)
            .flatten_right()
    }

    pub fn flex<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::FlexView::new(Flex::Loose(1.0)), show)
            .flatten_right()
    }

    pub fn vertical_wrap<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::vertical_wrap(), show)
            .flatten_right()
    }

    pub fn horizontal_wrap<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::horizontal_wrap().row_gap(1), show)
            .flatten_right()
    }

    pub fn expand_space(&self) -> Response {
        self.show(views::Fill::all_space())
    }

    pub fn expand_axis(&self) -> Response {
        self.show(views::expander())
    }

    pub fn separator(&self) -> Response {
        self.show(views::separator())
    }

    pub fn vertical<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::list().vertical(), show)
            .flatten_right()
    }

    pub fn horizontal<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::list().horizontal().gap(1), show)
            .flatten_right()
    }

    pub fn border<R>(&self, border: Border, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::border(border), show)
            .flatten_right()
    }

    pub fn frame<R>(
        &self,
        border: Border,
        title: impl Into<Str>,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::frame(border, title), show)
            .flatten_right()
    }

    // pub fn collapsible<R: 'static>(
    //     &self,
    //     state: &mut bool,
    //     title: &str,
    //     show: impl FnOnce(&Ui) -> R,
    // ) -> Response<Option<R>> {
    //     self.adhoc(views::collapsible(state, title, show))
    // }
}
