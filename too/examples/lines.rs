use too::{
    view::{
        geom::{float_step_inclusive, Margin, Point},
        view::Context,
        views::*,
        App, Event, MouseDragHeld, MouseDragRelease, MouseDragStart, Ui,
    },
    Rgba,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Ui::new().run(Lines::default(), term)
}

#[derive(Default)]
struct Lines {
    lines: Vec<Line>,
}

impl App for Lines {
    fn view(ctx: &mut Context<Self>) {
        row(ctx, |ctx| {
            Self::draw_line_list(ctx);
            vertical_separator(ctx);

            // here
            let resp = event_area(ctx, |ctx| {
                Self::render_lines(ctx);
            });
            if let Some(ev) = resp {
                Self::handle_events(ctx, ev);
            }
        });
    }
}

impl Lines {
    fn draw_line_list(ctx: &mut Context<Self>) {
        min_width(20.0, ctx, |ctx| {
            margin(Margin::symmetric(1.0, 0.0), ctx, |ctx| {
                column(ctx, |ctx| {
                    for i in 0..ctx.lines.len() {
                        Self::draw_line_label(ctx, i);
                    }
                });
            });
        });
    }

    fn draw_line_label(ctx: &mut Context<Self>, index: usize) {
        label(ctx, move |ctx| {
            // FIXME this is awful
            LabelParams::new(ctx.lines.get(index).map(|line| {
                LabelOptions::new(format!(
                    "#{index}: {},{}..{},{}",
                    line.start.x, line.start.y, line.end.x, line.end.y
                ))
                .fg(line.color)
            }))
        });
    }

    fn render_lines(ctx: &mut Context<Self>) {
        canvas(ctx, |this, surface| {
            for line in &this.lines {
                let Point { x: sx, y: sy } = line.start;
                let Point { x: ex, y: ey } = line.end;

                let xd = sx.max(ex) - sx.min(ex);
                let yd = sy.max(ey) - sy.min(ey);

                let dx = if sx <= ex { 1.0 } else { -1.0 };
                let dy = if sy <= ey { 1.0 } else { -1.0 };

                let slope = xd.max(yd);

                for i in float_step_inclusive(0.0, slope, 1.0) {
                    let mut x = sx;
                    let mut y = sy;
                    if xd != 0.0 {
                        x += ((i * xd) / slope) * dx;
                    }
                    if yd != 0.0 {
                        y += ((i * yd) / slope) * dy;
                    }
                    surface.set((x, y), line.color);
                }
            }
        });
    }

    // TODO find a better way of doing this
    fn handle_events(ctx: &mut Context<Self>, ev: Event) {
        if ev.is_keybind('r') {
            ctx.lines.clear();
        }
        if ev.is_keybind('d') {
            ctx.lines.pop();
        }

        if let Event::MouseDragStart(MouseDragStart { origin, .. }) = ev {
            let value = Line {
                start: origin,
                end: origin,
                color: Rgba::sine(ctx.lines.len() as f32 * 0.05),
            };
            ctx.lines.push(value);
        }

        if let Event::MouseDragHeld(MouseDragHeld { pos, .. })
        | Event::MouseDragRelease(MouseDragRelease { pos, .. }) = ev
        {
            if let Some(last) = ctx.lines.last_mut() {
                last.end = pos;
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Line {
    start: Point,
    end: Point,
    color: Rgba,
}
