use std::ops::Add;

use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    math::{pos2, Rect},
    view::{
        geom::{Size, Space},
        EventCtx, EventInterest, Layout, Render, Ui, View, ViewEvent, ViewId,
    },
    Grapheme, Key,
};

// TODO multi-line
pub struct TextInput<'a> {
    enabled: bool,
    ghost: Option<&'a str>,
    initial: Option<&'a str>,
}

impl<'a> TextInput<'a> {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn ghost(mut self, text: &'a str) -> Self {
        self.ghost = Some(text);
        self
    }

    pub fn initial(mut self, text: &'a str) -> Self {
        self.initial = Some(text);
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct InputResponse {
    changed: bool,
}

impl InputResponse {
    pub const fn changed(&self) -> bool {
        self.changed
    }
}

#[derive(Debug)]
pub struct InputView;

impl View for InputView {
    type Args<'v> = TextInput<'v>;
    type Response = InputResponse;

    fn create(args: Self::Args<'_>, ui: &Ui, id: ViewId) -> Self {
        let this = Self;
        let mut state = ui.view_state_mut();

        let input_state = state.get_or_default::<InputState>(id);
        if let Some(initial) = args.initial {
            input_state.buf = initial.to_string();
            input_state.cursor = input_state.buf.width()
        }

        this
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui, id: ViewId, _: Rect) -> Self::Response {
        InputResponse { changed: false }
    }

    fn event_interests(&self) -> EventInterest {
        EventInterest::KEY_INPUT | EventInterest::MOUSE
    }

    fn event(&mut self, event: ViewEvent, rect: Rect, ctx: EventCtx) {
        if !ctx.is_focused() {
            return;
        }

        let state = ctx.view_state.get_or_default::<InputState>(ctx.current);

        if let ViewEvent::MouseDragStart { origin: pos, .. } | ViewEvent::MouseClick { pos, .. } =
            event
        {
            state.cancel_select();
            state.cursor = ((pos.x - rect.left()) as usize).clamp(0, state.buf.width());
        }

        if let ViewEvent::MouseDragHeld {
            pos,
            delta,
            modifiers,
            ..
        } = event
        {
            let mut offset = delta.x;
            if pos.x < rect.left() {
                offset = offset.saturating_sub_unsigned(1)
            } else if pos.x > rect.right() {
                offset = offset.add(1).clamp(0, state.buf.width() as i32)
            }

            state.select(offset);
        }

        if let ViewEvent::KeyInput { key, modifiers } = event {
            match key {
                Key::Escape => state.cancel_select(),

                Key::Char(ch) if !state.has_selection() && !modifiers.is_ctrl() => state.append(ch),
                Key::Backspace if modifiers.is_none() && !state.has_selection() => state.delete(-1),
                Key::Delete if modifiers.is_none() && !state.has_selection() => state.delete(1),

                Key::Backspace if !state.has_selection() => state.delete_word(Direction::Backward),
                // ^W
                Key::Char(w) if modifiers.is_ctrl_only() => state.delete_word(Direction::Backward),

                Key::Delete if !state.has_selection() => state.delete_word(Direction::Forward),

                Key::Char(ch) if !modifiers.is_ctrl() => state.overwrite_selection(ch),
                Key::Backspace | Key::Delete => state.delete_selection(),

                Key::Left if modifiers.is_none() => state.move_cursor(-1),
                Key::Right if modifiers.is_none() => state.move_cursor(1),

                Key::Left if modifiers.is_shift() && modifiers.is_ctrl() => {
                    state.select_word(Direction::Backward)
                }
                Key::Right if modifiers.is_shift() && modifiers.is_ctrl() => {
                    state.select_word(Direction::Forward)
                }

                Key::Left if modifiers.is_shift() => state.select(-1),
                Key::Right if modifiers.is_shift() => state.select(1),

                Key::Left => state.move_word(Direction::Backward),
                Key::Right => state.move_word(Direction::Forward),

                Key::Home if modifiers.is_shift() => state.select_start(),
                Key::End if modifiers.is_shift() => state.select_end(),

                Key::Home => state.move_to_start(),
                Key::End => state.move_to_end(),
                _ => {}
            }
        }
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        // TODO multi-line
        space.fit(Size::new(f32::INFINITY, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        let w = render.rect.width() as usize;

        render.surface.fill(render.theme.surface);

        let fg = if render.is_focused() {
            render.theme.foreground
        } else {
            render.theme.outline
        };

        let state: &mut InputState = render.view_state.get_or_default(render.current);
        state.update_anchor();

        let cursor = state.cursor_pos();
        let selection = state.selection_pos();

        let buffer = state.buffer();

        let mut offset = 0;
        for (i, g) in buffer.grapheme_indices(true) {
            if i == cursor {
                break;
            }
            offset += g.width();
        }

        let offset = if offset >= w {
            offset.saturating_sub(w - 1)
        } else {
            0
        };

        let mut x = 0;

        for (i, grapheme) in buffer.grapheme_indices(true) {
            let gw = grapheme.width();

            if x + gw <= offset {
                x += gw;
                continue;
            }

            if x >= offset && x - offset < w {
                let cell = Grapheme::new(grapheme).fg(fg);
                render.surface.set(((x - offset) as i32, 0), cell);
            }
            x += gw;

            if x - offset >= w {
                break;
            }
        }

        let fg = render.theme.primary;
        if let Some(selection) = selection {
            let fade = render.theme.outline;
            let blend = fg.blend(fade, 0.5);

            let start = selection.min(cursor);
            let end = selection.max(cursor);

            for x in start..end {
                let pos = ((x.saturating_sub(offset)) as i32, 0);
                render.surface.patch(pos, |cell| {
                    cell.set_bg(blend);
                });
            }
            let pos = pos2((selection.saturating_sub(offset)) as i32, 0);
            render.surface.patch(pos, |cell| cell.set_bg(fg));
        } else {
            let pos = pos2((cursor - offset) as i32, 0);
            render.surface.patch(pos, |cell| cell.set_bg(fg));
        }
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    buf: String,
    cursor: usize,
    selection: Option<usize>,
}

impl InputState {
    pub const fn cursor_pos(&self) -> usize {
        self.cursor
    }

    const fn has_selection(&self) -> bool {
        self.selection_pos().is_some()
    }

    pub const fn selection_pos(&self) -> Option<usize> {
        self.selection
    }

    pub fn buffer(&self) -> &str {
        &self.buf
    }

    pub fn clear(&mut self) {
        _ = std::mem::take(self);
    }

    fn update_anchor(&mut self) {
        if self.selection == Some(self.cursor) {
            self.selection.take();
        }
    }

    fn cancel_select(&mut self) {
        let Some(selection) = self.selection.take() else {
            return;
        };
        self.cursor = selection;
    }

    fn move_to_end(&mut self) {
        self.cursor = self.buf.width()
    }

    fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    fn append(&mut self, ch: char) {
        self.buf.insert(self.cursor, ch);
        self.cursor += 1;
    }

    fn overwrite_selection(&mut self, ch: char) {
        self.delete_selection();
        self.append(ch);
    }

    fn move_word(&mut self, dir: Direction) {
        self.cancel_select();

        self.cursor = match dir {
            Direction::Forward => {
                let w = self.buf.width();
                WordSep::find_next_word_start(&self.buf, self.cursor).unwrap_or(w)
            }
            Direction::Backward => WordSep::find_prev_word(&self.buf, self.cursor).unwrap_or(0),
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        self.cancel_select();

        let mut cursor = self.cursor as i32;
        let mut remaining = delta.abs();
        let total = self.buf.width();
        while remaining > 0 {
            cursor = cursor.saturating_add(delta.signum()).clamp(0, total as i32);
            self.cursor = cursor as usize;
            if self.buf.is_char_boundary(self.cursor) {
                remaining -= 1;
            }
        }
    }

    fn select(&mut self, delta: i32) {
        self.select_range(self.selection.unwrap_or(self.cursor) as i32 + delta);
    }

    fn select_range(&mut self, pos: i32) {
        let len = self.buf.width();
        let pos = pos.max(0).min(len as i32);
        if pos == self.cursor as i32 {
            self.selection.take();
        } else {
            self.selection = Some(pos as usize);
        }
    }

    fn select_start(&mut self) {
        self.select_range(0);
    }

    fn select_end(&mut self) {
        self.select_range(self.buf.width() as _);
    }

    fn select_word(&mut self, dir: Direction) {
        let start = self.selection.unwrap_or(self.cursor);
        let pos = match dir {
            Direction::Forward => {
                WordSep::find_next_word_start(
                    &self.buf, //
                    start,
                )
                .unwrap_or(self.buf.width())
            }
            Direction::Backward => WordSep::find_prev_word(
                &self.buf, //
                start,
            )
            .unwrap_or(0),
        };
        self.select_range(pos as _);
    }

    fn delete_selection(&mut self) {
        let Some(selection) = self.selection.take() else {
            return;
        };

        let start = selection.min(self.cursor);
        let end = selection.max(self.cursor);
        self.buf.replace_range(start..end, "");
        self.cursor = if selection < self.cursor {
            selection
        } else {
            self.cursor
        }
    }

    fn delete_word(&mut self, dir: Direction) {
        match dir {
            Direction::Forward => {
                let w = self.buf.width();
                let p = WordSep::find_next_word_start(
                    &self.buf, //
                    self.cursor,
                )
                .unwrap_or(w);
                self.buf.replace_range(self.cursor..p, "");
            }
            Direction::Backward => {
                let p = WordSep::find_prev_word(
                    &self.buf, //
                    self.cursor,
                )
                .unwrap_or(0);
                self.buf.replace_range(p..self.cursor, "");
                self.cursor = p;
            }
        }
    }

    fn delete(&mut self, delta: i32) {
        let anchor = self.cursor as i32;
        let mut end = anchor;
        let mut remaining = delta.abs();
        let mut len = 0;

        let total = self.buf.width();
        while remaining > 0 {
            end = end.saturating_add(delta.signum()).clamp(0, total as i32);
            len += 1;
            if self.buf.is_char_boundary(self.cursor) {
                remaining -= 1;
            }
        }

        if delta < 0 {
            self.cursor = self.cursor.saturating_sub(len);
        }

        let range = anchor.min(end) as usize..anchor.max(end) as usize;
        self.buf.replace_range(range, "");
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Forward,
    Backward,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum WordSep {
    Space,
    Punctuation,
    Other,
}

impl WordSep {
    fn new(data: &str) -> Self {
        if !data.is_ascii() {
            return Self::Other;
        }

        data.chars()
            .next()
            .map(|c| match c {
                c if c.is_ascii_whitespace() => Self::Space,
                c if c.is_ascii_punctuation() => Self::Punctuation,
                _ => Self::Other,
            })
            .unwrap_or(Self::Other)
    }

    fn find_prev_word(data: &str, start: usize) -> Option<usize> {
        let w = data.width();
        let p = data
            .grapheme_indices(true)
            .nth(start)
            .map(|(i, _)| i)
            .unwrap_or(w);

        let mut graphemes = data[..p]
            .grapheme_indices(true)
            .rev()
            .map(|(i, g)| (i, Self::new(g)))
            .peekable();

        while let Some((i, current)) = graphemes.next() {
            let (_, next) = graphemes.peek().copied().unzip();
            if current != Self::Space && next == Some(Self::Space) {
                return Some(i);
            }
        }

        None
    }

    fn find_next_word_start(data: &str, start: usize) -> Option<usize> {
        let w = data.width();
        let mut graphemes = data.grapheme_indices(true).skip(start);
        let mut previous = graphemes.next().map(|(_, c)| Self::new(c))?;
        for (i, g) in graphemes {
            let current = Self::new(g);
            if current == Self::Space && previous != current {
                return Some(i);
            }
            previous = current;
        }
        None
    }
}

// this shouldn't have a provided id. each text_input is unique
// for users to get the state out of it, they should use the TextInput ViewId
pub const fn text_input<'a>() -> TextInput<'a> {
    TextInput {
        enabled: true,
        ghost: None,
        initial: None,
    }
}
