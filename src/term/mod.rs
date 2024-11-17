//! A terminal backend

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, StdoutLock},
    thread::JoinHandle,
};

use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    backend::{Backend, Command, CurrentScreen, Event, EventReader},
    backend::{Key, Keybind, Modifiers, MouseButton},
    math::{pos2, vec2, Vec2},
    renderer::{Renderer as _, TermRenderer},
};

/// Configuration for [`Term`]
///
/// The defaults for this are:
///
/// |option|enabled|
/// |---|---|
/// |[`hide_cursor`](Self::hide_cursor)|true|
/// |[`mouse_capture`](Self::mouse_capture)|true|
/// |[`ctrl_c_quits`](Self::ctrl_c_quits)|true|
/// |[`ctrl_z_switches`](Self::ctrl_z_switches)|false|
/// |[`use_alt_screen`](Self::use_alt_screen)|true|
/// |[`hook_panics`](Self::hook_panics)|false|
///
/// # When using [`too`](https://crates.io/too)
/// You'll likely want to keep most of the defaults.
/// - [`ctrl_c_quits`](Self::ctrl_c_quits) can be set to `false`
/// - [`hook_panics`](Self::hook_panics) can be set to `true`
/// - [`ctrl_z_switches`](Self::ctrl_z_switches) can be set to `true`
///
/// The others should be kept default for unsurprising behavior.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub(crate) hide_cursor: bool,
    pub(crate) mouse_capture: bool,
    pub(crate) ctrl_c_quits: bool,
    pub(crate) ctrl_z_switches: bool,
    pub(crate) use_alt_screen: bool,
    pub(crate) hook_panics: bool,

    current_screen: CurrentScreen,
}

impl Config {
    /// Should we hide the cursor?
    pub fn hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.hide_cursor = hide_cursor;
        self
    }

    /// Should we capture the mouse?
    pub fn mouse_capture(mut self, mouse_capture: bool) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    /// Should pressing `Ctrl-C` signal a quit?
    pub fn ctrl_c_quits(mut self, ctrl_c_quits: bool) -> Self {
        self.ctrl_c_quits = ctrl_c_quits;
        self
    }

    /// Should pressing `Ctrl-Z` switch out of the alternative screen?
    pub fn ctrl_z_switches(mut self, ctrl_z_switches: bool) -> Self {
        self.ctrl_z_switches = ctrl_z_switches;
        self
    }

    /// Should we use an alternative screen
    pub fn use_alt_screen(mut self, use_alt_screen: bool) -> Self {
        self.use_alt_screen = use_alt_screen;
        self.current_screen = CurrentScreen::Alt;
        self
    }

    /// Should we hook into panics?
    pub fn hook_panics(mut self, hook_panics: bool) -> Self {
        self.hook_panics = hook_panics;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hide_cursor: true,
            mouse_capture: true,
            ctrl_c_quits: true,
            ctrl_z_switches: false,
            use_alt_screen: true,
            hook_panics: false,
            current_screen: CurrentScreen::Alt,
        }
    }
}

struct Output {
    #[cfg(windows)]
    mode: u32,
    out: BufWriter<File>,
}

impl Output {
    fn resize(&mut self, hint: usize) {
        let fd = self.out.get_ref().try_clone().unwrap();
        let _ = std::mem::replace(&mut self.out, BufWriter::with_capacity(hint * 21, fd));
    }
}

#[cfg(windows)]
impl Drop for Output {
    fn drop(&mut self) {
        extern "system" {
            fn SetConsoleOutputCP(id: u32) -> i32;
        }
        unsafe { SetConsoleOutputCP(self.mode) };
    }
}

impl Output {
    #[cfg(not(windows))]
    fn new(out: StdoutLock<'static>) -> Self {
        use std::os::fd::AsFd as _;
        let owned = out.as_fd().try_clone_to_owned().expect("ownable fd");
        Self {
            out: BufWriter::new(File::from(owned)),
        }
    }

    #[cfg(windows)]
    fn new(out: StdoutLock<'static>) -> Self {
        use std::os::windows::io::AsHandle as _;

        extern "system" {
            fn SetConsoleOutputCP(id: u32) -> i32;
            fn GetConsoleCP() -> u32;
        }

        let mode = unsafe { GetConsoleCP() };
        let res = unsafe { SetConsoleOutputCP(65001) };
        assert_eq!(res, 1, "cannot switch to the UTF8 codepage");

        let owned = out
            .as_handle()
            .try_clone_to_owned()
            .expect("ownable handle");

        Self {
            mode,
            out: BufWriter::new(File::from(owned)),
        }
    }
}

/// A terminal handle
pub struct Term {
    _handle: JoinHandle<()>,
    events: flume::Receiver<Event>,
    config: Config,
    output: Output,
    _stdout: std::io::StdoutLock<'static>,
    size: Vec2,
    commands: VecDeque<Command>,
}

impl Term {
    pub fn setup(config: Config) -> std::io::Result<Self> {
        let mut out = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;

        if config.use_alt_screen {
            crossterm::execute!(&mut out, EnterAlternateScreen)?;
        }

        crossterm::execute!(&mut out, DisableLineWrap)?;

        if config.hide_cursor {
            crossterm::execute!(&mut out, Hide)?;
        }

        if config.mouse_capture {
            crossterm::execute!(&mut out, EnableMouseCapture)?;
        }

        let size = crossterm::terminal::size().map(|(w, h)| vec2(w as _, h as _))?;

        if config.hook_panics {
            Self::init_panic_hook();
        }

        let (tx, events) = flume::unbounded();
        Ok(Self {
            _handle: std::thread::spawn(move || read_event(tx)),
            events,
            config,
            output: Output::new(out.lock()),
            _stdout: out.lock(),
            size,
            commands: VecDeque::new(),
        })
    }

    fn resize(&mut self, size: Vec2) {
        self.size = size;
        self.output.resize(size.x as usize * size.y as usize);
    }

    pub fn reset() -> std::io::Result<()> {
        let mut out = std::io::stdout();

        // always do these
        crossterm::execute!(&mut out, LeaveAlternateScreen)?;
        crossterm::execute!(&mut out, EnableLineWrap)?;
        crossterm::execute!(&mut out, DisableMouseCapture)?;
        crossterm::execute!(&mut out, Show)?;

        crossterm::terminal::disable_raw_mode()
    }

    pub fn init_panic_hook() {
        let old = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            _ = Self::reset();
            old(info)
        }));
    }
}

impl Backend for Term {
    type Renderer<'a> = TermRenderer<&'a mut BufWriter<File>>;

    fn size(&self) -> Vec2 {
        self.size
    }

    fn should_draw(&self) -> bool {
        self.config.current_screen.is_alt_screen()
    }

    fn command(&mut self, cmd: Command) {
        self.commands.push_back(cmd);
    }

    fn writer(&mut self) -> Self::Renderer<'_> {
        TermRenderer::new(&mut self.output.out)
    }
}

impl std::io::Write for Term {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.out.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.out.flush()
    }
}

impl EventReader for Term {
    fn try_read_event(&mut self) -> Option<Event> {
        const CTRL_C: Keybind = Keybind::from_char('c').ctrl();
        const CTRL_Z: Keybind = Keybind::from_char('z').ctrl();

        let mut inplace = None;
        for cmd in std::mem::take(&mut self.commands) {
            #[allow(unreachable_patterns)]
            match cmd {
                Command::SetTitle(title) => {
                    let _ = TermRenderer::new(&mut *self).set_title(&title);
                }
                Command::SwitchMainScreen => {
                    let _ = TermRenderer::new(&mut *self).switch_to_main_screen();
                    self.config.current_screen = CurrentScreen::Main;
                    inplace.replace(Event::SwitchMainScreen);
                }
                Command::SwitchAltScreen => {
                    let _ = TermRenderer::new(&mut *self).switch_to_alt_screen();
                    self.config.current_screen = CurrentScreen::Alt;
                    inplace.replace(Event::SwitchAltScreen);
                }
                Command::RequestQuit => return Some(Event::Quit),
                _ => {}
            }
        }

        if let Some(ev) = inplace {
            return Some(ev);
        }

        let ev = match self.events.try_recv() {
            Ok(ev) => ev,
            Err(flume::TryRecvError::Disconnected) => return Some(Event::Quit),
            _ => return None,
        };

        if let Event::Resize(size) = ev {
            self.resize(size);
        }

        if ev.is_keybind_pressed(CTRL_C) && self.config.ctrl_c_quits {
            return Some(Event::Quit);
        }

        if ev.is_keybind_pressed(CTRL_Z) && self.config.ctrl_z_switches {
            match self.config.current_screen {
                CurrentScreen::Main => {
                    let _ = TermRenderer::new(&mut *self).switch_to_alt_screen();
                    self.config.current_screen = CurrentScreen::Alt;
                    return Some(Event::SwitchAltScreen);
                }
                CurrentScreen::Alt => {
                    let _ = TermRenderer::new(&mut *self).switch_to_main_screen();
                    self.config.current_screen = CurrentScreen::Main;
                    return Some(Event::SwitchMainScreen);
                }
            }
        }

        Some(ev)
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        _ = Self::reset();
    }
}

fn translate(ev: crossterm::event::Event) -> Option<Event> {
    use crossterm::event::{Event as E, KeyCode as K, KeyEventKind, MouseEventKind as M};

    let ev = match ev {
        E::FocusGained => Event::FocusGained,
        E::FocusLost => Event::FocusLost,
        E::Key(ev) => {
            let mut modifiers = translate_modifiers(ev.modifiers);

            let key = match ev.code {
                K::Char(char) => Key::Char(char),
                K::F(func) => Key::Function(func),
                K::Left => Key::Left,
                K::Right => Key::Right,
                K::Up => Key::Up,
                K::Down => Key::Down,
                K::PageUp => Key::PageUp,
                K::PageDown => Key::PageDown,
                K::Home => Key::Home,
                K::End => Key::End,
                K::Insert => Key::Insert,
                K::Enter => Key::Enter,
                K::Delete => Key::Delete,
                K::Backspace => Key::Backspace,
                K::Esc => Key::Escape,
                K::Tab => Key::Tab,
                K::BackTab => {
                    modifiers |= Modifiers::SHIFT;
                    Key::Tab
                }
                _ => return None,
            };

            match ev.kind {
                KeyEventKind::Press => Event::KeyPressed { key, modifiers },
                _ => return None,
            }
        }
        E::Mouse(ev) => {
            let pos = pos2(ev.column as _, ev.row as _);
            let modifiers = translate_modifiers(ev.modifiers);

            match ev.kind {
                M::Down(button) => Event::MouseButtonChanged {
                    pos,
                    button: translate_button(button),
                    down: true,
                    modifiers,
                },
                M::Drag(button) => Event::MouseDrag {
                    pos,
                    button: translate_button(button),
                    modifiers,
                },
                M::Up(button) => Event::MouseButtonChanged {
                    pos,
                    button: translate_button(button),
                    down: false,
                    modifiers,
                },
                M::Moved => Event::MouseMove { pos },
                M::ScrollDown => Event::MouseScroll {
                    delta: vec2(0, 1),
                    modifiers,
                },
                M::ScrollUp => Event::MouseScroll {
                    delta: vec2(0, -1),
                    modifiers,
                },
                M::ScrollLeft => Event::MouseScroll {
                    delta: vec2(-1, 0),
                    modifiers,
                },
                M::ScrollRight => Event::MouseScroll {
                    delta: vec2(1, 0),
                    modifiers,
                },
            }
        }
        E::Resize(w, h) => Event::Resize(vec2(w as _, h as _)),
    };

    Some(ev)
}

fn read_event(tx: flume::Sender<Event>) {
    while let Ok(ev) = crossterm::event::read() {
        let Some(ev) = translate(ev) else {
            continue;
        };

        if tx.send(ev).is_err() {
            break;
        }
    }

    let _ = tx.send(Event::Quit);
}

fn translate_button(value: crossterm::event::MouseButton) -> MouseButton {
    match value {
        crossterm::event::MouseButton::Left => MouseButton::Primary,
        crossterm::event::MouseButton::Right => MouseButton::Secondary,
        crossterm::event::MouseButton::Middle => MouseButton::Middle,
    }
}

fn translate_modifiers(value: crossterm::event::KeyModifiers) -> Modifiers {
    [
        crossterm::event::KeyModifiers::SHIFT,
        crossterm::event::KeyModifiers::CONTROL,
        crossterm::event::KeyModifiers::ALT,
    ]
    .into_iter()
    .fold(Modifiers::NONE, |this, m| {
        this | Modifiers((value & m).bits())
    })
}
