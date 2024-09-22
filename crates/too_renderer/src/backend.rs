use too_math::Vec2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CurrentScreen {
    Main,
    Alt,
}

impl CurrentScreen {
    pub const fn is_main_screen(&self) -> bool {
        matches!(self, Self::Main)
    }

    pub const fn is_alt_screen(&self) -> bool {
        matches!(self, Self::Alt)
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Main => Self::Alt,
            Self::Alt => Self::Main,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[non_exhaustive]
/// Commands are requests sent to the backend
pub enum Command {
    /// Set the title to this string
    SetTitle(String),
    /// Switch to the main screen (e.g. the screen the backend isn't using)
    SwitchMainScreen,
    /// Switch to the alt screen (e.g. the screen the backend is using)
    SwitchAltScreen,
    /// Request the backend to quit
    RequestQuit,
}

impl Command {
    /// Set the title to this string
    pub fn set_title(title: impl ToString) -> Self {
        Self::SetTitle(title.to_string())
    }

    /// Switch to the main screen (e.g. the screen the backend isn't using)
    pub const fn switch_alt_screen() -> Self {
        Self::SwitchAltScreen
    }

    /// Switch to the alt screen (e.g. the screen the backend is using)
    pub const fn switch_main_screen() -> Self {
        Self::SwitchMainScreen
    }

    /// Request the backend to quit
    pub const fn request_quit() -> Self {
        Self::RequestQuit
    }
}

/// An abstraction over a writable backend
pub trait Backend {
    /// The writer for this backend
    type Out: std::io::Write;
    /// The current size of the backends 'screen'
    fn size(&self) -> Vec2;
    /// Can we draw to this backend, currently?
    fn should_draw(&self) -> bool;
    /// Send a [`Command`] to this backend
    fn command(&mut self, cmd: Command);
    /// Get the writer for this backend
    fn writer(&mut self) -> Self::Out;
}
