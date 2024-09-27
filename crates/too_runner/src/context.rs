use std::collections::VecDeque;

use too_math::Vec2;
use too_renderer::Command;

use crate::overlay::Overlay;

/// Provides the ability to send [`Command`] to the backend and access to the [`Overlay`]
pub struct Context<'a> {
    pub(crate) overlay: &'a mut Overlay,
    pub(crate) commands: &'a mut VecDeque<Command>,
    pub(crate) size: Vec2,
}

impl<'a> Context<'a> {
    /// Send a [`Command`] to the backend
    ///
    /// Commands are things like:
    /// * Quit
    /// * Set the title
    pub fn command(&mut self, cmd: Command) {
        self.commands.push_back(cmd);
    }

    /// The overlay
    pub fn overlay(&mut self) -> &mut Overlay {
        self.overlay
    }

    /// Current size of the terminal screen
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Toggle the FPS overlay
    pub fn toggle_fps(&mut self) {
        self.overlay.fps.show = !self.overlay.fps.show
    }
}
