//! Some convenient types and functions

use std::collections::VecDeque;

pub use crate::hasher::{BuildIntHasher, DefaultIntHasher, IntHasher};

/// Converts a long Rust type name into a shorter, more readable one
pub fn short_name(name: &str) -> String {
    const fn is_special(c: char) -> bool {
        matches!(c, ' ' | '<' | '>' | '(' | ')' | '[' | ']' | ',' | ';')
    }

    fn collapse(s: &str) -> &str {
        s.split("::").last().unwrap()
    }

    let mut index = 0;
    let end = name.len();
    let mut out = String::new();

    while index < end {
        let rest = &name[index..end];
        if let Some(mut p) = rest.find(is_special) {
            out.push_str(collapse(&rest[0..p]));

            let ch = &rest[p..=p];
            out.push_str(ch);

            if matches!(ch, ">" | ")" | "]" if rest[p + 1..].starts_with("::")) {
                out.push_str("::");
                p += 2;
            }
            index += p + 1;
        } else {
            out.push_str(collapse(rest));
            index = end;
        }
    }
    out
}

/// A simple bounded queue that rotates its buffer
///
/// This acts as a FIFO queue
#[derive(Default, Debug)]
pub struct Queue<T = String> {
    queue: VecDeque<T>,
    max: usize,
}

impl<T> Queue<T> {
    /// Create a new queue with a max size.
    ///
    /// The size can be zero
    pub const fn new(max: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max,
        }
    }

    /// Get the current max size of the queue
    pub const fn current_size(&self) -> usize {
        self.max
    }

    /// Resizes the queue
    ///
    /// If the queue were to truncate, this'll remove the first N elements
    pub fn resize(&mut self, mut size: usize) {
        let old = std::mem::replace(&mut self.max, size);
        if size >= old {
            return;
        }

        if size == 0 {
            self.clear();
            return;
        }

        while size > old {
            let _ = self.queue.pop_front();
            size -= 1;
        }
    }

    /// Push a value into the queue
    ///
    /// If the number of elements exceeds the max, the first (oldest) element is removed
    pub fn push(&mut self, item: T) {
        if self.max == 0 {
            return;
        }

        while self.queue.len() >= self.max {
            self.queue.pop_front();
        }
        self.queue.push_back(item);
    }

    /// How many elements are in the queue?
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Is this queue empty?
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clears the queue -- this will remove all elements but not change the `max` size
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Drains all elements from the queue
    ///
    /// This is FIFO -- so the order is from oldest to newest
    pub fn drain(&mut self) -> impl ExactSizeIterator<Item = T> + '_ {
        self.queue.drain(..)
    }

    /// Get an iterator for all of the elements in the queue
    ///
    /// This is FIFO -- so the order is from oldest to newest
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> + '_ {
        self.queue.iter()
    }

    /// Get an iterator for all of the elements in the queue, mutably
    ///
    /// This is FIFO -- so the order is from oldest to newest
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> + '_ {
        self.queue.iter_mut()
    }
}
