use std::collections::VecDeque;

use crate::{animation::AnimationManager, math::Vec2, overlay::Overlay, Command};

/// Provides the ability to send [`Command`] to the backend and access to the [`Overlay`]
pub struct Context<'a> {
    pub(crate) overlay: &'a mut Overlay,
    pub(crate) commands: &'a mut VecDeque<Command>,
    pub(crate) size: Vec2,
    pub(crate) animations: &'a mut AnimationManager,
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

    /// Get the animation manager
    pub fn animations(&self) -> &AnimationManager {
        self.animations
    }

    /// Get the animation manager, mutably
    pub fn animations_mut(&mut self) -> &mut AnimationManager {
        self.animations
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
