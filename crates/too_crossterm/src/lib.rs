use std::{collections::VecDeque, thread::JoinHandle};

use too::{
    math::{pos2, vec2, Vec2},
    Backend, Command, CurrentScreen, Event, EventReader, Key, Keybind, Modifiers, MouseButton,
    MouseState, Renderer, TemporalMouseEvent, TermRenderer,
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
/// |[`enable_line_wrap`](Self::enable_line_wrap)|false|
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
    pub(crate) enable_line_wrap: bool,
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

    /// Should we enable line wrapping?
    pub fn enable_line_wrap(mut self, enable_line_wrap: bool) -> Self {
        self.enable_line_wrap = enable_line_wrap;
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
            enable_line_wrap: false,
            hook_panics: false,
            current_screen: CurrentScreen::Alt,
        }
    }
}

/// A terminal handle
pub struct Term {
    _handle: JoinHandle<()>,
    events: flume::Receiver<Event>,
    config: Config,
    out: std::io::StdoutLock<'static>,
    size: Vec2,
    commands: VecDeque<Command>,
}

impl Term {
    pub fn setup(config: Config) -> std::io::Result<Self> {
        use crossterm::terminal::{self, *};

        let mut out = std::io::stdout();
        terminal::enable_raw_mode()?;

        if config.use_alt_screen {
            crossterm::execute!(&mut out, EnterAlternateScreen)?;
        }

        if config.enable_line_wrap {
            crossterm::execute!(&mut out, EnableLineWrap)?;
        } else {
            crossterm::execute!(&mut out, DisableLineWrap)?;
        }

        if config.hide_cursor {
            crossterm::execute!(&mut out, crossterm::cursor::Hide)?;
        }

        if config.mouse_capture {
            crossterm::execute!(&mut out, crossterm::event::EnableMouseCapture)?;
        }

        let size = terminal::size().map(|(w, h)| vec2(w as _, h as _))?;

        if config.hook_panics {
            Self::init_panic_hook();
        }

        let (tx, events) = flume::unbounded();
        Ok(Self {
            _handle: std::thread::spawn(move || read_event(tx)),
            events,
            config,
            out: out.lock(),
            size,
            commands: VecDeque::new(),
        })
    }

    pub fn reset() -> std::io::Result<()> {
        use crossterm::terminal::{self, *};

        let mut out = std::io::stdout();

        // always do these
        crossterm::execute!(&mut out, LeaveAlternateScreen)?;
        crossterm::execute!(&mut out, EnableLineWrap)?;
        crossterm::execute!(&mut out, crossterm::event::DisableMouseCapture)?;
        crossterm::execute!(&mut out, crossterm::cursor::Show)?;

        terminal::disable_raw_mode()
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
    type Out<'a> = &'a mut std::io::StdoutLock<'static>;
    // type Out<'a> = std::fs::File;

    fn size(&self) -> Vec2 {
        self.size
    }

    fn should_draw(&self) -> bool {
        self.config.current_screen.is_alt_screen()
    }

    fn command(&mut self, cmd: Command) {
        self.commands.push_back(cmd);
    }

    fn writer(&mut self) -> Self::Out<'_> {
        &mut self.out
        // #[cfg(windows)]
        // use std::os::windows::io::AsHandle as _;

        // #[cfg(not(windows))]
        // use std::os::fd::AsFd as _;

        // #[cfg(windows)]
        // let owned = self
        //     .out
        //     .as_handle()
        //     .try_clone_to_owned()
        //     .expect("ownable handle");

        // #[cfg(not(windows))]
        // let owned = self.out.as_fd().try_clone_to_owned().expect("ownable fd");

        // std::fs::File::from(owned)
    }
}

impl std::io::Write for Term {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

impl EventReader for Term {
    fn try_read_event(&mut self) -> Option<Event> {
        const CTRL_C: Keybind = Keybind::from_char('c').ctrl();
        const CTRL_Z: Keybind = Keybind::from_char('z').ctrl();

        let mut inplace = None;
        for cmd in std::mem::take(&mut self.commands) {
            match cmd {
                Command::SetTitle(title) => {
                    let _ = TermRenderer::new(self).set_title(&title);
                }
                Command::SwitchMainScreen => {
                    let _ = TermRenderer::new(self).switch_to_main_screen();
                    self.config.current_screen = CurrentScreen::Main;
                    inplace.replace(Event::SwitchMainScreen);
                }
                Command::SwitchAltScreen => {
                    let _ = TermRenderer::new(self).switch_to_alt_screen();
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

        if ev.is_keybind_pressed(CTRL_C) && self.config.ctrl_c_quits {
            return Some(Event::Quit);
        }

        if ev.is_keybind_pressed(CTRL_Z) && self.config.ctrl_z_switches {
            match self.config.current_screen {
                CurrentScreen::Main => {
                    let _ = TermRenderer::new(self).switch_to_alt_screen();
                    self.config.current_screen = CurrentScreen::Alt;
                    return Some(Event::SwitchAltScreen);
                }
                CurrentScreen::Alt => {
                    let _ = TermRenderer::new(self).switch_to_main_screen();
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

fn translate(ev: crossterm::event::Event, mouse_state: &mut MouseState) -> Option<Event> {
    use crossterm::event::{Event as E, KeyCode as K, KeyEventKind, MouseEventKind as M};

    let ev = match ev {
        E::FocusGained => Event::FocusGained,
        E::FocusLost => Event::FocusLost,
        E::Key(ev) => {
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
                _ => return None,
            };

            let modifiers = translate_modifiers(ev.modifiers);

            match ev.kind {
                KeyEventKind::Press => Event::KeyPressed { key, modifiers },
                KeyEventKind::Repeat => Event::KeyRepeat { key, modifiers },
                KeyEventKind::Release => Event::KeyReleased { key, modifiers },
            }
        }
        E::Mouse(ev) => {
            let pos = pos2(ev.column as _, ev.row as _);
            let modifiers = translate_modifiers(ev.modifiers);

            match ev.kind {
                M::Down(button) => {
                    let ev = TemporalMouseEvent::Down(pos, translate_button(button));
                    mouse_state.update(ev, pos, modifiers)?
                }
                M::Up(button) => {
                    let ev = TemporalMouseEvent::Up(pos, translate_button(button));
                    mouse_state.update(ev, pos, modifiers)?
                }
                M::Drag(button) => {
                    let ev = TemporalMouseEvent::Drag(pos, translate_button(button));
                    mouse_state.update(ev, pos, modifiers)?
                }
                M::Moved => Event::MouseMove { pos, modifiers },
                M::ScrollDown => Event::MouseScroll {
                    delta: vec2(0, 1),
                    pos,
                    modifiers,
                },
                M::ScrollUp => Event::MouseScroll {
                    delta: vec2(0, -1),
                    pos,
                    modifiers,
                },
                M::ScrollLeft => Event::MouseScroll {
                    delta: vec2(-1, 0),
                    pos,
                    modifiers,
                },
                M::ScrollRight => Event::MouseScroll {
                    delta: vec2(1, 0),
                    pos,
                    modifiers,
                },
            }
        }
        E::Resize(w, h) => Event::Resize(vec2(w as _, h as _)),
    };

    Some(ev)
}

fn read_event(tx: flume::Sender<Event>) {
    let mut mouse_state = MouseState::default();

    while let Ok(ev) = crossterm::event::read() {
        let Some(ev) = translate(ev, &mut mouse_state) else {
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
