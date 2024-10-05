use std::collections::HashMap;

use super::Animation;
use crate::Index;

/// Immutable reference to an animation state
#[derive(Copy, Clone)]
pub struct AnimationRef<'a> {
    pub animation: &'a Animation,
    pub value: &'a f32,
}

/// Mutable reference to an animation state
pub struct AnimationMut<'a> {
    pub animation: &'a mut Animation,
    pub value: &'a mut f32,
}

/// A manager for dispatching/updating many animations at once.
///
/// This lets you add, retrieve and remove animations from the system
#[derive(Default)]
pub struct AnimationManager {
    animations: HashMap<u64, (Animation, f32), crate::index::BuildIndexHasher>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new animation
    ///
    /// Use the provided key to refer to it
    ///
    /// `initial` is the starting value (in the range of `0.0..=1.0`)
    ///
    /// This returns a previous animation associated with that key
    pub fn add<T: ?Sized>(
        &mut self,
        key: impl Into<Index<T>>,
        animation: Animation,
        initial: f32,
    ) -> Option<Animation> {
        let key = key.into();
        self.animations
            .insert(key.key, (animation, initial))
            .map(|(animation, _)| animation)
    }

    /// Get an immutable reference to the animation and its value
    ///
    /// This panics if the 'key' was not in the manager
    pub fn value<T: ?Sized>(&self, key: impl Into<Index<T>>) -> AnimationRef<'_> {
        self.get(key).unwrap()
    }

    /// Get a mutable reference to the animation and its value
    ///
    /// This panics if the 'key' was not in the manager
    pub fn value_mut<T: ?Sized>(&mut self, key: impl Into<Index<T>>) -> AnimationMut<'_> {
        self.get_mut(key).unwrap()
    }

    /// Tries to get an immutable reference to the animation and its value
    pub fn get<T: ?Sized>(&self, key: impl Into<Index<T>>) -> Option<AnimationRef<'_>> {
        let key = key.into();
        self.animations
            .get(&key.key)
            .map(|(animation, value)| AnimationRef { animation, value })
    }

    /// Tries to get a mutable reference to the animation and its value
    pub fn get_mut<T: ?Sized>(&mut self, key: impl Into<Index<T>>) -> Option<AnimationMut<'_>> {
        let key = key.into();
        self.animations
            .get_mut(&key.key)
            .map(|(animation, value)| AnimationMut { animation, value })
    }

    /// Remove this animation, returning it if it existed
    pub fn remove<T: ?Sized>(&mut self, key: impl Into<Index<T>>) -> Option<Animation> {
        let key = key.into();
        self.animations
            .remove(&key.key)
            .map(|(animation, _)| animation)
    }

    /// Update all animations with this delta-time
    pub fn update(&mut self, dt: f32) {
        for (animation, value) in self.animations.values_mut() {
            *value = animation.update(dt)
        }
    }

    /// Remove all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }
}
