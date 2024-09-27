use std::collections::{HashMap, HashSet};

use too_events::{Key, Modifiers, MouseButton};

use crate::{
    geom::{Point, Rectf, Vector},
    view_node::ViewNode,
    ViewId,
};

#[derive(Copy, Clone, PartialEq)]
pub struct Interest(u8);

impl std::fmt::Debug for Interest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FIELDS: [&str; 6] = [
            "MOUSE_ENTER",
            "MOUSE_LEAVE",
            "MOUSE_MOVE",
            "KEY_INPUT",
            "FOCUS_GAINED",
            "FOCUS_LOST",
        ];

        let mut seen = false;

        for (flag, repr) in (0..).zip(FIELDS) {
            if (self.0 >> flag) & 1 == 1 {
                if seen {
                    f.write_str(" | ")?;
                }
                f.write_str(repr)?;
                seen |= true
            }
        }

        if !seen {
            f.write_str("NONE")?;
        }

        Ok(())
    }
}

impl Interest {
    pub const NONE: Self = Self(0);
    pub const MOUSE_ENTER: Self = Self(1 << 0);
    pub const MOUSE_LEAVE: Self = Self(1 << 1);
    pub const MOUSE_MOVE: Self = Self(1 << 2);
    pub const KEY_INPUT: Self = Self(1 << 3);
    pub const FOCUS_GAINED: Self = Self(1 << 4);
    pub const FOCUS_LOST: Self = Self(1 << 5);

    pub const MOUSE: Self = Self(Self::MOUSE_ENTER.0 | Self::MOUSE_LEAVE.0 | Self::MOUSE_MOVE.0);
    pub const FOCUS: Self = Self(Self::FOCUS_GAINED.0 | Self::FOCUS_LOST.0);
}

impl Interest {
    pub const fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_mouse_any(&self) -> bool {
        self.is_mouse_enter() || self.is_mouse_leave() || self.is_mouse_move()
    }

    pub const fn is_focus(&self) -> bool {
        self.is_focus_gained() || self.is_focus_lost()
    }

    pub const fn is_mouse_enter(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub const fn is_mouse_leave(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub const fn is_mouse_move(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub const fn is_key_input(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub const fn is_focus_gained(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub const fn is_focus_lost(&self) -> bool {
        self.0 & (1 << 5) != 0
    }
}

impl std::ops::BitAnd for Interest {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for Interest {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for Interest {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitAndAssign for Interest {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOrAssign for Interest {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitXorAssign for Interest {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl std::ops::Not for Interest {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(self.0)
    }
}

pub struct EventCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub rect: Rectf,

    debug: &'a mut Vec<String>,
    // layout nodes
    // hovered status
}

impl<'a, T: 'static> EventCtx<'a, T> {
    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Handled {
    #[default]
    Bubble,
    Sink,
}

impl Handled {
    const fn is_sink(&self) -> bool {
        matches!(self, Self::Sink)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseMove {
    pub pos: Point,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseClick {
    pub pos: Point,
    pub button: MouseButton,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseHeld {
    pub pos: Point,
    pub button: MouseButton,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseDrag {
    pub released: bool,
    pub origin: Point,
    pub pos: Point,
    pub delta: Vector,
    pub button: MouseButton,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseScroll {
    pub pos: Point,
    pub delta: Vector,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KeyInput {
    pub key: Key,
    pub modifiers: Modifiers,
    // TODO DOWN | UP | REPEAT
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    MouseEnter(MouseMove),
    MouseLeave(MouseMove),
    MouseMove(MouseMove),
    MouseClick(MouseClick),
    MouseHeld(MouseHeld),
    MouseDrag(MouseDrag),
    MouseScroll(MouseScroll),
    KeyInput(KeyInput),
    FocusGained,
    FocusLost,
}

impl Event {
    pub fn modifiers(&self) -> Option<Modifiers> {
        match self {
            Self::MouseEnter(MouseMove { modifiers, .. })
            | Self::MouseLeave(MouseMove { modifiers, .. })
            | Self::MouseMove(MouseMove { modifiers, .. })
            | Self::MouseClick(MouseClick { modifiers, .. })
            | Self::MouseHeld(MouseHeld { modifiers, .. })
            | Self::MouseDrag(MouseDrag { modifiers, .. })
            | Self::MouseScroll(MouseScroll { modifiers, .. })
            | Self::KeyInput(KeyInput { modifiers, .. }) => Some(*modifiers),
            _ => None,
        }
        .filter(|c| !c.is_none())
    }
}

#[derive(Debug, Default)]
struct Intersections {
    hit: HashSet<ViewId>,
    entered: Vec<ViewId>,
    entered_and_sunk: Vec<ViewId>,
}

#[derive(Debug, Default)]
pub struct Mouse {
    pub current: Point,
    pub previous: Point,
    layered: Layered<Interest>,
    pub mouse_over: HashSet<ViewId>,
    pub buttons: HashMap<MouseButton, ButtonState>,
}

impl Mouse {
    pub fn push_layer(&mut self, id: ViewId) {
        self.layered.push_layer(id);
    }

    pub fn pop_layer(&mut self) {
        self.layered.pop_layer();
    }

    pub fn current_layer_root(&self) -> Option<ViewId> {
        self.layered.current_root()
    }

    pub fn clear(&mut self) {
        self.layered.clear();
    }

    pub fn add(&mut self, id: ViewId, interest: Interest) {
        self.layered.insert(id, interest);
    }

    pub fn remove(&mut self, id: ViewId) {
        self.layered.remove(id);
    }

    pub fn hovered(&mut self, hit: ViewId) {
        self.mouse_over.insert(hit);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ButtonState {
    Held,
    Released,
}

#[derive(Debug, Default)]
pub struct Keyboard {
    layered: Layered,
}

impl Keyboard {
    pub fn push_layer(&mut self, id: ViewId) {
        self.layered.push_layer(id);
    }

    pub fn pop_layer(&mut self) {
        self.layered.pop_layer();
    }

    pub fn current_layer_root(&mut self) -> Option<ViewId> {
        self.layered.current_root()
    }

    pub fn clear(&mut self) {
        self.layered.clear();
    }

    pub fn add(&mut self, id: ViewId) {
        self.layered.insert(id, ());
    }

    pub fn remove(&mut self, id: ViewId) {
        self.layered.remove(id);
    }
}

#[derive(Debug)]
pub struct Layered<T = ()> {
    // PERF don't do this like this
    pub layers: Vec<Vec<Item<T>>>,
    pub stack: Vec<Item<usize>>,
}

impl<T> Default for Layered<T> {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            stack: Vec::new(),
        }
    }
}

impl<T> Layered<T> {
    // NOTE: this doesn't reuse the allocations
    pub fn clear(&mut self) {
        std::mem::take(self);
    }

    pub fn insert(&mut self, id: ViewId, item: T) {
        self.stack
            .last()
            .and_then(|&Item { item, .. }| self.layers.get_mut(item))
            .unwrap()
            .push(Item { id, item });
    }

    pub fn remove(&mut self, vid: ViewId) {
        self.stack.retain(|&Item { id, .. }| id != vid);
        for layer in &mut self.layers {
            layer.retain(|&Item { id, .. }| id != vid);
        }
    }

    pub fn current_root(&self) -> Option<ViewId> {
        self.stack.last().map(|&Item { id, .. }| id)
    }

    pub fn push_layer(&mut self, id: ViewId) {
        let item = self.layers.len();
        self.layers.push(vec![]);
        self.stack.push(Item { id, item });
    }

    pub fn pop_layer(&mut self) {
        assert!(
            self.stack.pop().is_some(),
            "cannot pop a layer without one existing"
        );
    }

    // PERF be smarter about this
    pub fn iter(&self) -> impl Iterator<Item = (&ViewId, &T)> + '_ {
        self.layers
            .iter()
            .rev()
            .flatten()
            .map(|item| (&item.id, &item.item))
    }
}

#[derive(Debug)]
pub struct Item<T> {
    pub id: ViewId,
    pub item: T,
}

#[derive(Debug, Default)]
pub struct Input {
    pub(crate) mouse: Mouse,
    pub(crate) keyboard: Keyboard,
    modifiers: Modifiers,
    last_event: Option<too_events::Event>,
    intersections: Intersections,
}

const fn modifiers_for_event(event: &too_events::Event) -> Option<Modifiers> {
    use too_events::Event as E;
    match event {
        E::KeyPressed { modifiers, .. }
        | E::KeyReleased { modifiers, .. }
        | E::KeyRepeat { modifiers, .. }
        | E::MouseMove { modifiers, .. }
        | E::MouseClick { modifiers, .. }
        | E::MouseHeld { modifiers, .. }
        | E::MouseDragStart { modifiers, .. }
        | E::MouseDragHeld { modifiers, .. }
        | E::MouseDragRelease { modifiers, .. }
        | E::MouseScroll { modifiers, .. } => Some(*modifiers),
        _ => None,
    }
}

impl Input {
    pub fn begin(&mut self) {
        // TODO focus stuff
    }

    pub fn end(&mut self, removed: &[ViewId]) {
        // TODO focus stuff
        // for &id in removed {
        //     self.remove(id);
        // }
    }

    pub fn handle<T: 'static>(
        &mut self,
        event: &too_events::Event,
        nodes: &mut thunderdome::Arena<Option<ViewNode<T>>>,
        state: &mut T,
        debug: &mut Vec<String>,
    ) -> Handled {
        self.last_event = Some(event.clone());
        self.modifiers = modifiers_for_event(event).unwrap_or(Modifiers::NONE);

        macro_rules! ctx {
            () => {
                Context {
                    nodes,
                    mouse: &mut self.mouse,
                    intersections: &mut self.intersections,
                    state,
                    debug,
                }
            };
        }

        use too_events::Event as E;
        match *event {
            E::KeyPressed { key, .. } => {
                let event = Event::KeyInput(KeyInput {
                    key,
                    modifiers: self.modifiers,
                });

                let mut resp = Handled::Bubble;

                for (&id, ()) in self.keyboard.layered.iter() {
                    if resp.is_sink() {
                        break;
                    }

                    let node = nodes[id.0].as_mut().unwrap();
                    let ctx = EventCtx {
                        current_id: id,
                        children: &node.children,
                        state,
                        rect: node.rect,
                        debug,
                    };

                    resp = node.view.event(ctx, &event);
                }

                resp
            }

            E::MouseMove { pos, .. } => {
                let event = MouseMove {
                    pos: pos.into(),
                    modifiers: self.modifiers,
                };
                ctx!().mouse_move(event)
            }

            E::MouseClick { pos, button, .. } => {
                self.mouse.buttons.insert(button, ButtonState::Released);
                let event = MouseClick {
                    pos: pos.into(),
                    button,
                    modifiers: self.modifiers,
                };
                ctx!().mouse_button(&Event::MouseClick(event))
            }

            E::MouseHeld { pos, button, .. } => {
                self.mouse.buttons.insert(button, ButtonState::Held);
                let event = MouseHeld {
                    pos: pos.into(),
                    button,
                    modifiers: self.modifiers,
                };
                ctx!().mouse_button(&Event::MouseHeld(event))
            }

            E::MouseDragStart { pos, button, .. } => {
                self.mouse.buttons.insert(button, ButtonState::Held);
                let pos = pos.into();
                let event = MouseDrag {
                    released: false,
                    origin: pos,
                    pos,
                    delta: Vector::ZERO,
                    button,
                    modifiers: self.modifiers,
                };
                ctx!().mouse_drag(event)
            }

            E::MouseDragHeld {
                pos,
                origin,
                delta,
                button,
                ..
            } => {
                self.mouse.buttons.insert(button, ButtonState::Held);
                let pos = pos.into();
                let event = MouseDrag {
                    released: false,
                    origin: pos,
                    pos,
                    delta: delta.into(),
                    button,
                    modifiers: self.modifiers,
                };
                ctx!().mouse_drag(event)
            }

            E::MouseDragRelease {
                pos,
                origin,
                button,
                ..
            } => {
                self.mouse.buttons.insert(button, ButtonState::Released);
                let pos = pos.into();
                let event = MouseDrag {
                    released: true,
                    origin: pos,
                    pos,
                    delta: Vector::ZERO,
                    button,
                    modifiers: self.modifiers,
                };
                ctx!().mouse_drag(event)
            }

            E::MouseScroll { pos, delta, .. } => {
                let event = MouseScroll {
                    pos: pos.into(),
                    delta: delta.into(),
                    modifiers: self.modifiers,
                };
                ctx!().mouse_scroll(event)
            }

            _ => Handled::Bubble,
        }
    }

    fn remove(&mut self, id: ViewId) {
        self.keyboard.remove(id);
        self.mouse.remove(id);
        self.mouse.mouse_over.remove(&id);
    }
}

struct Context<'a, T: 'static> {
    nodes: &'a mut thunderdome::Arena<Option<ViewNode<T>>>,
    mouse: &'a mut Mouse,
    intersections: &'a mut Intersections,
    state: &'a mut T,
    debug: &'a mut Vec<String>,
}

impl<'a, T: 'static> Context<'a, T> {
    fn mouse_move(&mut self, event: MouseMove) -> Handled {
        for (&id, interest) in self.mouse.layered.iter() {
            if !interest.is_mouse_move() {
                continue;
            }

            let node = self.nodes[id.0].as_mut().unwrap();
            let ctx = EventCtx {
                current_id: id,
                children: &node.children,
                state: self.state,
                rect: node.rect,
                debug: self.debug,
            };

            node.view.event(ctx, &Event::MouseMove(event));
        }

        self.intersections.hit.clear();
        self.do_hit_test(event.pos);

        for &hit in &self.intersections.hit {
            if self.intersections.entered.contains(&hit) {
                continue;
            }

            self.intersections.entered.push(hit);
            self.mouse.hovered(hit);

            let node = self.nodes[hit.0].as_mut().unwrap();
            let ctx = EventCtx {
                current_id: hit,
                children: &node.children,
                state: self.state,
                rect: node.rect,
                debug: self.debug,
            };

            if node.view.event(ctx, &Event::MouseEnter(event)).is_sink() {
                self.intersections.entered_and_sunk.push(hit);
                break;
            }

            if self.intersections.entered_and_sunk.contains(&hit) {
                break;
            }
        }

        let mut inactive = vec![];

        for (hit, _) in self.mouse.layered.iter() {
            if !self.intersections.entered.contains(hit) {
                continue;
            }

            let Some(node) = self.nodes.get_mut(hit.0) else {
                continue;
            };

            let Some(node) = node.as_mut() else {
                unreachable!("node {hit:?} is missing")
            };

            if node.rect.contains(event.pos) {
                continue;
            }

            self.mouse.mouse_over.remove(hit);

            let ctx = EventCtx {
                current_id: *hit,
                children: &node.children,
                state: self.state,
                rect: node.rect,
                debug: self.debug,
            };

            node.view.event(ctx, &Event::MouseLeave(event));
            inactive.push(hit);
        }

        for inactive in inactive {
            self.intersections.entered.retain(|id| id != inactive);
            self.intersections
                .entered_and_sunk
                .retain(|id| id != inactive);
        }

        Handled::Bubble
    }

    fn mouse_button(&mut self, event: &Event) -> Handled {
        for (id, interest) in self.mouse.layered.iter() {
            if !interest.is_mouse_any() {
                continue;
            }

            if !self.intersections.hit.contains(id) {
                continue;
            }

            let node = self.nodes[id.0].as_mut().unwrap();
            let ctx = EventCtx {
                current_id: *id,
                children: &node.children,
                state: self.state,
                rect: node.rect,
                debug: self.debug,
            };

            if node.view.event(ctx, event).is_sink() {
                return Handled::Sink;
            }
        }

        Handled::Bubble
    }

    fn mouse_drag(&mut self, event: MouseDrag) -> Handled {
        self.mouse_event(&Event::MouseDrag(event))
    }

    fn mouse_scroll(&mut self, event: MouseScroll) -> Handled {
        self.mouse_event(&Event::MouseScroll(event))
    }

    fn mouse_event(&mut self, event: &Event) -> Handled {
        for &id in &self.intersections.hit {
            let node = self.nodes[id.0].as_mut().unwrap();
            let ctx = EventCtx {
                current_id: id,
                children: &node.children,
                state: self.state,
                rect: node.rect,
                debug: self.debug,
            };

            if node.view.event(ctx, event).is_sink() {
                return Handled::Sink;
            }
        }

        Handled::Bubble
    }

    fn do_hit_test(&mut self, pos: Point) {
        for (&id, _) in self.mouse.layered.iter() {
            let Some(node) = self.nodes.get(id.0) else {
                continue;
            };

            let Some(node) = node else {
                unreachable!("node {id:?} is missing")
            };

            // let mut rect = node.rect;
            // TODO traverse the clip stack

            if node.rect.contains(pos) {
                self.intersections.hit.insert(id);
            }
        }
    }
}
