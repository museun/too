use too::Rgba;
use too_crossterm::{Config, Term};
use too_view::{views::*, App, AppRunner, Properties};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Colors::default().run(Properties::default(), term)
}

#[derive(Default)]
struct Colors {
    r: f32,
    g: f32,
    b: f32,

    sat: f32,
    dark: f32,
}

impl App for Colors {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        column(ctx, |ctx| {
            expand(ctx, |ctx| {
                List::horizontal().show(ctx, |ctx| {
                    expand(ctx, |ctx| {
                        fill(ctx, Rgba::from_float([ctx.r, ctx.g, ctx.b, 1.0]));
                    });

                    expand(ctx, |ctx| {
                        column(ctx, |ctx| {
                            List::horizontal().gap(1.0).show(ctx, |ctx| {
                                slider(ctx, |ctx| SliderParams::new(&mut ctx.r));
                                flex(ctx, |ctx| static_label(ctx, format!("red: {}", ctx.r)));
                            });
                            List::horizontal().gap(1.0).show(ctx, |ctx| {
                                slider(ctx, |ctx| SliderParams::new(&mut ctx.g));
                                flex(ctx, |ctx| static_label(ctx, format!("green: {}", ctx.g)));
                            });
                            List::horizontal().gap(1.0).show(ctx, |ctx| {
                                slider(ctx, |ctx| SliderParams::new(&mut ctx.b));
                                flex(ctx, |ctx| static_label(ctx, format!("blue: {}", ctx.b)));
                            });
                        });
                    });
                });
            });

            expand(ctx, |ctx| {
                row(ctx, |ctx| {
                    expand(ctx, |ctx| {
                        let color = Rgba::from_float([ctx.r, ctx.g, ctx.b, 1.0])
                            .desaturate(ctx.sat)
                            .darken(ctx.dark);
                        fill(ctx, color)
                    });

                    flex(ctx, |ctx| {
                        column(ctx, |ctx| {
                            List::horizontal().gap(1.0).show(ctx, |ctx| {
                                slider(ctx, |ctx| SliderParams::new(&mut ctx.sat));
                                static_label(ctx, format!("saturation: {}", ctx.sat))
                            });
                            List::horizontal().gap(1.0).show(ctx, |ctx| {
                                slider(ctx, |ctx| SliderParams::new(&mut ctx.dark));
                                static_label(ctx, format!("value: {}", ctx.dark));
                            });
                        });
                    });
                });
            });
        });
    }
}
