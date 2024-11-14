use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use crate::{
    math::{Pos2, Rect, Vec2},
    Animations, Event as TooEvent, Key, Modifiers, MouseButton,
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
    entered: Vec<ViewId>,
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
pub struct InputState {
    mouse: RefCell<Mouse>,
    modifiers: Cell<Modifiers>,
    intersections: RefCell<Intersections>,
    focus: Cell<Option<ViewId>>,
    prev_focus: Cell<Option<ViewId>>,
}

impl InputState {
    pub fn begin(&self, nodes: &ViewNodes, layout: &LayoutNodes, animation: &mut Animations) {
        self.notify_focus(nodes, layout, animation)
    }

    pub fn end(&mut self) {
        let mut mouse = self.mouse.borrow_mut();
        for state in mouse.buttons.values_mut() {
            state.settle();
        }
    }

    pub fn mouse_pos(&self) -> Pos2 {
        self.mouse.borrow().pos
    }

    pub fn focus(&self) -> Option<ViewId> {
        self.focus.get()
    }

    pub fn set_focus(&self, id: Option<ViewId>) {
        self.focus.set(id);
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn update(
        &self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
        event: &TooEvent,
    ) -> Handled {
        if let Some(modifiers) = event.modifiers() {
            self.modifiers.set(modifiers);
        }

        match *event {
            TooEvent::KeyPressed { key, .. } => {
                self.update_key_event(key, nodes, layout, animation)
            }
            TooEvent::MouseMove { pos } => self.mouse_moved(pos, nodes, layout, animation),
            TooEvent::MouseButtonChanged {
                button, down, pos, ..
            } => {
                self.mouse.borrow_mut().pos = pos;
                if self.mouse_button_changed(button, down, nodes, layout) {
                    let resp =
                        self.send_mouse_button_changed(button, down, nodes, layout, animation);
                    if resp.is_bubble() && (button == MouseButton::Primary && down) {
                        self.set_focus(None);
                        self.notify_focus(nodes, layout, animation);
                    }
                    return resp;
                }
                Handled::Bubble
            }
            TooEvent::MouseDrag { pos, button, .. } => {
                let (start, delta) = {
                    let mut mouse = self.mouse.borrow_mut();
                    let previous = std::mem::replace(&mut mouse.pos, pos);
                    let &mut start = mouse.drag_start.get_or_insert(pos);
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
        &self,
        key: Key,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let Some(id) = self.focus.get() else {
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
            modifiers: self.modifiers.get(),
        };

        nodes
            .scoped(id, |node| {
                self.send_event(nodes, layout, animation, id, node, event)
            })
            .unwrap()
    }

    fn mouse_moved(
        &self,
        pos: Pos2,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        self.mouse.borrow_mut().pos = pos;
        self.send_mouse_move(nodes, layout, animation);
        self.mouse_hit_test(nodes, layout);
        self.send_mouse_enter(nodes, layout, animation);
        self.send_mouse_leave(nodes, layout, animation);
        Handled::Bubble
    }

    fn send_mouse_move(&self, nodes: &ViewNodes, layout: &LayoutNodes, animation: &mut Animations) {
        let mouse = self.mouse.borrow();
        let event = ViewEvent::MouseMove {
            pos: mouse.pos,
            modifiers: self.modifiers.get(),
        };

        for (id, interest) in layout.interest.iter() {
            if !interest.is_mouse_move() {
                continue;
            }

            nodes.scoped(id, |node| {
                self.send_event(nodes, layout, animation, id, node, event);
            });
        }
    }

    fn send_mouse_enter(
        &self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        let intersections = &mut *self.intersections.borrow_mut();

        for &hit in &intersections.hit {
            if !nodes
                .scoped(hit, |node| {
                    if !intersections.entered.contains(&hit) {
                        intersections.entered.push(hit);
                        let ev = ViewEvent::MouseEntered;
                        let resp = self.send_event(nodes, layout, animation, hit, node, ev);
                        if resp.is_sink() {
                            intersections.sunk.push(hit);
                            return false;
                        }
                    } else if intersections.sunk.contains(&hit) {
                        return false;
                    }
                    true
                })
                .unwrap_or(true)
            {
                break;
            }
        }
    }

    fn send_mouse_leave(
        &self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) {
        let intersections = &mut *self.intersections.borrow_mut();

        // TODO small vec
        let mut inactive = vec![];
        for &hit in &intersections.entered {
            if !intersections.hit.contains(&hit) {
                nodes.scoped(hit, |node| {
                    self.send_event(nodes, layout, animation, hit, node, ViewEvent::MouseLeave);
                    inactive.push(hit);
                });
            }
        }

        for inactive in inactive {
            intersections.entered.retain(|&id| id != inactive);
            intersections.sunk.retain(|&id| id != inactive);
        }
    }

    fn mouse_button_changed(
        &self,
        button: MouseButton,
        down: bool,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
    ) -> bool {
        let mut mouse = self.mouse.borrow_mut();
        let state = mouse.buttons.entry(button).or_insert(ButtonState::Up);
        match (state.is_down(), down) {
            (true, true) | (false, false) => {}
            (false, true) => *state = ButtonState::JustDown,
            (true, false) => *state = ButtonState::JustUp,
        };

        if !down {
            return mouse.drag_start.take().is_none();
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
        let mouse = self.mouse.borrow();
        let intersections = self.intersections.borrow();

        let mut resp = Handled::Bubble;

        let event = ViewEvent::MouseDrag {
            start,
            current: mouse.pos,
            delta,
            inside: true,
            modifiers: self.modifiers.get(),
            button,
        };

        for &hit in &intersections.hit {
            if !nodes
                .scoped(hit, |node| {
                    let new = self.send_event(nodes, layout, animation, hit, node, event);
                    if new.is_sink() {
                        resp = new;
                        return false;
                    }
                    true
                })
                .unwrap_or(true)
            {
                break;
            }
        }

        let event = ViewEvent::MouseDrag {
            start,
            current: mouse.pos,
            delta,
            inside: false,
            modifiers: self.modifiers.get(),
            button,
        };

        for (id, interest) in layout.interest.iter() {
            if interest.is_mouse_outside() && !intersections.hit.contains(&id) {
                nodes.scoped(id, |node| {
                    self.send_event(nodes, layout, animation, id, node, event);
                });
            }
        }

        resp
    }

    fn send_mouse_button_changed(
        &self,
        button: MouseButton,
        down: bool,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        animation: &mut Animations,
    ) -> Handled {
        let state = {
            let mut mouse = self.mouse.borrow_mut();
            *mouse.buttons.entry(button).or_insert(ButtonState::Up)
        };

        let mouse = self.mouse.borrow();
        let intersections = self.intersections.borrow();
        let mut resp = Handled::Bubble;

        let event = if state.is_down() {
            ViewEvent::MouseHeld {
                pos: mouse.pos,
                inside: true,
                button,
                modifiers: self.modifiers.get(),
            }
        } else {
            ViewEvent::MouseClicked {
                pos: mouse.pos,
                inside: true,
                button,
                modifiers: self.modifiers.get(),
            }
        };

        for &hit in &intersections.hit {
            if !nodes
                .scoped(hit, |node| {
                    let new = self.send_event(nodes, layout, animation, hit, node, event);
                    if new.is_sink() {
                        resp = new;
                        return false;
                    }
                    true
                })
                .unwrap_or(true)
            {
                break;
            }
        }

        let event = if state.is_down() {
            ViewEvent::MouseHeld {
                pos: mouse.pos,
                inside: false,
                button,
                modifiers: self.modifiers.get(),
            }
        } else {
            ViewEvent::MouseClicked {
                pos: mouse.pos,
                inside: false,
                button,
                modifiers: self.modifiers.get(),
            }
        };

        for (id, interest) in layout.interest.iter() {
            if interest.is_mouse_outside() && !intersections.hit.contains(&id) {
                nodes.scoped(id, |node| {
                    self.send_event(nodes, layout, animation, id, node, event);
                });
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
        let intersections = self.intersections.borrow();

        let event = ViewEvent::MouseScroll {
            delta,
            modifiers: self.modifiers.get(),
        };
        for &hit in &intersections.hit {
            if nodes
                .scoped(hit, |node| {
                    self.send_event(nodes, layout, animation, hit, node, event)
                })
                .unwrap_or_default()
                .is_sink()
            {
                return Handled::Sink;
            }
        }

        Handled::Bubble
    }

    fn notify_focus(&self, nodes: &ViewNodes, layout: &LayoutNodes, animation: &mut Animations) {
        let mut current = self.focus.get();
        let last = self.prev_focus.get();
        if current == last {
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
                self.focus.set(None);
                current = None;
            }
        }

        if let Some(left) = last {
            let ev = ViewEvent::FocusLost;
            nodes.scoped(left, |node| {
                self.send_event(nodes, layout, animation, left, node, ev);
            });
        }

        self.prev_focus.set(current);
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
        nodes.begin(id);
        let resp = node.event(event, ctx);
        nodes.end(id);
        resp
    }

    fn mouse_hit_test(&self, nodes: &ViewNodes, layout: &LayoutNodes) {
        let mut intersections = self.intersections.borrow_mut();
        intersections.hit.clear();
        let mouse = self.mouse.borrow();
        Self::hit_test(mouse.pos, layout, &mut intersections.hit);
    }

    pub(in crate::view) fn remove(&self, id: ViewId) {
        self.intersections.borrow_mut().remove(id);
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
        // TODO ViewNodes::scoped
        let Some(node) = nodes.get(id) else {
            return Handled::Bubble;
        };

        nodes.begin(id);
        let ctx = EventCtx {
            current: id,
            nodes,
            layout,
            animation,
            input: self,
        };
        let resp = node.view.borrow_mut().event(event, ctx);
        nodes.end(id);
        resp
    }

    pub(crate) fn is_focused(&self, current: ViewId) -> bool {
        self.focus.get() == Some(current)
    }

    pub(crate) fn is_hovered(&self, current: ViewId) -> bool {
        self.intersections.borrow().hit.contains(&current)
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
    pub fn event(&mut self, id: ViewId, event: ViewEvent) -> Handled {
        self.input
            .dispatch(self.nodes, self.layout, self.animation, id, event)
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
