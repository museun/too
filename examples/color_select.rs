use too::{
    format_str,
    layout::{Anchor2, CrossAlign},
    math::{pos2, remap, Pos2, Rect, Size, Space},
    renderer::{Attribute, Pixel, Rgba},
    view::{Builder, EventCtx, Handled, Interest, Layout, Render, Ui, View, ViewEvent},
    views::{list, slider},
};

fn main() -> std::io::Result<()> {
    let app = |ui: &Ui| ui.show(ColorSelect::default());
    // Ok(println!("{}", too::view::debug::pretty_tree(app)))
    too::application(
        too::RunConfig {
            debug: too::view::DebugMode::Rolling,
            debug_anchor: Anchor2::LEFT_TOP,
            ..Default::default()
        },
        app,
    )
}

#[derive(Debug)]
struct ColorSelect {
    val: f32,
}

impl Default for ColorSelect {
    fn default() -> Self {
        Self { val: 1.0 }
    }
}

impl Builder<'_> for ColorSelect {
    type View = Self;
    type Style = ();
}

impl View for ColorSelect {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn update(&mut self, _args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        ui.show_children(list().horizontal().cross_align(CrossAlign::Stretch), |ui| {
            let color = **ui.expand(|ui| ui.show(color_select(self.val)));

            ui.show_children(list().vertical().cross_align(CrossAlign::Stretch), |ui| {
                ui.label(format_str!("{color:X}"));
                ui.show(slider(&mut self.val));

                ui.expand(|ui| {
                    ui.background(color, |ui| ui.expand_axis());
                });
            });
        });
    }
}

#[derive(Debug)]
struct ColorFill {
    input: Rgba,
    cursor: Pos2,
    val: f32,
}

fn color_select(val: f32) -> ColorFill {
    ColorFill {
        input: Rgba::new(0, 0, 0, 0xFF),
        cursor: Pos2::ZERO,
        val,
    }
}

impl Builder<'_> for ColorFill {
    type View = Self;
    type Style = ();
}

impl View for ColorFill {
    type Args<'v> = Self;
    type Response = Rgba;

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn update(&mut self, args: Self::Args<'_>, _ui: &Ui) -> Self::Response {
        self.val = args.val;
        self.input
    }

    fn interests(&self) -> too::view::Interest {
        Interest::MOUSE_INSIDE | Interest::MOUSE_MOVE
    }

    fn layout(&mut self, _layout: Layout, space: Space) -> Size {
        space.fit(space.max)
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        if let ViewEvent::MouseDrag {
            current,
            inside: true,
            ..
        } = event
        {
            let rect = ctx.rect();
            let rect = Rect::from_min_max(rect.min, rect.max - Pos2::splat(1));
            self.cursor = rect.clamp(current);
            return Handled::Sink;
        }
        Handled::Bubble
    }

    fn draw(&mut self, mut render: Render) {
        let height = render.rect().height();
        let width = render.rect().width();

        let h = height as f32 - 1.0;
        let w = width as f32 - 1.0;

        let xp = remap(self.cursor.x as f32, 0.0..=w, 0.0..=360.0) % 360.0;
        let yp = remap(h - self.cursor.y as f32, 0.0..=h, 0.0..=100.0) / 100.0;

        self.input = Hsv([xp, yp, self.val]).to_rgba();

        for y in 0..height {
            let s = remap(h - y as f32, 0.0..=h, 0.0..=100.0) / 100.0;
            for x in 0..width {
                let h = remap(x as f32, 0.0..=w, 0.0..=360.0) % 360.0;
                let color = Hsv([h, s, 1.0]).to_rgba();
                render.set(pos2(x, y), color);
            }
        }

        render.patch(self.cursor, |cell| {
            *cell = Pixel::new('◯')
                .fg("#000") // TODO contrast ratio
                .bg(cell.bg())
                .attribute(Attribute::BOLD)
                .into();
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct Hsv([f32; 3]);

impl Hsv {
    fn to_rgba(self) -> Rgba {
        let Hsv([h, s, v]) = self;

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Rgba::new(
            ((r + m) * 255.0).round() as u8,
            ((g + m) * 255.0).round() as u8,
            ((b + m) * 255.0).round() as u8,
            0xFF,
        )
    }
}
