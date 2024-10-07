use too::{
    layout::Anchor2,
    math::{lerp, pos2},
    Gradient, Keybind, Pixel,
};
use too_crossterm::{Config, Term};
use too_view::{
    views::{center, column, immediate, radio, size, slider, static_label, SliderParams},
    AppRunner, Properties,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    NestedUi { im: Demo::new() }.run(Properties::default(), term)
}

struct NestedUi {
    im: Demo,
}

impl too_view::App for NestedUi {
    fn view(ctx: &mut too_view::view::Context<Self>) {
        center(ctx, |ctx| {
            size((50.0, 20.0), ctx, |ctx| {
                immediate(ctx, |ctx| &mut ctx.im);
            })
        });

        column(ctx, |ctx| {
            slider(ctx, |ctx| SliderParams::new(&mut ctx.im.skew));
            slider(ctx, |ctx| {
                SliderParams::new(&mut ctx.im.duration).range(0.1..=5.0)
            });

            for &opt in LIST {
                // TODO radio needs to return a bool if selected
                radio(
                    ctx,
                    opt,
                    |ctx| &mut ctx.im.current,
                    |ctx| static_label(ctx, opt.0),
                );
            }
        });
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Gradients {
    Rainbow1,
    Rainbow2,
    Rainbow3,
    Rainbow4,
    YellowMagentaCyan,
    OrangeBlue,
    GreenMagenta,
    GreenRed,
    GreenCyan,
    YellowRed,
    BlueCyan,
    RedBlue,
    YellowGreenBlue,
    BlueWhiteRed,
    CyanMagenta,
    YellowPurpleMagenta,
    GreenBlueOrange,
    OrangeMagentaBlue,
    BlueMagentaOrange,
    MagentaGreen,
}

impl Gradients {
    fn to_color(&self) -> Gradient {
        match self {
            Gradients::Rainbow1 => Gradient::RAINBOW1,
            Gradients::Rainbow2 => Gradient::RAINBOW2,
            Gradients::Rainbow3 => Gradient::RAINBOW3,
            Gradients::Rainbow4 => Gradient::RAINBOW4,
            Gradients::YellowMagentaCyan => Gradient::YELLOW_MAGENTA_CYAN,
            Gradients::OrangeBlue => Gradient::ORANGE_BLUE,
            Gradients::GreenMagenta => Gradient::GREEN_MAGENTA,
            Gradients::GreenRed => Gradient::GREEN_RED,
            Gradients::GreenCyan => Gradient::GREEN_CYAN,
            Gradients::YellowRed => Gradient::YELLOW_RED,
            Gradients::BlueCyan => Gradient::BLUE_CYAN,
            Gradients::RedBlue => Gradient::RED_BLUE,
            Gradients::YellowGreenBlue => Gradient::YELLOW_GREEN_BLUE,
            Gradients::BlueWhiteRed => Gradient::BLUE_WHITE_RED,
            Gradients::CyanMagenta => Gradient::CYAN_MAGENTA,
            Gradients::YellowPurpleMagenta => Gradient::YELLOW_PURPLE_MAGENTA,
            Gradients::GreenBlueOrange => Gradient::GREEN_BLUE_ORANGE,
            Gradients::OrangeMagentaBlue => Gradient::ORANGE_MAGENTA_BLUE,
            Gradients::BlueMagentaOrange => Gradient::BLUE_MAGENTA_ORANGE,
            Gradients::MagentaGreen => Gradient::MAGENTA_GREEN,
        }
    }
}

const LIST: &[(&str, Gradients)] = &[
    ("Rainbow1", Gradients::Rainbow1),
    ("Rainbow2", Gradients::Rainbow2),
    ("Rainbow3", Gradients::Rainbow3),
    ("Rainbow4", Gradients::Rainbow4),
    ("YellowMagentaCyan", Gradients::YellowMagentaCyan),
    ("OrangeBlue", Gradients::OrangeBlue),
    ("GreenMagenta", Gradients::GreenMagenta),
    ("GreenRed", Gradients::GreenRed),
    ("GreenCyan", Gradients::GreenCyan),
    ("YellowRed", Gradients::YellowRed),
    ("BlueCyan", Gradients::BlueCyan),
    ("RedBlue", Gradients::RedBlue),
    ("YellowGreenBlue", Gradients::YellowGreenBlue),
    ("BlueWhiteRed", Gradients::BlueWhiteRed),
    ("CyanMagenta", Gradients::CyanMagenta),
    ("YellowPurpleMagenta", Gradients::YellowPurpleMagenta),
    ("GreenBlueOrange", Gradients::GreenBlueOrange),
    ("OrangeMagentaBlue", Gradients::OrangeMagentaBlue),
    ("BlueMagentaOrange", Gradients::BlueMagentaOrange),
    ("MagentaGreen", Gradients::MagentaGreen),
];

struct Demo {
    theta: f32,
    skew: f32,
    duration: f32,
    current: (&'static str, Gradients),
    pos: usize,
    up: bool,
}

impl Demo {
    const fn new() -> Self {
        Self {
            current: LIST[0],
            theta: 0.0,
            skew: 1.0,
            duration: 5.0,
            pos: 0,
            up: true,
        }
    }
}
impl too::App for Demo {
    fn event(&mut self, event: too::Event, mut ctx: too::Context<'_>) {
        const NEXT_GRADIENT: Keybind = Keybind::from_char('d');
        const PREV_GRADIENT: Keybind = Keybind::from_char('a');

        const SPEED_UP: Keybind = Keybind::from_char('w');
        const SPEED_DOWN: Keybind = Keybind::from_char('s');

        const SKEW_MORE: Keybind = Keybind::from_char('1');
        const SKEW_LESS: Keybind = Keybind::from_char('2');

        if event.is_keybind_pressed('t') {
            ctx.overlay().fps.anchor = Anchor2::RIGHT_BOTTOM;
            ctx.toggle_fps();
        }

        if event.is_keybind_pressed(SKEW_LESS) {
            self.skew += 0.1;
            self.skew = self.skew.clamp(0.1, 10.0);
        }
        if event.is_keybind_pressed(SKEW_MORE) {
            self.skew -= 0.1;
            self.skew = self.skew.clamp(0.1, 10.0);
        }

        if event.is_keybind_pressed(SPEED_UP) {
            self.duration += 1.0;
            self.duration = self.duration.clamp(1.0, 10.0);
        }
        if event.is_keybind_pressed(SPEED_DOWN) {
            self.duration -= 1.0;
            self.duration = self.duration.clamp(1.0, 10.0);
        }

        if event.is_keybind_pressed(NEXT_GRADIENT) {
            self.pos = (self.pos + 1) % LIST.len();
            self.current = LIST[self.pos];
        }
        if event.is_keybind_pressed(PREV_GRADIENT) {
            self.pos = self.pos.checked_sub(1).unwrap_or(LIST.len() - 1);
            self.current = LIST[self.pos];
        }
    }

    fn update(&mut self, dt: f32, _ctx: too::Context<'_>) {
        self.theta += (self.up as u8 as f32 * 2.0 - 1.0) * self.duration.recip() * dt;
        self.theta = self.theta.clamp(-1.0, 1.0);
        self.up = self.up ^ (self.theta >= 1.0) ^ (self.theta <= -1.0)
    }

    fn render(&mut self, surface: &mut impl too::Canvas, _ctx: too::Context<'_>) {
        fn normalize(x: i32, y: i32, w: i32, h: i32, factor: f32) -> f32 {
            let x = x as f32 / (w as f32 - 1.0);
            let y = y as f32 / (h as f32 - 1.0);
            lerp(x, y, factor)
        }

        let size = surface.rect().size();

        let (name, gradient) = self.current;

        for y in 0..size.y.max(1) {
            for x in 0..size.x {
                let pos = pos2(x, y);
                let t = normalize(x, y, size.x, size.y, self.theta * self.skew);
                let bg = gradient.to_color().as_rgba(t);
                surface.set(pos, Pixel::new(' ').bg(bg));
            }
        }

        let rect = surface.rect();

        let dur = too::Text::new(format!("duration: {:.2?}", self.duration))
            .fg("#FF0")
            .bg("#000");

        let skew = too::Text::new(format!("skew: {:.2?}", self.skew))
            .fg("#FF0")
            .bg("#000")
            .main(too::Justification::Center);

        let name = too::Text::new(name)
            .main(too::Justification::End)
            .fg("#FF0")
            .bg("#000");

        surface.text(rect, dur).text(rect, skew).text(rect, name);
    }
}
