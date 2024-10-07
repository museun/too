use too::{
    layout::Align2,
    math::{Pos2, Rect},
    Rgba,
};
use too_crossterm::{Config, Term};
use too_view::{
    geom::Margin,
    views::{
        align, background, center, column, immediate, label, margin, radio, size, slider,
        static_label, LabelParams, List, SliderParams,
    },
    AppRunner as _, Properties,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;

    let rgba = Rgba::hex("#F00");
    let [r, g, b, _] = rgba.to_float();

    Simple {
        im: Demo {
            rgba,
            ..Demo::default()
        },
        screen: Screen::default(),

        w: 20.0,
        h: 20.0,
        r,
        g,
        b,
    }
    .run(Properties::default(), term)
}

#[derive(PartialEq, Default)]
enum Screen {
    #[default]
    View,
    Immediate,
}

#[derive(Default)]
struct Simple {
    im: Demo,
    screen: Screen,

    w: f32,
    h: f32,

    r: f32,
    g: f32,
    b: f32,
}

impl Simple {
    fn show_view(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::LEFT_TOP, ctx, |ctx| {
            label(ctx, |ctx| {
                LabelParams::new(format!("{:?}", ctx.im.pos)).blink()
            });
        });

        center(ctx, |ctx| {
            size((ctx.w, ctx.h), ctx, |ctx| immediate(ctx, |ctx| &mut ctx.im));
        });

        align(Align2::LEFT_CENTER, ctx, |ctx| {
            column(ctx, |ctx| {
                let r = slider(ctx, |ctx| SliderParams::new(&mut ctx.r).range(0.0..=1.0));
                let g = slider(ctx, |ctx| SliderParams::new(&mut ctx.g).range(0.0..=1.0));
                let b = slider(ctx, |ctx| SliderParams::new(&mut ctx.b).range(0.0..=1.0));

                if r.changed | g.changed | b.changed {
                    ctx.im.rgba = Rgba::from_float([ctx.r, ctx.g, ctx.b, 1.0]);
                }

                background(ctx, ctx.im.rgba, |ctx| {
                    size((5.0, 3.0), ctx, |ctx| static_label(ctx, "asdf"))
                });

                slider(ctx, |ctx| SliderParams::new(&mut ctx.w).range(10.0..=40.0));
                slider(ctx, |ctx| SliderParams::new(&mut ctx.h).range(10.0..=40.0));
            });
        });
    }

    fn show_immediate(ctx: &mut too_view::view::Context<Self>) {
        immediate(ctx, |ctx| &mut ctx.im);
    }
}

impl too_view::App for Simple {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::CENTER_TOP, ctx, |ctx| {
            List::horizontal().gap(3.0).show(ctx, |ctx| {
                for (name, screen) in [("View", Screen::View), ("Immediate", Screen::Immediate)] {
                    radio(
                        ctx,
                        screen,
                        |ctx| &mut ctx.screen,
                        |ctx| static_label(ctx, name),
                    );
                }
            });
        });

        margin(Margin::new(0.0, 1.0, 0.0, 0.0), ctx, |ctx| {
            match ctx.screen {
                Screen::View => Self::show_view(ctx),
                Screen::Immediate => Self::show_immediate(ctx),
            }
        });
    }
}

#[derive(Default)]
struct Demo {
    pos: Pos2,
    rgba: Rgba,
}

impl too::App for Demo {
    fn event(&mut self, event: too::Event, _ctx: too::Context<'_>) {
        if let too::Event::MouseMove { pos, .. } = event {
            self.pos = pos;
        }
    }

    fn render(&mut self, surface: &mut impl too::Canvas, ctx: too::Context<'_>) {
        surface
            .fill(surface.rect(), Rgba::hex("#333"))
            .fill(Rect::from_center_size(self.pos, ctx.size() / 4), self.rgba);
    }
}
