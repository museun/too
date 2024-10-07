use too::layout::Align2;
use too_crossterm::{Config, Term};
use too_view::{
    views::{align, center, static_label, toggle_dark_mode},
    App, AppRunner, Properties,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    DarkMode::default().run(Properties::default(), term)
}

#[derive(Default)]
struct DarkMode {
    light_mode: bool,
}

impl App for DarkMode {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::RIGHT_TOP, ctx, |ctx| {
            toggle_dark_mode(ctx, |ctx| &mut ctx.light_mode);
        });

        center(ctx, |ctx| {
            static_label(ctx, format!("light mode? {}", ctx.light_mode));
        });
    }
}
