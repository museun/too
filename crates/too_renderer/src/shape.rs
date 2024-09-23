use too_math::{pos2, Pos2, Vec2};

use crate::Pixel;

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
    /// ```rust,no_run
    /// struct FillBg { bg: Rgba }
    /// impl Shape for FillBg {
    ///     fn draw(&self, size: Vec2, mut put: impl Fnmut(Pos2, Pixel)) {
    ///         for y in 0..size.y {
    ///             for x in 0..size.x {
    ///                 put(pos2(x, y), Pixel::new(' ').bg(self.bg))
    ///             }
    ///         }
    ///     }
    /// }
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
/// ```rust,no_run
/// // equivlant to [`Fill`] with 'red'
/// surface.draw(anonymous(|_size| {
///     |this, pos| Some(Pixel::new(' ').bg("#F00"))
/// });
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
/// This takes in a function that takes the canvas size and returns a function that returns a pixel for a position
///
/// e.g.
///
/// ```rust,no_run
/// // equivlant to [`Fill`] with `color` from 'self'
/// surface.draw(anonymous(&self, |_size| {
///     |this, pos| Some(Pixel::new(' ').bg(this.color))
/// });
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
