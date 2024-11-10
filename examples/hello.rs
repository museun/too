use too::{
    layout::{Align, Align2, CrossAlign},
    view::{self, debug, Ui},
    views::{frame, list, slider},
    Border,
};

fn eval_args_run(view: impl FnMut(&Ui)) -> std::io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("-t") => view::debug_view(view),

        #[cfg(not(feature = "profile"))]
        _ => view::run(view),

        #[cfg(feature = "profile")]
        _ => {
            let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
            let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
            profiling::puffin::set_scopes_on(true);
            view::run(view)
        }
    }
}

#[derive(Default)]
struct App {
    w: f32,
    h: f32,
    a: Align,
}

impl App {
    fn view(&mut self, ui: &Ui) {
        debug(format!(
            "mouse pos: {:?} | {:?}",
            ui.cursor_pos(),
            ui.client_rect().size(),
        ));

        ui.aligned(Align2::RIGHT_TOP, |ui| {
            ui.show_children(list().vertical().cross_align(CrossAlign::End), |ui| {
                ui.horizontal(|ui| {
                    ui.show(slider(&mut self.w).range(0.0..=20.0));
                    ui.label(format!("{:.2?}", self.w))
                });
                ui.horizontal(|ui| {
                    ui.show(slider(&mut self.h).range(0.0..=10.0));
                    ui.label(format!("{:.2?}", self.h))
                });

                for (label, align) in [
                    ("Min", Align::Min),
                    ("Center", Align::Center),
                    ("Max", Align::Max),
                ] {
                    ui.radio(align, &mut self.a, label);
                }
            });
        });

        ui.center(|ui| {
            ui.background("#222", |ui| {
                ui.show_children(
                    frame(Border::THICK, "something").title_align(self.a),
                    |ui| {
                        ui.background("#226", |ui| {
                            ui.exact_size((self.w, self.h), |ui| {
                                ui.expand_space();
                            })
                        });
                    },
                );
            });
        });
    }
}

fn main() -> std::io::Result<()> {
    let mut app = App::default();
    app.w = 20.0;
    app.h = 10.0;

    eval_args_run(|ui| app.view(ui))
}
