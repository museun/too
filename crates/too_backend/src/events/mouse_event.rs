use too_math::{Pos2, Vec2};

use crate::{Event, Modifiers, MouseButton};

#[derive(Default, Debug)]
enum MouseKind {
    #[default]
    None,
    Held,
    DragStart(Pos2),
    Drag {
        previous: Pos2,
        origin: Pos2,
    },
}

#[derive(Default)]
pub struct MouseState {
    pos: Pos2,
    previous: MouseKind,
    button: Option<MouseButton>,
}

impl MouseState {
    pub fn update(
        &mut self,
        ev: TemporalEvent,
        raw_pos: Pos2,
        modifiers: Modifiers,
    ) -> Option<Event> {
        use TemporalEvent as E;

        let mev = match ev {
            E::Down(pos, button) => {
                self.previous = MouseKind::Held;
                self.pos = pos;
                self.button = Some(button);
                Event::MouseHeld {
                    button,
                    pos: raw_pos,
                    modifiers,
                }
            }
            E::Up(pos, button) => match std::mem::take(&mut self.previous) {
                MouseKind::Held if self.check(pos, button) => {
                    self.button.take();
                    Event::MouseClick {
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                MouseKind::Drag { origin, .. } if Some(button) == self.button => {
                    self.button.take();
                    Event::MouseDragRelease {
                        origin,
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                _ => return None,
            },
            E::Drag(pos, button) => match std::mem::take(&mut self.previous) {
                MouseKind::None if self.pos == pos => {
                    self.previous = MouseKind::Held;
                    self.pos = pos;
                    self.button = Some(button);
                    Event::MouseHeld {
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                MouseKind::Held if self.pos == pos => {
                    self.previous = MouseKind::Held;
                    self.pos = pos;
                    self.button = Some(button);
                    return None;
                }
                MouseKind::None | MouseKind::Held => {
                    self.previous = MouseKind::DragStart(pos);
                    self.pos = pos;
                    self.button = Some(button);
                    Event::MouseDragStart {
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                MouseKind::DragStart(origin) if self.check(origin, button) => {
                    self.previous = MouseKind::Drag {
                        previous: origin,
                        origin,
                    };
                    self.pos = origin;
                    self.button = Some(button);
                    Event::MouseDragHeld {
                        origin,
                        delta: Vec2::ZERO,
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                MouseKind::Drag {
                    previous: old,
                    origin,
                } if self.check(origin, button) => {
                    self.previous = MouseKind::Drag {
                        previous: pos,
                        origin,
                    };
                    self.pos = origin;
                    self.button = Some(button);
                    Event::MouseDragHeld {
                        origin,
                        delta: (pos - old).to_vec2(),
                        button,
                        pos: raw_pos,
                        modifiers,
                    }
                }
                _ => return None,
            },
        };

        Some(mev)
    }

    fn check(&self, pos: Pos2, button: MouseButton) -> bool {
        self.pos == pos && self.button == Some(button)
    }
}

pub enum TemporalEvent {
    Down(Pos2, MouseButton),
    Up(Pos2, MouseButton),
    Drag(Pos2, MouseButton),
}
