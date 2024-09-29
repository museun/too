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
