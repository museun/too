use too_events::EventReader;
use too_runner::{Backend, Runner};

use crate::{geom::Size, view::Context, Ui};

pub trait App: Sized + 'static {
    /// [`Context`] derefs into `Self`
    fn view(ctx: &mut Context<'_, Self>);
}

pub trait AppRunner: App {
    fn run(self, backend: impl Backend + EventReader) -> std::io::Result<()> {
        struct Wrapper<T: 'static> {
            ui: Ui<T>,
            app: T,
            view: fn(&mut Context<'_, T>),
        }

        let wrapper = Wrapper {
            ui: Ui::new(Size::from(backend.size())),
            app: self,
            view: Self::view,
        };

        <Runner<Wrapper<Self>, _>>::new()
            .frame_ready(|wrapper| {
                wrapper.ui.scope(&mut wrapper.app, wrapper.view);
            })
            .update(|wrapper, dt, _size| {
                wrapper.ui.tick(dt);
            })
            .event(|wrapper, ev, _ctx, _size| {
                // TODO what to do with `ctx`;
                wrapper.ui.event(&mut wrapper.app, ev);
            })
            .render(|wrapper, surface| {
                wrapper.ui.render(&mut wrapper.app, surface);
            })
            .run(wrapper, backend)
    }
}

impl<T: App> AppRunner for T {}
