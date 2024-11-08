use too::{
    view::{
        self,
        views::{list::ScrollStyle, shorthands::list, Constrain, CrossAlign, Fill},
        Elements, Ui, ViewExt,
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
struct App;

impl App {
    fn view(&mut self, ui: &Ui) {}
}

fn main() -> std::io::Result<()> {
    let mut app = App::default();
    eval_args_run(|ui| app.view(ui))
}
