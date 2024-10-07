use super::{expand, flex, static_label, toggle, LabelParams, List};
use crate::{view::Context, Theme};

pub fn toggle_dark_mode<T: 'static>(ctx: &mut Context<T>, get_light_mode: fn(&mut T) -> &mut bool) {
    const SUN: &str = "☀️";
    const MOON: &str = "🌙";

    List::horizontal().gap(0.0).show(ctx, |ctx| {
        expand(ctx, |ctx| {
            if toggle(ctx, get_light_mode).changed {
                match *get_light_mode(ctx) {
                    true => ctx.ui.set_theme(Theme::light()),
                    false => ctx.ui.set_theme(Theme::dark()),
                };
            }
        });
        flex(ctx, |ctx| {
            let s = if *get_light_mode(ctx) { SUN } else { MOON };
            static_label(ctx, LabelParams::new(s));
        });
    });
}
