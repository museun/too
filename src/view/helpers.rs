use std::collections::VecDeque;

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

#[derive(Default, Debug)]
pub struct Queue<T = String> {
    queue: VecDeque<T>,
    max: usize,
}

impl<T> Queue<T> {
    pub const fn new(max: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max,
        }
    }

    pub const fn current_size(&self) -> usize {
        self.max
    }

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

    pub fn push(&mut self, item: T) {
        if self.max == 0 {
            return;
        }

        while self.queue.len() >= self.max {
            self.queue.pop_front();
        }
        self.queue.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn drain(&mut self) -> impl ExactSizeIterator<Item = T> + '_ {
        self.queue.drain(..)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> + '_ {
        self.queue.iter()
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> + '_ {
        self.queue.iter_mut()
    }
}
