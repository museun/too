use super::{expand, flex, static_label, toggle, List};
use crate::view::{view::Context, views::toggle::ToggleParams, Theme};

pub fn toggle_dark_mode<T: 'static>(ctx: &mut Context<T>) {
    const SUN: &str = "☀️";
    const MOON: &str = "🌙";

    List::horizontal().gap(0.0).show(ctx, |ctx| {
        expand(ctx, |ctx| {
            let light_mode = ctx.ui.light_mode.clone();
            if toggle(ctx, move |ctx| ToggleParams::shared(light_mode)).changed {
                match ctx.ui.light_mode.get() {
                    true => ctx.ui.set_theme(Theme::light()),
                    false => ctx.ui.set_theme(Theme::dark()),
                };
            }
        });
        flex(ctx, |ctx| {
            let s = if ctx.ui.light_mode.get() { SUN } else { MOON };
            static_label(ctx, s);
        });
    });
}
