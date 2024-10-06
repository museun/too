use crate::{layout::Align2, math::Pos2, Rgba};
use compact_str::ToCompactString;

use super::{
    geom::{Flex, Margin},
    input::InputState,
    state::{Debug, LayoutNodes, ViewNodes},
    style::{Styled, Stylesheet, Theme},
    views::{self, Border, Constrain, Dragging, MouseArea, MouseAreaResponse, Unconstrained},
    Builder, DebugMode, Response, State, View, ViewId,
};

pub struct Ui<'a> {
    pub(in crate::view) nodes: &'a ViewNodes,
    pub(in crate::view) layout: &'a LayoutNodes,
    pub(in crate::view) input: &'a InputState,
    pub(in crate::view) theme: &'a Theme,
    pub(in crate::view) stylesheet: &'a Stylesheet,
    pub(in crate::view) debug: &'a Debug,
}

impl<'a> Ui<'a> {
    pub const fn new(state: &'a State) -> Self {
        Self {
            nodes: &state.nodes,
            layout: &state.layout,
            input: &state.input,
            theme: &state.theme,
            stylesheet: &state.stylesheet,
            debug: &state.debug,
        }
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
        // FIXME when we implement `UpdateFlags` we can have a `NoSpace` flag
        // that can be checked here and return early
        // (with Response<<B::View as View>::Response>::default())
        // but we'd need some 'null' id that won't ever do anything when its used
        // for that, we can just give out the root id. it can't be used any way
        let (id, resp) = self.nodes.begin_view::<B::View>(args, self);
        let inner = show(self);
        self.nodes.end_view(id);
        Response::new(id, (resp, inner))
    }
}

impl<'a> Ui<'a> {
    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn cursor_pos(&self) -> Pos2 {
        self.input.cursor_pos()
    }

    pub fn set_focus(&self, id: impl Into<Option<ViewId>>) {
        self.input.set_focus(id.into());
    }

    pub fn reset<T>(&self, key: Styled<T>)
    where
        T: 'static + Copy,
    {
        self.set(key, key.default());
    }

    pub fn set<T>(&self, key: Styled<T>, value: T)
    where
        T: 'static + Copy,
    {
        self.stylesheet.replace(self.current(), key, value);
    }

    pub const fn theme(&self) -> &Theme {
        self.theme
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
    pub fn set_debug_mode(&self, mode: DebugMode) {
        self.debug.mode.set(mode);
    }

    pub fn debug(&self, msg: impl ToString) {
        self.debug.push(msg);
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

    pub fn constrain<R>(&self, constrain: Constrain, show: impl FnOnce(&Ui) -> R) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(constrain, show).flatten_right()
    }

    pub fn unconstrained<R>(
        &self,
        unconstrained: Unconstrained,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<R>
    where
        R: 'static,
    {
        self.show_children(unconstrained, show).flatten_right()
    }

    pub fn draggable<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<(Option<Dragging>, R)>
    where
        R: 'static,
    {
        self.mouse_area(show).map(|(m, r)| (m.dragged(), r))
    }

    pub fn mouse_area<R>(&self, show: impl FnOnce(&Ui) -> R) -> Response<(MouseAreaResponse, R)>
    where
        R: 'static,
    {
        self.show_children(MouseArea::default(), show)
    }

    pub fn progress_bar(&self, value: f32) -> Response {
        self.show(views::progress_bar(value))
    }

    pub fn slider(&self, value: &mut f32) -> Response {
        self.show(views::slider(value))
    }

    pub fn radio<V>(&self, value: V, existing: &mut V, label: &str) -> Response<bool>
    where
        V: PartialEq,
    {
        let resp = self
            .mouse_area(|ui| {
                // let fill = match (ui.is_hovered(), *value == selected) {
                //     (false, true) => ui.theme().primary,
                //     (false, ..) => ui.theme().surface,
                //     _ => ui.theme().accent,
                // };

                let fill = if *existing == value {
                    ui.theme.primary
                } else {
                    ui.theme.surface
                };
                ui.background(fill, |ui| ui.label(label));
            })
            .flatten_left();

        if resp.clicked() {
            *existing = value;
        }
        resp.map(|c| c.clicked())
    }

    pub fn label(&self, data: impl ToCompactString) -> Response {
        self.show(views::label(data))
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
        self.show_children(views::border().style(border), show)
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
        self.show_children(views::border().style(border).title(title), show)
            .flatten_right()
    }
}
