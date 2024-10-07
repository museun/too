use too::{overlay::Overlay, Backend, EventReader, Runner};

use crate::{geom::Size, view::Context, Properties, Ui};

pub trait App: Sized + 'static {
    /// [`Context`] derefs into `Self`
    fn view(ctx: &mut Context<Self>);
}

pub trait AppRunner: App + Sealed + Sized {
    fn run(
        self,
        properties: Properties,
        backend: impl Backend + EventReader,
    ) -> std::io::Result<()> {
        struct Wrapper<T: 'static> {
            ui: Ui<T>,
            app: T,
            view: fn(&mut Context<T>),
        }

        let wrapper = Wrapper {
            ui: Ui::new(Size::from(backend.size()), properties),
            app: self,
            view: Self::view,
        };

        <Runner<Wrapper<Self>>>::new()
            .frame_ready(|wrapper, ctx| {
                wrapper.ui.scope(&mut wrapper.app, wrapper.view, ctx);
            })
            .update(|wrapper, dt, ctx| {
                wrapper.ui.animate(&mut wrapper.app, dt, ctx);
            })
            .event(|wrapper, ev, ctx| {
                wrapper.ui.event(&mut wrapper.app, ev, ctx);
            })
            .render(|wrapper, surface, ctx| {
                wrapper.ui.render(&mut wrapper.app, surface, ctx);
            })
            .post_render(|wrapper, overlay, surface| {
                Overlay::default_draw(wrapper, overlay, surface);
            })
            .run(wrapper, backend)
    }
}

impl<T: App + Sealed> AppRunner for T {}

#[doc(hidden)]
pub trait Sealed {}
impl<T> Sealed for T {}
