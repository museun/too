use crate::{
    backend::Keybind,
    layout::{Align2, Flex},
    lock::{Lock, Ref, RefMapped},
    math::{Margin, Pos2, Rect, Size, Vec2},
    renderer::{Border, Rgba},
    views::{self, Constrain},
    Str,
};

use super::{
    filter::{Filter, Filterable},
    input::InputState,
    internal_views, Builder, LayoutNodes, Palette, Response, State, View, ViewId, ViewNodes,
};

impl<'a> Filterable for Ui<'a> {
    fn filter(&self) -> Filter<'_> {
        Filter::new(self.nodes, self.layout, self.input)
    }
}

pub struct Ui<'a> {
    nodes: &'a ViewNodes,
    layout: &'a LayoutNodes,
    input: &'a InputState,
    palette: &'a Lock<Palette>,

    client_rect: Rect,
    size_changed: Option<Vec2>,
    frame_count: u64,
    dt: f32,
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
            size_changed: state.size_changed,
        }
    }
}

impl<'a> Ui<'a> {
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
    pub fn filter(&self) -> Filter<'_> {
        <Self as Filterable>::filter(self)
    }
}

impl<'a> Ui<'a> {
    pub fn root(&self) -> ViewId {
        self.nodes.root()
    }

    pub fn rect_of(&self, id: ViewId) -> Option<Rect> {
        self.layout.rect(id)
    }

    pub fn lookup<'v, T, R>(
        &self,
        id: ViewId,
        found: impl FnMut(&<T as Builder<'v>>::View) -> R,
    ) -> Option<R>
    where
        T: Builder<'v>,
        R: 'static,
    {
        self.filter().lookup::<T, R>(id, found)
    }

    pub fn client_rect(&self) -> Rect {
        self.client_rect
    }

    pub fn current_available_rect(&self) -> Rect {
        let parent = self.nodes.parent();
        // TODO don't unwrap_or_default here, just return the Option (first frame would always be 'None')
        self.layout.get(parent).map(|c| c.rect).unwrap_or_default()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn size_changed(&self) -> Option<Vec2> {
        self.size_changed
    }

    pub fn palette(&self) -> Ref<'_, Palette> {
        self.palette.borrow()
    }

    pub fn set_palette(&self, palette: Palette) {
        *self.palette.borrow_mut() = palette
    }
}

impl<'a> Ui<'a> {
    pub fn key_pressed(&self, keybind: impl Into<Keybind>) -> bool {
        let prev = match self.input.key_press() {
            Some(prev) => prev,
            None => return false,
        };

        let keybind = keybind.into();
        // TODO normalize this. 'a' and 'A' should be separate w/ modifiers here
        if keybind == prev {
            return true;
        }
        false
    }

    pub fn cursor_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    pub fn is_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.current())
    }

    pub fn is_parent_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.parent())
    }

    pub fn is_focused(&self) -> bool {
        self.input.is_focused(self.nodes.current())
    }

    pub fn is_parent_focused(&self) -> bool {
        self.input.is_focused(self.nodes.parent())
    }

    pub fn set_focus(&self, id: impl Into<Option<ViewId>>) {
        self.input.set_focus(id.into());
    }
}

impl<'a> Ui<'a> {
    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn children(&self) -> RefMapped<'_, [ViewId]> {
        self.children_for(self.current()).unwrap()
    }

    pub fn children_for(&self, id: ViewId) -> Option<RefMapped<'_, [ViewId]>> {
        let inner = self.nodes.get(id)?;
        Some(RefMapped::map(inner, |node| &*node.children))
    }
}

impl<'a> Ui<'a> {
    // TODO this is a bad name, this means input layer not render layer
    pub fn layer<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(internal_views::Layer, show)
            .flatten_right()
    }

    pub fn new_layer<R>(&self, layer: super::Layer, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(internal_views::Float(layer), show)
            .flatten_right()
    }

    pub fn float<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.new_layer(super::Layer::Top, show)
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
        self.show_children(views::Padding::new(margin), show)
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

    pub fn toggle<R>(
        &self,
        state: bool,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(views::ToggleResponse, R)>
    where
        R: 'static,
    {
        self.show_children(views::toggle(state), show)
    }

    pub fn toggle_switch(&self, value: &mut bool) -> Response<views::ToggleResponse> {
        self.show(views::toggle_switch(value))
    }

    pub fn button(&self, label: impl Into<Str>) -> Response<views::ButtonResponse> {
        self.show(views::button(label).margin((1, 0)))
    }

    pub fn checkbox(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.show(views::checkbox(value, label))
    }

    pub fn todo_value(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.show(views::todo_value(value, label))
    }

    pub fn selected(&self, value: &mut bool, label: impl Into<Str>) -> Response<bool> {
        self.show(views::selected(value, label))
    }

    pub fn radio<V>(&self, value: V, existing: &mut V, label: impl Into<Str>) -> Response<bool>
    where
        V: PartialEq + 'static,
    {
        self.show(views::radio(value, existing, label))
    }

    pub fn label(&self, data: impl Into<Str>) -> Response {
        self.show(views::label(data))
    }

    pub fn expand<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Flexible::new(Flex::Tight(1.0)), show)
            .flatten_right()
    }

    pub fn flex<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(views::Flexible::new(Flex::Loose(1.0)), show)
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
