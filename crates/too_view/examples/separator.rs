use too_crossterm::{Config, Term};
use too_view::{views::*, App, AppRunner as _, Properties};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Demo.run(Properties::default(), term)
}

struct Demo;
impl App for Demo {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        row(ctx, |ctx| {
            flex(ctx, |ctx| {
                column(ctx, |ctx| {
                    static_label(ctx, "top");
                    horizontal_separator(ctx);
                    static_label(ctx, "bottom");
                });
            });
            flex(ctx, |ctx| {
                vertical_separator(ctx);
            });
            static_label(ctx, "left");
            static_label(ctx, "right");
        });
    }
}
