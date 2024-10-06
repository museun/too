use crate::{
    geom::{Size, Space},
    LayoutCtx, NoArgs, NoResponse, UpdateCtx, View,
};

mod aligned;
pub use aligned::{align, center};

mod background;
pub use background::background;

mod button;
pub use button::{button, checkbox, radio, selected, todo_value, ButtonParams};

mod constrain;
pub use constrain::{
    constrain, height, max_height, max_size, max_width, min_height, min_size, min_width, size,
    width,
};

mod key_area;
pub use key_area::{hot_key, key_area, key_press, KeyAreaResponse};

mod label;
pub use label::{label, static_label, LabelParams};

mod list;
pub use list::{column, list, row, CrossAlign, ListParams, MainSpacing};

mod margin;
pub use margin::margin;

mod mouse_area;
pub use mouse_area::{
    mouse_area, on_click, on_drag, on_scroll, Dragged, MouseAreaResponse, MouseEvent,
};

mod offset;
pub use offset::offset;

mod progress_bar;
pub use progress_bar::{progress_bar, ProgressBarParams};

mod slider;
pub use slider::{slider, SliderParams};

mod splitter;
pub use splitter::{horizontal_split, split, vertical_split};

// dark / light mode switcher

mod toggle;
pub use toggle::toggle;

pub(crate) struct RootView;
impl<T: 'static> View<T> for RootView {
    type Args<'a> = NoArgs;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self
    }

    fn update(&mut self, _: UpdateCtx<T>, _: Self::Args<'_>) {}

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        ctx.new_layer();
        for &child in ctx.children {
            ctx.compute_layout(child, space);
        }
        space.max
    }
}

pub struct CroppedSurface<'a> {
    rect: too::math::Rect,
    surface: &'a mut too::Surface,
}

impl<'a> too::Canvas for CroppedSurface<'a> {
    fn set(&mut self, pos: too::math::Pos2, cell: impl Into<too::Cell>) {
        self.surface.set(pos + self.rect.left_top(), cell);
    }

    fn fill(&mut self, rect: too::math::Rect, pixel: impl Into<too::Pixel>) -> &mut Self {
        self.surface.fill(rect.intersection(self.rect), pixel);
        self
    }

    fn text<T: too::MeasureText>(
        &mut self,
        rect: too::math::Rect,
        text: impl Into<too::Text<T>>,
    ) -> &mut Self {
        self.surface.text(rect.intersection(self.rect), text);
        self
    }

    fn rect(&self) -> too::math::Rect {
        self.rect
    }
}

pub mod canvas {
    use super::CroppedSurface;
    use crate::{
        geom::{Size, Space},
        view::Context,
        DrawCtx, LayoutCtx, NoResponse, UpdateCtx, View, ViewExt,
    };

    struct Canvas<T: 'static> {
        draw: fn(&mut T, &mut CroppedSurface<'_>),
    }

    impl<T: 'static> View<T> for Canvas<T> {
        type Args<'a> = fn(&mut T, &mut CroppedSurface<'_>);
        type Response = NoResponse;

        fn create(args: Self::Args<'_>) -> Self {
            Self { draw: args }
        }

        fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
            self.draw = args;
        }

        fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
            space.max
        }

        fn draw(&mut self, mut ctx: DrawCtx<T>) {
            let mut surface = CroppedSurface {
                rect: ctx.rect.into(),
                surface: ctx.surface.surface_raw(),
            };
            (self.draw)(ctx.state, &mut surface);
        }
    }

    pub fn canvas<T: 'static>(ctx: &mut Context<T>, draw: fn(&mut T, &mut CroppedSurface<'_>)) {
        Canvas::show(draw, ctx);
    }
}

pub mod animate {
    use too::animation::AnimationManager;

    use crate::{view::Context, AnimateCtx, NoResponse, UpdateCtx, View, ViewExt};

    struct Animate<T: 'static> {
        update: fn(&mut T, f32, &mut AnimationManager),
    }

    impl<T: 'static> View<T> for Animate<T> {
        type Args<'a> = fn(&mut T, f32, &mut AnimationManager);
        type Response = NoResponse;

        fn create(args: Self::Args<'_>) -> Self {
            Self { update: args }
        }

        fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
            self.update = args;
        }

        fn animate(&mut self, mut ctx: AnimateCtx<T>, dt: f32) {
            (self.update)(ctx.state, dt, ctx.too_ctx.animations_mut())
        }
    }

    pub fn animate<T: 'static>(
        ctx: &mut Context<T>,
        update: fn(&mut T, f32, &mut AnimationManager),
    ) {
        Animate::show(update, ctx);
    }
}

pub mod immediate {
    use crate::{
        geom::Space, view::Context, AnimateCtx, DrawCtx, EventCtx, Handled, Interest, LayoutCtx,
        NoResponse, Response, UpdateCtx, View, ViewExt,
    };

    use super::CroppedSurface;

    struct Immediate<T, A>
    where
        T: 'static,
        A: too::App,
    {
        app: fn(&mut T) -> &mut A,
    }

    impl<T: 'static, A> View<T> for Immediate<T, A>
    where
        T: 'static,
        A: too::App,
    {
        type Args<'a> = fn(&mut T) -> &mut A;
        type Response = NoResponse;

        fn create(args: Self::Args<'_>) -> Self {
            Self { app: args }
        }

        fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
            self.app = args
        }

        fn interest(&self) -> Interest {
            Interest::MOUSE | Interest::KEY_INPUT
        }

        fn event(&mut self, mut ctx: EventCtx<T>, event: &crate::Event) -> Handled {
            let Some(ev) = event.translate_to_too() else {
                return Handled::Bubble;
            };

            let app = (self.app)(&mut ctx.state);
            ctx.too_ctx.size = ctx.rect.size().into();
            app.event(ev, ctx.too_ctx);
            Handled::Sink
        }

        fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> crate::geom::Size {
            space.max
        }

        fn animate(&mut self, mut ctx: AnimateCtx<T>, dt: f32) {
            let app = (self.app)(&mut ctx.state);
            app.update(dt, ctx.too_ctx);
        }

        fn draw(&mut self, mut ctx: DrawCtx<T>) {
            let app = (self.app)(&mut ctx.state);
            ctx.too_ctx.size = ctx.rect.size().into();
            let mut surface = CroppedSurface {
                rect: ctx.rect.into(),
                surface: ctx.surface.surface_raw(),
            };
            app.render(&mut surface, ctx.too_ctx);
        }
    }

    pub fn immediate<T, A>(ctx: &mut Context<T>, get_app: fn(&mut T) -> &mut A) -> Response
    where
        T: 'static,
        A: too::App + 'static,
    {
        Immediate::show(get_app, ctx)
    }
}

// float
// flex
// constrained
// unconstrained
// text input
// border
// radio
// checkbox (wip)
// todo value
