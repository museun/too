use std::{cell::Cell, collections::HashMap};

use crate::{
    animation::Animations,
    backend::Event as TooEvent,
    backend::{Key, Keybind, Modifiers, MouseButton},
    math::{Pos2, Rect, Vec2},
};

use super::{Erased, LayoutNodes, ViewId, ViewNodes};

mod interest;
pub use interest::Interest;

mod view_event;
pub use view_event::ViewEvent;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Handled {
    Sink,
    #[default]
    Bubble,
}

impl Handled {
    pub const fn is_sink(&self) -> bool {
        matches!(self, Self::Sink)
    }

    pub const fn is_bubble(&self) -> bool {
        matches!(self, Self::Bubble)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

impl ButtonState {
    pub const fn is_down(&self) -> bool {
        matches!(self, Self::JustDown | Self::Down)
    }

    pub fn settle(&mut self) {
        *self = match self {
            Self::JustDown => Self::Down,
            Self::JustUp => Self::Up,
            _ => return,
        }
    }
}

#[derive(Default, Debug)]
struct Intersections {
    hit: Vec<ViewId>,
    entered: Vec<ViewId>, // TODO why isn't this a hashset?
    sunk: Vec<ViewId>,
}

impl Intersections {
    fn remove(&mut self, id: ViewId) {
        self.hit.retain(|&c| c != id);
        self.entered.retain(|&c| c != id);
        self.sunk.retain(|&c| c != id);
    }
}

#[derive(Default, Debug)]
struct Mouse {
    pos: Pos2,
    drag_start: Option<Pos2>,
    buttons: HashMap<MouseButton, ButtonState>,
}

#[derive(Debug, Default)]
struct Notify<T: Copy + PartialEq = ViewId> {
    current: Cell<Option<T>>,
    prev: Option<T>,
}

impl<T: Copy + PartialEq> Notify<T> {
    fn get(&self) -> Option<T> {
        self.current.get()
    }

    fn set(&self, value: Option<T>) {
        self.current.set(value);
    }
}

#[derive(Debug, Default)]
struct Focus {
    notify: Notify,
}

#[derive(Debug, Default)]
struct Selection {
    notify: Notify,
}

#[derive(Debug, Default)]
pub struct InputState {
    mouse: Mouse,
    modifiers: Modifiers,
    intersections: Intersections,

    focus: Focus,
    selection: Selection,

    key_press: Option<Keybind>,
}

impl InputState {
    pub fn begin(&mut self, nodes: &ViewNodes, layout: &LayoutNodes, animation: &mut Animations) {
        self.notify_focus(nodes, layout, animation);
        self.notify_selection(nodes, layout, animation);
    }

    pub fn end(&mut self) {
        self.key_press.take();
        for state in self.mouse.buttons.values_mut() {
            state.settle();
        }
    }

    pub fn key_press(&self) -> Option<Keybind> {
        self.key_press
    }

    pub fn mouse_pos(&self) -> Pos2 {
        self.mouse.pos
    }

    pub fn focus(&self) -> Option<ViewId> {
        self.focus.notify.get()
    }

    pub fn set_focus(&self, id: Option<ViewId>) {
        self.focus.notify.set(id)
    }

    pub fn selection(&self) -> Option<ViewId> {
        self.selection.notify.get()
    }

    pub fn set_selection(&self, id: Option<ViewId>) {
        self.selection.notify.set(id)
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn update(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
        event: &TooEvent,
    ) -> Handled {
        if let Some(modifiers) = event.modifiers() {
            self.modifiers = modifiers;
        }

        match *event {
            TooEvent::KeyPressed { key, .. } => {
                self.key_press = Some(Keybind::new(key, self.modifiers));
                self.update_key_event(key, nodes, layout, animation)
            }

            TooEvent::MouseMove { pos } => self.mouse_moved(pos, nodes, layout, animation),

            TooEvent::MouseButtonChanged {
                button, down, pos, ..
            } => {
                self.mouse.pos = pos;
                if self.mouse_button_changed(button, down) {
                    let resp = self.send_mouse_button_changed(button, nodes, layout, animation);
                    // TODO don't do this here
                    if resp.is_bubble() && (button == MouseButton::Primary && down) {
                        self.set_focus(None);
                        self.notify_focus(nodes, layout, animation);

                        self.set_selection(None);
                        self.notify_selection(nodes, layout, animation);
                    }
                    return resp;
                }
                Handled::Bubble
            }

            TooEvent::MouseDrag { pos, button, .. } => {
                let (start, delta) = {
                    let previous = std::mem::replace(&mut self.mouse.pos, pos);
                    let &mut start = self.mouse.drag_start.get_or_insert(pos);
                    if previous == pos {
                        return Handled::Bubble;
                    }
                    (start, pos - previous)
                };
                let delta = delta.to_vec2();
                self.send_mouse_drag(start, delta, button, nodes, layout, animation)
            }

            TooEvent::MouseScroll { delta, .. } => {
                self.mouse_scrolled(delta, nodes, layout, animation)
            }
            _ => Handled::Bubble,
        }
    }

    fn update_key_event(
        &mut self,
        key: Key,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let Some(id) = self.focus.notify.get() else {
            return Handled::Bubble;
        };

        let Some(view) = layout.get(id) else {
            return Handled::Bubble;
        };

        if !view.interest.is_focus_input() {
            return Handled::Bubble;
        }

        let event = ViewEvent::KeyInput {
            key,
            modifiers: self.modifiers,
        };
        self.dispatch(nodes, layout, animation, id, event)
    }

    fn mouse_moved(
        &mut self,
        pos: Pos2,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        self.mouse.pos = pos;
        self.send_mouse_move(nodes, layout, animation);
        self.mouse_hit_test(nodes, layout);
        self.send_mouse_enter(nodes, layout, animation);
        self.send_mouse_leave(nodes, layout, animation);
        Handled::Bubble
    }

    fn send_mouse_move(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        let event = ViewEvent::MouseMove {
            pos: self.mouse.pos,
            modifiers: self.modifiers,
        };

        for (id, interest) in layout.interest.iter() {
            if !interest.is_mouse_move() {
                continue;
            }

            self.dispatch(nodes, layout, animation, id, event);
        }
    }

    fn send_mouse_enter(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        for &hit in &self.intersections.hit {
            if !self.intersections.entered.contains(&hit) {
                self.intersections.entered.push(hit);
            } else if self.intersections.sunk.contains(&hit) {
                break;
            }

            if self
                .dispatch(nodes, layout, animation, hit, ViewEvent::MouseEntered)
                .is_sink()
            {
                self.intersections.sunk.push(hit);
                break;
            }
        }
    }

    fn send_mouse_leave(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        // TODO small vec
        let mut inactive = vec![];
        for &hit in &self.intersections.entered {
            if !self.intersections.hit.contains(&hit) {
                self.dispatch(nodes, layout, animation, hit, ViewEvent::MouseLeave);
                inactive.push(hit);
            }
        }

        for inactive in inactive {
            self.intersections.entered.retain(|&id| id != inactive);
            self.intersections.sunk.retain(|&id| id != inactive);
        }
    }

    fn mouse_button_changed(&mut self, button: MouseButton, down: bool) -> bool {
        let state = self.mouse.buttons.entry(button).or_insert(ButtonState::Up);
        match (state.is_down(), down) {
            (true, true) | (false, false) => {}
            (false, true) => *state = ButtonState::JustDown,
            (true, false) => *state = ButtonState::JustUp,
        };

        if !down {
            return self.mouse.drag_start.take().is_none();
        }
        true
    }

    fn send_mouse_drag(
        &self,
        start: Pos2,
        delta: Vec2,
        button: MouseButton,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let mut resp = Handled::Bubble;

        let event = ViewEvent::MouseDrag {
            start,
            current: self.mouse.pos,
            delta,
            inside: true,
            modifiers: self.modifiers,
            button,
        };

        for &hit in &self.intersections.hit {
            if self
                .dispatch(nodes, layout, animation, hit, event)
                .is_sink()
            {
                resp = Handled::Sink;
                break;
            }
        }

        let event = ViewEvent::MouseDrag {
            start,
            current: self.mouse.pos,
            delta,
            inside: false,
            modifiers: self.modifiers,
            button,
        };

        for (id, interest) in layout.interest.iter() {
            if interest.is_mouse_outside() && !self.intersections.hit.contains(&id) {
                self.dispatch(nodes, layout, animation, id, event);
            }
        }

        resp
    }

    fn send_mouse_button_changed(
        &mut self,
        button: MouseButton,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let state = *self.mouse.buttons.entry(button).or_insert(ButtonState::Up);

        let mut resp = Handled::Bubble;
        let event = if state.is_down() {
            ViewEvent::MouseHeld {
                pos: self.mouse.pos,
                inside: true,
                button,
                modifiers: self.modifiers,
            }
        } else {
            ViewEvent::MouseClicked {
                pos: self.mouse.pos,
                inside: true,
                button,
                modifiers: self.modifiers,
            }
        };

        for &hit in &self.intersections.hit {
            if self
                .dispatch(nodes, layout, animation, hit, event)
                .is_sink()
            {
                resp = Handled::Sink;
                break;
            }
        }

        let event = if state.is_down() {
            ViewEvent::MouseHeld {
                pos: self.mouse.pos,
                inside: false,
                button,
                modifiers: self.modifiers,
            }
        } else {
            ViewEvent::MouseClicked {
                pos: self.mouse.pos,
                inside: false,
                button,
                modifiers: self.modifiers,
            }
        };

        for (id, interest) in layout.interest.iter() {
            if interest.is_mouse_outside() && !self.intersections.hit.contains(&id) {
                self.dispatch(nodes, layout, animation, id, event);
            }
        }

        resp
    }

    fn mouse_scrolled(
        &self,
        delta: Vec2,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let event = ViewEvent::MouseScroll {
            delta,
            modifiers: self.modifiers,
        };
        for &hit in &self.intersections.hit {
            if self
                .dispatch(nodes, layout, animation, hit, event)
                .is_sink()
            {
                return Handled::Sink;
            }
        }

        Handled::Bubble
    }

    fn notify_selection(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        let mut current = self.selection.notify.get();
        let previous = self.selection.notify.prev;
        if current == previous {
            return;
        }

        if let Some(entered) = current {
            let ev = ViewEvent::SelectionAdded(entered);

            for (id, interest) in layout.interest.iter() {
                if !interest.is_selection_change() {
                    continue;
                }

                let resp = nodes.scoped(id, |node| {
                    self.send_event(nodes, layout, animation, id, node, ev)
                });

                if let Some(Handled::Sink) = resp {
                    break;
                } else if resp.is_none() {
                    // if the node doesn't exist clear the notification
                    self.selection.notify.set(None);
                    current = None;
                }
            }
        }

        if let Some(left) = previous {
            let ev = ViewEvent::SelectionRemoved(left);
            for (id, interest) in layout.interest.iter() {
                if !interest.is_selection_change() {
                    continue;
                }
                if self.dispatch(nodes, layout, animation, id, ev).is_sink() {
                    break;
                }
            }
        }

        self.selection.notify.prev = current;
    }

    fn notify_focus(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        let mut current = self.focus.notify.get();
        let previous = self.focus.notify.prev;
        if current == previous {
            return;
        }

        if let Some(entered) = current {
            let ev = ViewEvent::FocusGained;
            if nodes
                .scoped(entered, |node| {
                    self.send_event(nodes, layout, animation, entered, node, ev);
                })
                .is_none()
            {
                // if the node doesn't exist clear the notification
                self.focus.notify.set(None);
                current = None;
            }
        }

        if let Some(left) = previous {
            let ev = ViewEvent::FocusLost;
            self.dispatch(nodes, layout, animation, left, ev);
        }

        self.focus.notify.prev = current;
    }

    fn send_event(
        &self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
        id: ViewId,
        node: &mut dyn Erased,
        event: ViewEvent,
    ) -> Handled {
        let ctx = EventCtx {
            current: id,
            nodes,
            layout,
            animation,
            input: self,
        };
        nodes.begin(id); // TODO this should be done implicitly by the node scope
        let resp = node.event(event, ctx);
        nodes.end(id);
        resp
    }

    fn mouse_hit_test(&mut self, _nodes: &ViewNodes, layout: &LayoutNodes) {
        self.intersections.hit.clear();
        Self::hit_test(self.mouse.pos, layout, &mut self.intersections.hit);
    }

    pub(in crate::view) fn remove(&mut self, id: ViewId) {
        self.intersections.remove(id);
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    fn hit_test(pos: Pos2, layout: &LayoutNodes, out: &mut Vec<ViewId>) {
        for (id, _) in layout.interest.iter() {
            let Some(mut node) = layout.get(id) else {
                continue;
            };

            let mut rect = node.rect;
            while let Some(parent) = node.clipped_by {
                node = layout.get(parent).unwrap();
                rect = rect.intersection(node.rect)
            }

            if rect.contains(pos) {
                out.push(id);
            }
        }
    }

    fn dispatch(
        &self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
        id: ViewId,
        event: ViewEvent,
    ) -> Handled {
        nodes
            .scoped(id, |node| {
                self.send_event(nodes, layout, animation, id, node, event)
            })
            .unwrap_or(Handled::Bubble)
    }

    pub(crate) fn is_focused(&self, current: ViewId) -> bool {
        self.focus.notify.get() == Some(current)
    }

    pub(crate) fn is_hovered(&self, current: ViewId) -> bool {
        self.intersections.hit.contains(&current)
    }
}

pub struct EventCtx<'a> {
    pub current: ViewId,
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,
    pub input: &'a InputState,
    pub animation: &'a mut Animations,
}

impl<'a> EventCtx<'a> {
    pub fn send_event(&mut self, id: ViewId, event: ViewEvent) -> Handled {
        self.input.dispatch(
            self.nodes, //
            self.layout,
            self.animation,
            id,
            event,
        )
    }

    pub fn cursor_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    #[track_caller]
    pub fn rect(&self) -> Rect {
        self.layout.rect(self.current).unwrap()
    }
}
