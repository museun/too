use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
};

use super::{EventCtx, Handled, Interest, IntrinsicSize, Layout, Render, Response, Ui, ViewEvent};

/// Builders are required to build and update views
///
/// This is a simple trait that 'binds' two types together -- a cheap builder and a view.
///
/// When the UI creates or updates a view, this builder gets passed to it.
///
/// When the user wants to show a view, they can pass its builder to
/// [`ui.show()`](Ui::show) or [`ui.show_children()`](Ui::show_children)
///
/// ### Note on the `'v` lifetime.
/// This lifetime is provided but you don't have to
/// use it. If you borrow any data from the application, it must conform to this
/// 'v lifetime. Views cannot borrow data as they must be 'static
///
/// ### Builders can also be views
/// ```rust,no_run
/// // views must be `Debug`, builders don't have to be
/// #[derive(Debug)]
/// struct Foo;
///
/// impl<'v> Builder<'v> for Foo {
///     type View = Self; // we cannot borrow anything for 'v because a View must be 'static
/// }
///
/// impl View for Foo {
///     type Args<'v> = Self; // we can build ourselves
///     type Response = ();
///
///     fn create(args: Self::Args<'_>) -> Self {
///         // args here is 'Self'
///         // so we can just return it
///         // but we can also do Foo{} or Self{}
///         args
///     }
///
///     // this method has a default implementation which is the same as below
///     // but we'll implement it for eludication
///     fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
///         // we're passed 'Foo' (the View of the Builder, which happens to be .. us)
///         // so we can just replace ourselves with the new one.
///         //
///         // if your builder has any parameters provided by the user.
///         // you should update yourself here. atleast partially.
///         //
///         // by default, `update` just replaces calls *self = Self::create()
///         *self = args;
///     }
/// }
/// ```
///
/// By convention, you should provide a function that creates your builder with some default parameters.
/// then expose the constructed builder to the user.
///
/// The associated view does not have to be public, but the builder _should_ be public
pub trait Builder<'v>: Sized {
    type View: View<Args<'v> = Self>;
}

/// An extension trait that is implemented for all [`Builder`]s
///
/// This provides a shorthand for chaining `show` and `show_children` on a builder
pub trait ViewExt<'v>: Builder<'v> {
    fn show(self, ui: &Ui) -> Response<<Self::View as View>::Response> {
        ui.show(self)
    }

    fn show_children<R>(
        self,
        ui: &Ui,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(<Self::View as View>::Response, R)>
    where
        R: 'static,
    {
        ui.show_children(self, show)
    }
}

impl<'v, T> ViewExt<'v> for T where T: Builder<'v> {}

#[cfg(not(feature = "sync"))]
pub trait ViewMarker {}
#[cfg(not(feature = "sync"))]
impl<T> ViewMarker for T {}

#[cfg(feature = "sync")]
pub trait ViewMarker: Send + Sync {}
#[cfg(feature = "sync")]
impl<T: Send + Sync> ViewMarker for T {}

/// View is the main trait for describing a view.
///
/// A view is basically a thing that exists in the Ui
///
/// Views must be:
/// - `Debug`
/// - `Sized`
/// - `'static`
///
/// If the `sync` feature is enabled, they must also be `Send` and `Sync`
#[allow(unused_variables)]
pub trait View: Sized + 'static + std::fmt::Debug + ViewMarker {
    /// Arguments for building a view.
    ///
    /// This is typically a [`Builder`] associated with this View
    type Args<'v>;

    /// Response from this view that is given to the user
    ///
    /// This type must be:
    /// - `'static`
    /// - `Default`
    ///
    /// This type gets wrapped in a [`Response`](crate::view::Response) with the views' current id.
    ///
    /// If your view does not return any data to the user, you can simply just use `()`
    type Response: 'static + Default;

    /// Create your view with the args provided from the user
    ///
    /// This method is required for all views.
    fn create(args: Self::Args<'_>) -> Self;

    /// Update your view with the args provided from the user
    ///
    /// ## Ui
    /// This gives you access to the `Ui` which lets you construct other views in-place.
    ///
    /// The views constructed this way will become your children.
    ///
    /// ## Default behavior
    /// By default, this just calls [`View::create`] and returns a [`Default`] [`Response`](crate::view::Response)
    ///
    /// ## NOTE on updating your view state.
    ///
    /// If you do not override this, no state will be persisted across frames.
    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        *self = Self::create(args);
        Self::Response::default()
    }

    /// If your view can be flexible, this returns how flexible it is.
    ///
    /// By default, views are flexible and don't ask for any specific flex space
    fn flex(&self) -> Flex {
        Flex::Loose(0.0)
    }

    /// If you want to receive events, you need to provide some [`Interest`]s
    ///
    /// This is a set of bit flags which lets the runtime filter and dispatch events efficiently.
    ///
    /// By default, views aren't interested in events
    fn interests(&self) -> Interest {
        Interest::NONE
    }

    /// The primary axis of your view.
    ///
    /// This is used as a hint for other views to adjust to your axis
    ///
    /// By default, views are horizontal.
    fn primary_axis(&self) -> Axis {
        Axis::Horizontal
    }

    /// When you provide specific [`Interest`] and an event is processed by the runtime, this method is called.
    ///
    /// [`EventCtx`] allows you to get your children and interact with the input state.
    ///
    /// You must return a [`Handled`] from this. If you intend to consume the
    /// event (e.g.) keep it from propagating, you should return
    /// [`Handled::Sink`] otherwise you should return [`Handled::Bubble`] to
    /// continue the event propagation
    ///
    /// A default event handler is provided for you that dispatches the event to your children, if you have any.
    ///
    /// If you want to handle an event and then continue the default behavior, you can use [`self.default_event(event, ctx)`](View::default_event)
    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        self.default_event(event, ctx)
    }

    /// This is a hint can be used for determining how much space a view will
    /// take up on a specific axis with a specific extent (width/height).
    ///
    /// This should return how much space on that axis this view would like to use
    ///
    /// Note, this is only a hint and isn't part of the layout algorithm. You can get this data for your children with [`Layout::intrinsic_size`]
    ///
    /// By default, this'll gather the maximumze size of your children for the provided parameters
    fn size(&self, intrinsic: IntrinsicSize, axis: Axis, extent: f32) -> f32 {
        let node = intrinsic.nodes.get_current();
        let mut size = 0.0_f32;
        for &child in &node.children {
            size = size.max(intrinsic.size(child, axis, extent))
        }
        size
    }

    /// Lay out your view.
    ///
    /// This gives you a [`Space`] provided by your parent that you can use.
    /// You can use this to determine the [`Size`] your view
    ///
    /// You can get your children from the [`Layout`] type.
    ///
    /// A default layout handler is provided for you that constrains your children to the available space.
    ///
    /// If you want to do some layout calculations and then continue the default behavior, you can use [`self.default_layout(layout, space)`](View::default_layout)
    ///
    /// ## Common usages.
    ///
    /// If you want to take up all the remainder space of your parent:
    /// - return `space.max`
    ///
    /// If you've calculated the total size of your children, you can further constrain this size to fit into the space:
    /// - `space.fit(size)`
    /// - `space.constrain_min(size)`
    ///
    /// with [`Space::fit`] it'll clamp to the size and handle infinite sizes for you
    ///
    /// with [`Space::constrain_min`] it'll produce the smallest size of the compute size and the provided space
    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        self.default_layout(layout, space)
    }

    /// Draw your view
    ///
    /// This gives you a [`Render`] context that is cropped to the size calcualted from [layout](View::layout)
    ///
    /// A default draw handler is provided for you that draws your children for you.
    ///
    /// If you want to do some drawing and then continue the default behavior, you can use [`self.default_draw(render)`](View::default_draw)
    fn draw(&mut self, render: Render) {
        self.default_draw(render)
    }

    /// The default event handling behavior to delegate to your children.
    ///
    /// # NOTE
    /// You should not override this, its provided out of convenience and has a simple, correct implementation.
    ///
    /// Using the implementing here can be useful to determining how you should handle events w/ your children.
    fn default_event(&mut self, event: ViewEvent, mut ctx: EventCtx) -> Handled {
        let node = ctx.nodes.get_current();
        let mut resp = Handled::Bubble;
        for &child in &node.children {
            let new = ctx.send_event(child, event);
            if new.is_sink() {
                return new;
            }
            resp = new;
        }
        resp
    }

    /// The default layout handling behavior to delegate to your children.
    ///
    /// # NOTE
    /// You should not override this, its provided out of convenience and has a simple, correct implementation.
    ///
    /// Using the implementing here can be useful to determining how you should layout your children
    fn default_layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let current = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &current.children {
            size = size.max(layout.compute(child, space))
        }
        size
    }

    /// The default draw handling behavior to delegate to your children.
    ///
    /// # NOTE
    /// You should not override this, its provided out of convenience and has a simple, correct implementation.
    ///
    /// Using the implementing here can be useful to determining how you should draw your children
    fn default_draw(&mut self, mut render: Render) {
        let current = render.nodes.get_current();
        for &child in &current.children {
            render.draw(child)
        }
    }
}
