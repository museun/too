use std::collections::HashMap;

use super::Animation;

// #[cfg(not(debug_assertions))]
// compile_error!("FIXME wrong id type");
// #[cfg(debug_assertions)]
type Id = u64;

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
    animations: HashMap<Id, (Animation, f32)>,
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
    pub fn add(
        &mut self,
        key: impl Into<Id>,
        animation: Animation,
        initial: f32,
    ) -> Option<Animation> {
        let key = key.into();
        self.animations
            .insert(key, (animation, initial))
            .map(|(animation, _)| animation)
    }

    /// Add this animation, lazily.
    ///
    /// This returns a [`&mut Animation`](Animation)
    ///
    /// If it does not exist for the key it'll be added
    pub fn add_once(
        &mut self,
        key: impl Into<Id>,
        once: impl FnOnce() -> Animation,
    ) -> &mut Animation {
        self.add_once_offset(key, || (once(), 0.0))
    }

    /// Add this animation, lazily, with an offset.
    ///
    /// This returns a [`&mut Animation`](Animation)
    ///
    /// If it does not exist for the key it'll be added
    pub fn add_once_offset(
        &mut self,
        key: impl Into<Id>,
        once: impl FnOnce() -> (Animation, f32),
    ) -> &mut Animation {
        let key = key.into();
        &mut self.animations.entry(key).or_insert_with(once).0
    }

    /// Get an immutable reference to the animation and its value
    ///
    /// This panics if the 'key' was not in the manager
    pub fn value(&self, key: impl Into<Id>) -> AnimationRef<'_> {
        self.get(key).unwrap()
    }

    /// Get a mutable reference to the animation and its value
    ///
    /// This panics if the 'key' was not in the manager
    pub fn value_mut(&mut self, key: impl Into<Id>) -> AnimationMut<'_> {
        self.get_mut(key).unwrap()
    }

    /// Tries to get an immutable reference to the animation and its value
    pub fn get(&self, key: impl Into<Id>) -> Option<AnimationRef<'_>> {
        let key = key.into();
        self.animations
            .get(&key)
            .map(|(animation, value)| AnimationRef { animation, value })
    }

    /// Tries to get a mutable reference to the animation and its value
    pub fn get_mut(&mut self, key: impl Into<Id>) -> Option<AnimationMut<'_>> {
        let key = key.into();
        self.animations
            .get_mut(&key)
            .map(|(animation, value)| AnimationMut { animation, value })
    }

    /// Remove this animation, returning it if it existed
    pub fn remove(&mut self, key: impl Into<Id>) -> Option<Animation> {
        let key = key.into();
        self.animations.remove(&key).map(|(animation, _)| animation)
    }

    /// Update all animations with this delta-time
    pub fn update(&mut self, dt: f32) {
        let mut dead = vec![];
        for (key, (animation, value)) in self.animations.iter_mut() {
            *value = animation.update(dt);
            if animation.is_done() {
                dead.push(*key);
            }
        }

        for dead in dead.drain(..) {
            self.animations.remove(&dead);
        }
    }

    /// Remove all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }
}
