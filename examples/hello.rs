use too::{
    view::{
        self,
        views::{shorthands::list, Constrain, CrossAlign},
        Ui, ViewExt,
    },
    Rgba,
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
    list: Vec<(usize, String)>,
}

impl App {
    fn view(&mut self, ui: &Ui) {
        ui.center(|ui| {
            ui.background("#333", |ui| {
                ui.constrain(Constrain::exact_size((20, 10)), |ui| {
                    let resp = list()
                        .vertical()
                        .cross_align(CrossAlign::Fill)
                        .scrollable(true)
                        .show_children(ui, |ui| {
                            for (i, (h, _s)) in self.list.iter().enumerate() {
                                ui.background(Rgba::sine(i as f32 * 1e-1), |ui| {
                                    ui.constrain(Constrain::exact_height(*h as i32), |ui| {
                                        // ui.horizontal(|ui| {
                                        ui.label(i);
                                        // ui.label(s);
                                        // });
                                    });
                                });
                            }
                        });
                    ui.set_focus(resp.id());
                });
            });
        });
    }
}

fn main() -> std::io::Result<()> {
    let mut app = App::default();
    app.list = std::iter::repeat_with(|| {
        let s = std::iter::repeat_with(fastrand::alphabetic)
            .take(10)
            .collect::<String>();
        (fastrand::usize(2..5), s)
    })
    .take(30)
    .collect();

    eval_args_run(|ui| app.view(ui))
}
