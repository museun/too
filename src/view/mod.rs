use std::time::{Duration, Instant};

pub use compact_str::CompactString;

pub mod debug;
pub mod geom;
pub mod helpers;
pub mod views;

mod state;
pub use state::{debug, DebugMode, State, ViewId};

mod response;
pub use response::Response;

mod input;
pub use input::{EventCtx, Handled, Interest, ViewEvent};

mod view;
pub use view::{Builder, View, ViewExt};

mod ui;
pub use ui::Ui;

mod layout;
pub use layout::Layout;

mod render;
pub use render::{CroppedSurface, Render};

mod style;
pub use style::{AxisProperty, Elements, Knob, Styled, Stylesheet, Theme};

pub fn debug_view(mut app: impl FnMut(&Ui)) -> std::io::Result<()> {
    let s = debug::pretty_tree(|ui| app(ui));
    println!("{s}");
    for debug in s.debug() {
        println!("{debug}")
    }
    Ok(())
}

pub fn run<R: 'static>(mut app: impl FnMut(&Ui) -> R) -> std::io::Result<()> {
    use crate::{Backend, EventReader};
    let mut term = crate::term::Term::setup(crate::term::Config::default().hook_panics(true))?;
    let mut surface = crate::Surface::new(term.size());

    let mut state = State::new();
    state.set_debug_mode(DebugMode::Off);

    let target = Duration::from_secs_f32(1.0 / 60.0);
    let max_budget = target / 2;

    let mut prev = Instant::now();

    'outer: loop {
        #[cfg(feature = "profile")]
        {
            profiling::finish_frame!();
        }

        let mut should_render = false;
        let mut last_resize = None;

        let start = Instant::now();
        while let Some(ev) = term.try_read_event() {
            if ev.is_quit() {
                break 'outer;
            }

            if start.elapsed() >= max_budget {
                break;
            }

            if let crate::Event::Resize(size) = ev {
                last_resize = Some(size);
                continue;
            }

            surface.update(&ev);
            state.event(&ev);
            should_render = true;
        }

        if let Some(size) = last_resize {
            let ev = crate::Event::Resize(size);
            surface.update(&ev);
            state.event(&ev);
            should_render = true;
        }

        let now = Instant::now();
        let dt = prev.elapsed();
        state.update(dt.as_secs_f32());
        state.build(surface.rect(), |ui| app(ui));

        if should_render || dt >= target {
            state.render(&mut surface);
            surface.render(&mut term.writer())?;
            prev = now;
        }

        let elapsed = prev.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }

    Ok(())
}
