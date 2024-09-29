use crate::{
    math::{pos2, Pos2, Vec2},
    Pixel,
};

// TODO this needs a `Patch` trait, or at the very least a patch() method

/// Shapes are drawing primitives, like _fill_ or _line_
pub trait Shape {
    /// This allows you to draw into a local __rect__
    ///
    /// `size` is the total size that you're allowed to draw in
    ///
    /// `put` is how you put a pixel as a specific position
    ///
    /// # Example:
    /// This will fill the entire rect with a specific color
    /// ```rust
    /// # use too::{Pixel, Rgba, Shape, pos2, vec2, Pos2, Vec2, Surface, rect};
    /// # let mut surface = Surface::new(vec2(80, 25));
    /// # let mut surface = surface.crop(rect(vec2(80, 25)));
    /// struct FillBg { bg: Rgba }
    /// impl Shape for FillBg {
    ///     fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
    ///         for y in 0..size.y {
    ///             for x in 0..size.x {
    ///                 put(pos2(x, y), Pixel::new(' ').bg(self.bg))
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// surface.draw(FillBg { bg: too::Rgba::hex("#FFF") });
    ///
    /// ```
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel));
}

impl<T: Shape> Shape for &T {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
        <T as Shape>::draw(self, size, put)
    }
}

impl Shape for () {
    fn draw(&self, _: Vec2, _: impl FnMut(Pos2, Pixel)) {}
}

/// Draw a shape from an anonymous function
///
/// This takes in a function that takes the canvas size and returns a function that returns a pixel for a position
///
/// e.g.
///
/// `fn(Vec2) -> fn(pos) -> maybe pixel`
///
/// ```rust
/// # use too::{anonymous, Pixel, SurfaceMut, Surface, rect, vec2};
/// // equivilant to [`Fill`] with 'red'
/// # let mut surface = Surface::new(vec2(80, 25));
/// # let mut surface = surface.crop(rect(vec2(80, 25)));
/// surface.draw(anonymous(|_size| {
///     move |pos| Some(Pixel::new(' ').bg("#F00"))
/// }));
/// ```
pub fn anonymous<P>(draw: impl Fn(Vec2) -> P) -> impl Shape
where
    P: Fn(Pos2) -> Option<Pixel>,
{
    struct Anonymous<F, P> {
        draw: F,
        _marker: std::marker::PhantomData<P>,
    }

    impl<F, P> Shape for Anonymous<F, P>
    where
        F: Fn(Vec2) -> P,
        P: Fn(Pos2) -> Option<Pixel>,
    {
        fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
            let anon = (self.draw)(size);
            for y in 0..size.y {
                for x in 0..size.x {
                    let pos = pos2(x, y);
                    let Some(pixel) = anon(pos) else {
                        continue;
                    };
                    put(pos, pixel)
                }
            }
        }
    }

    Anonymous {
        draw,
        _marker: std::marker::PhantomData,
    }
}

/// Draw a shape from an anonymous function, with 'context'
///
/// This takes in a function that takes the canvas size and returns a function that returns a pixel for a position with the context passed in.
///
/// e.g.
///
/// `fn(Vec2) -> fn(context, pos) -> maybe pixel`
///
/// ```rust
/// # use too::{anonymous_ctx, Rgba, Color, Pixel, Shape, SurfaceMut, Surface, rect, vec2};
/// # let mut surface = Surface::new(vec2(80, 25));
/// # let mut surface = surface.crop(rect(vec2(80, 25)));
/// // equivilant to [`Fill`] with `color` from 'self'
///
/// struct State {
///     color: Rgba,
/// }
///
/// let state = State {
///     color: Rgba::hex("#F00")
/// };
///
/// surface.draw(anonymous_ctx(&state, |_size| {
///     move |this, pos| Some(Pixel::new(' ').bg(this.color))
/// }));
/// ```
pub fn anonymous_ctx<'a, T, P>(context: &'a T, draw: impl Fn(Vec2) -> P + 'a) -> impl Shape + 'a
where
    P: Fn(&'a T, Pos2) -> Option<Pixel> + 'a,
{
    struct Anonymous<'a, T, F, P> {
        context: &'a T,
        draw: F,
        _marker: std::marker::PhantomData<P>,
    }

    impl<'a, T, F, P> Shape for Anonymous<'a, T, F, P>
    where
        F: Fn(Vec2) -> P + 'a,
        P: Fn(&'a T, Pos2) -> Option<Pixel>,
    {
        fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
            let anon = (self.draw)(size);
            for y in 0..size.y {
                for x in 0..size.x {
                    let pos = pos2(x, y);
                    let Some(pixel) = anon(self.context, pos) else {
                        continue;
                    };
                    put(pos, pixel)
                }
            }
        }
    }

    Anonymous {
        context,
        draw,
        _marker: std::marker::PhantomData,
    }
}

impl Shape for Pixel {
    fn draw(&self, _size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        put(pos2(0, 0), *self)
    }
}
