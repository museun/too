use std::time::Duration;

use too::{
    animation::{easing, Animation},
    layout::Align2,
    math::{lerp, pos2},
    Index, Keybind, Pixel,
};
use too_crossterm::{Config, Term};
use too_view::{
    geom::Margin,
    views::{
        align, animate, background, canvas, column, key_area, margin, slider, static_label,
        SliderParams,
    },
    App, AppRunner, Properties,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default().hook_panics(true))?;
    Gradient::new().run(Properties::default(), term)
}

struct Gradient {
    theta: f32,
    skew: f32,
    duration: f32,
    pos: usize,
}

impl App for Gradient {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        Self::draw_animation_container(ctx);
        Self::draw_controls(ctx);
        Self::draw_info(ctx);
    }
}

impl Gradient {
    const LIST: &[(&str, too::Gradient)] = &[
        ("RAINBOW1", too::Gradient::RAINBOW1),
        ("RAINBOW2", too::Gradient::RAINBOW2),
        ("RAINBOW3", too::Gradient::RAINBOW3),
        ("RAINBOW4", too::Gradient::RAINBOW4),
        ("YELLOW_MAGENTA_CYAN", too::Gradient::YELLOW_MAGENTA_CYAN),
        ("ORANGE_BLUE", too::Gradient::ORANGE_BLUE),
        ("GREEN_MAGENTA", too::Gradient::GREEN_MAGENTA),
        ("GREEN_RED", too::Gradient::GREEN_RED),
        ("GREEN_CYAN", too::Gradient::GREEN_CYAN),
        ("YELLOW_RED", too::Gradient::YELLOW_RED),
        ("BLUE_CYAN", too::Gradient::BLUE_CYAN),
        ("RED_BLUE", too::Gradient::RED_BLUE),
        ("YELLOW_GREEN_BLUE", too::Gradient::YELLOW_GREEN_BLUE),
        ("BLUE_WHITE_RED", too::Gradient::BLUE_WHITE_RED),
        ("CYAN_MAGENTA", too::Gradient::CYAN_MAGENTA),
        (
            "YELLOW_PURPLE_MAGENTA",
            too::Gradient::YELLOW_PURPLE_MAGENTA,
        ),
        ("GREEN_BLUE_ORANGE", too::Gradient::GREEN_BLUE_ORANGE),
        ("ORANGE_MAGENTA_BLUE", too::Gradient::ORANGE_MAGENTA_BLUE),
        ("BLUE_MAGENTA_ORANGE", too::Gradient::BLUE_MAGENTA_ORANGE),
        ("MAGENTA_GREEN", too::Gradient::MAGENTA_GREEN),
    ];

    const fn new() -> Self {
        Self {
            pos: 0,
            theta: 0.0,
            skew: 1.0,
            duration: 5.0,
        }
    }
}

impl Gradient {
    const GRADIENT_THETA: Index = Index::namespace("gradient").with("theta");
    const PREVIOUS: Keybind = Keybind::from_char('a');
    const NEXT: Keybind = Keybind::from_char('d');

    fn draw_animation_container(ctx: &mut too_view::view::Context<Self>) {
        animate(ctx, |this, dt, manager| {
            let animation = manager.add_once(Self::GRADIENT_THETA, || {
                let animation = Animation::new()
                    .repeat(true)
                    .round_trip(true)
                    .with(|d| d * d * d)
                    .with(easing::linear)
                    .schedule(Duration::from_secs_f32(this.duration))
                    .unwrap();
                (animation, 0.0)
            });

            this.theta = animation.update(dt).clamp(-3.0, 3.0)
        });

        Self::handle_key_input(ctx);
    }

    fn draw_gradient(ctx: &mut too_view::view::Context<Self>) {
        canvas(ctx, |this, surface| {
            fn normalize(x: i32, y: i32, w: i32, h: i32, factor: f32) -> f32 {
                let x = x as f32 / (w as f32 - 1.0);
                let y = y as f32 / (h as f32 - 1.0);
                lerp(x, y, factor)
            }
            use too::Canvas as _;

            let (_, gradient) = Self::LIST[this.pos];
            let size = surface.rect().size();
            for y in 0..size.y.max(1) {
                for x in 0..size.x {
                    let pos = pos2(x, y);
                    let t = normalize(x, y, size.x, size.y, this.theta * this.skew);
                    let bg = gradient.as_rgba(t);
                    surface.set(pos, Pixel::new(' ').bg(bg));
                }
            }
        });
    }

    fn handle_key_input(ctx: &mut too_view::view::Context<Self>) {
        let resp = key_area(ctx, Self::draw_gradient);
        if resp.is_keybind(Self::PREVIOUS) {
            ctx.pos = (ctx.pos + 1) % Self::LIST.len();
        }
        if resp.is_keybind(Self::NEXT) {
            ctx.pos = ctx.pos.checked_sub(1).unwrap_or(Self::LIST.len() - 1)
        }
    }

    fn draw_duration_slider(ctx: &mut too_view::view::Context<Self>) {
        let resp = slider(ctx, |ctx| {
            SliderParams::new(&mut ctx.duration).range(0.5..=10.0)
        });

        if !resp.changed {
            return;
        }

        let Some(val) = ctx.animations.get_mut(Self::GRADIENT_THETA) else {
            return;
        };

        val.animation
            .reschedule(Duration::from_secs_f32(ctx.state.duration))
            .unwrap();
    }

    fn draw_skew_slider(ctx: &mut too_view::view::Context<Self>) {
        slider(ctx, |ctx| SliderParams::new(&mut ctx.skew).range(0.1..=3.0));
    }

    fn draw_controls_container(ctx: &mut too_view::view::Context<Self>) {
        background(ctx, "#0003", |ctx| {
            margin((1.0, 0.0), ctx, |ctx| {
                column(ctx, |ctx| {
                    Self::draw_duration_slider(ctx);
                    Self::draw_skew_slider(ctx);
                })
            });
        });
    }

    fn draw_controls(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::LEFT_TOP, ctx, |ctx| {
            margin(Margin::new(3.0, 1.0, 0.0, 0.0), ctx, |ctx| {
                Self::draw_controls_container(ctx);
            });
        });
    }

    fn draw_info(ctx: &mut too_view::view::Context<Self>) {
        align(Align2::RIGHT_TOP, ctx, |ctx| {
            background(ctx, "#000", |ctx| {
                column(ctx, |ctx| {
                    static_label(ctx, Self::LIST[ctx.pos].0);
                    static_label(ctx, format!("duration: {:.2?}", ctx.duration))
                })
            });
        });
    }
}
