use std::time::Duration;

use crate::{
    animation::{easing, Animation},
    math::lerp,
    view::{
        geom::{Size, Space},
        properties::{Elements, Theme},
        Builder, Event, Handled, Interest, Layout, Render, Response, Ui, Update, View, ViewEvent,
        ViewId,
    },
    Pixel,
};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct ToggleResponse {
    changed: bool,
}

impl ToggleResponse {
    pub fn changed(&self) -> bool {
        self.changed
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct ToggleSwitch<'a> {
    value: &'a mut bool,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self { value }
    }
}

impl<'v> Builder<'v> for ToggleSwitch<'v> {
    type View = ToggleSwitchView;
}

#[derive(Debug)]
pub struct ToggleSwitchView {
    value: bool,
    changed: bool,
}

impl View for ToggleSwitchView {
    type Args<'v> = ToggleSwitch<'v>;
    type Response = ToggleResponse;

    fn create(args: Self::Args<'_>, _: &Ui, _: ViewId) -> Self {
        Self {
            value: *args.value,
            changed: false,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, _: &Ui, _: Update) -> Self::Response {
        let changed = self.changed;
        if std::mem::take(&mut self.changed) {
            *args.value = self.value;
        } else if self.value != *args.value {
            self.value = *args.value;
        };
        ToggleResponse { changed }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, event: Event) -> Handled {
        match event.event {
            ViewEvent::MouseClick {
                pos,
                button,
                modifiers,
            } => {
                self.value = !self.value;
                self.changed = true;

                event.manager.add_once(event.current, || {
                    Animation::new()
                        .oneshot(true)
                        .with(easing::sine_in_out)
                        .schedule(Duration::from_millis(150))
                        .unwrap()
                });
            }

            ViewEvent::MouseDragHeld { delta, .. }
                if (self.value && delta.x.is_negative())
                    || (!self.value && delta.x.is_positive()) =>
            {
                self.value = !self.value;
                self.changed = true;

                event.manager.add_once(event.current, || {
                    Animation::new()
                        .oneshot(true)
                        .with(easing::sine_in_out)
                        .schedule(Duration::from_millis(50))
                        .unwrap()
                });
            }

            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, _: Layout, space: Space) -> Size {
        // TODO axis
        // TOOD properties
        space.fit(Size::new(4.0, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();

        let selected = self.value;
        let mut bg = if selected {
            render.theme.primary
        } else {
            render.theme.secondary
        };

        if render.is_hovered() {
            bg = render.theme.accent;
        }

        // TODO properties
        // TODO axis
        let unfilled = Elements::MEDIUM_RECT;
        let pixel = Pixel::new(unfilled).fg(render.theme.surface);
        render.surface.fill_with(pixel);

        let w = rect.width() as f32 - 1.0;

        let x = match render.animations.get_mut(render.current) {
            Some(animation) if selected => lerp(0.0, w, *animation.value),
            Some(animation) if !selected => lerp(w, 0.0, *animation.value),
            _ if selected => w,
            _ => 0.0,
        };

        let filled = Elements::LARGE_RECT;
        let pixel = Pixel::new(filled).fg(bg);
        render.surface.set((x, 0.0), pixel);
    }
}

pub fn dark_mode_switch(ui: &Ui) -> Response {
    // TODO get these from the properties
    const SUN: &str = "☀️";
    const MOON: &str = "🌙";

    let resp = ui.horizontal(|ui| {
        ui.label(if *ui.dark_mode() { MOON } else { SUN });
        ui.toggle_switch(&mut ui.dark_mode());
    });

    if *ui.dark_mode() {
        ui.set_theme(Theme::dark());
    } else {
        ui.set_theme(Theme::light())
    }
    resp
}
