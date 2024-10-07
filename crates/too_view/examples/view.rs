use too::layout::Align2;
use too_crossterm::{Config, Term};
use too_view::{
    views::{
        align, button, center, column, label, progress_bar, slider, static_label, todo_value,
        toggle, ButtonParams, CrossAlign, LabelParams, List, ProgressBarParams, SliderParams,
    },
    App, AppRunner, Properties,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default().hook_panics(true))?;
    Demo::default().run(Properties::default(), term)
}

#[derive(Default)]
struct Demo {
    okay: bool,
    f: f32,
}

impl App for Demo {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::LEFT_TOP, ctx, |ctx| {
            column(ctx, |ctx| {
                if button(ctx, |_ctx| ButtonParams::new("toggle debug")) {
                    ctx.ui.toggle_debug();
                }
                static_label(ctx, LabelParams::new(format!("okay?: {}", ctx.okay)))
            });
        });

        center(ctx, |ctx| {
            List::vertical()
                .cross_align(CrossAlign::Center)
                .show(ctx, |ctx| {
                    todo_value(
                        ctx,
                        |ctx| &mut ctx.okay,
                        |_ctx| LabelParams::new("click me"),
                    );
                    toggle(ctx, |ctx| &mut ctx.okay);
                    slider(ctx, |ctx| SliderParams::new(&mut ctx.f));
                    progress_bar(ctx, |ctx| ProgressBarParams::new(&ctx.f));
                    label(ctx, |ctx| LabelParams::new(format!("{}", ctx.f)));
                })
        });
    }
}
