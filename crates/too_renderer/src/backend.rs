use std::fs::File;

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

#[derive(Debug)]
#[non_exhaustive]
pub enum Command {
    SetTitle(String),
    SwitchMainScreen,
    SwitchAltScreen,
    RequestQuit,
}

impl Command {
    pub fn set_title(title: impl ToString) -> Self {
        Self::SetTitle(title.to_string())
    }

    pub const fn switch_alt_screen() -> Self {
        Self::SwitchAltScreen
    }

    pub const fn switch_main_screen() -> Self {
        Self::SwitchMainScreen
    }

    pub const fn request_quit() -> Self {
        Self::RequestQuit
    }
}

pub trait Backend {
    fn size(&self) -> Vec2;
    fn is_in_alt_screen(&self) -> bool;
    fn command(&mut self, cmd: Command);

    fn file(&mut self) -> File;

    // #[cfg(not(windows))]
    // fn fd(&mut self) -> &mut impl ::std::os::fd::AsFd;

    // #[cfg(windows)]
    // fn handle(&mut self) -> &mut impl ::std::os::windows::io::AsHandle;
}
