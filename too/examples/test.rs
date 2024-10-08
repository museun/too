use too::{
    layout::Align2,
    view::{view::Context, views::*, App, Ui},
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Ui::new().run(Test::default(), term)
}

#[derive(Default)]
struct Test {
    style: Style,
    gap: f32,
}

#[derive(PartialEq, Copy, Clone, Default)]
enum Style {
    #[default]
    Empty,
    Thin,
    ThinWide,
    Rounded,
    Double,
    Thick,
    ThickTall,
    ThickWide,
}

impl Test {
    fn draw_borders(ctx: &mut Context<Self>, align2: [Align2; 3], style: Border) {
        List::horizontal().gap(ctx.gap).show(ctx, |ctx| {
            for n in align2 {
                size((20.0, 10.0), ctx, |ctx| {
                    border(ctx, style, |ctx| {
                        align(n, ctx, |ctx| static_label(ctx, "hello"))
                    })
                });
            }
        });
    }
}

impl App for Test {
    fn view(ctx: &mut Context<Self>) {
        let style = match ctx.style {
            Style::Empty => Border::EMPTY,
            Style::Thin => Border::THIN,
            Style::ThinWide => Border::THIN_WIDE,
            Style::Rounded => Border::ROUNDED,
            Style::Double => Border::DOUBLE,
            Style::Thick => Border::THICK,
            Style::ThickTall => Border::THICK_TALL,
            Style::ThickWide => Border::THICK_WIDE,
        };

        List::vertical().gap(ctx.gap).show(ctx, |ctx| {
            for alignment in [
                [
                    Align2::LEFT_TOP, //
                    Align2::CENTER_TOP,
                    Align2::RIGHT_TOP,
                ],
                [
                    Align2::LEFT_CENTER,
                    Align2::CENTER_CENTER,
                    Align2::RIGHT_CENTER,
                ],
                [
                    Align2::LEFT_BOTTOM,
                    Align2::CENTER_BOTTOM,
                    Align2::RIGHT_BOTTOM,
                ],
            ] {
                Self::draw_borders(ctx, alignment, style);
            }
        });

        right_top(ctx, |ctx| {
            column(ctx, |ctx| {
                for (name, style) in [
                    ("Empty", Style::Empty),
                    ("Thin", Style::Thin),
                    ("ThinWide", Style::ThinWide),
                    ("Rounded", Style::Rounded),
                    ("Double", Style::Double),
                    ("Thick", Style::Thick),
                    ("ThickTall", Style::ThickTall),
                    ("ThickWide", Style::ThickWide),
                ] {
                    radio(
                        ctx,
                        style,
                        |ctx| &mut ctx.style,
                        |ctx| static_label(ctx, name),
                    );
                }
            })
        });

        right_bottom(ctx, |ctx| {
            column(ctx, |ctx| {
                slider(ctx, |ctx| SliderParams::new(&mut ctx.gap).range(0.0..=5.0));
                toggle_dark_mode(ctx);
            });
        })
    }
}
