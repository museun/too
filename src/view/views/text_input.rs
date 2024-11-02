use core::f32;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    layout::Axis,
    math::pos2,
    view::{
        geom::{Size, Space},
        Builder, EventCtx, Handled, Interest, Layout, Render, Ui, View, ViewEvent,
    },
    Attribute, Grapheme, Key, Pixel,
};

// TODO multi-line
pub struct TextInput<'a> {
    enabled: bool,
    placeholder: Option<&'a str>,
    initial: Option<&'a str>,
}

impl<'a> TextInput<'a> {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = Some(text);
        self
    }

    pub fn initial(mut self, text: &'a str) -> Self {
        self.initial = Some(text);
        self
    }
}

impl<'v> Builder<'v> for TextInput<'v> {
    type View = InputView;
}

#[derive(Debug, Default)]
pub struct TextInputResponse {
    state: Rc<RefCell<Inner>>,
    submitted: Option<String>,
}

impl TextInputResponse {
    pub fn changed(&self) -> bool {
        self.state.borrow().changed
    }

    pub fn cursor(&self) -> usize {
        self.state.borrow().cursor
    }

    pub fn data(&self) -> Ref<'_, str> {
        let g = self.state.borrow();
        Ref::map(g, |i| &*i.buf)
    }

    pub fn submitted(&self) -> Option<&str> {
        self.submitted.as_deref().filter(|s| !s.is_empty())
    }

    pub fn take_submitted(&mut self) -> Option<String> {
        self.submitted.take().filter(|s| !s.is_empty())
    }

    pub fn selection(&self) -> Option<Ref<'_, str>> {
        let g = self.state.borrow();
        Ref::filter_map(g, |i| i.selection_buffer()).ok()
    }
}

#[derive(Debug)]
pub struct InputView {
    state: InputState,
    enabled: bool,
}

impl View for InputView {
    type Args<'v> = TextInput<'v>;
    type Response = TextInputResponse;

    fn create(args: Self::Args<'_>) -> Self {
        let mut input = Inner::default();

        if let Some(initial) = args.initial {
            input.buf = initial.to_string();
            input.cursor = input.buf.width();
            input.selection = input.cursor;
        };
        input.placeholder = args.placeholder.map(ToString::to_string);

        Self {
            state: InputState {
                inner: Rc::new(RefCell::new(input)),
            },
            enabled: args.enabled,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        self.enabled = args.enabled;

        let mut resp = TextInputResponse {
            state: Rc::clone(&self.state.inner),
            submitted: None,
        };

        let mut g = self.state.inner.borrow_mut();
        if g.submitted {
            resp.submitted = Some(std::mem::take(&mut g.buf));
            g.clear();
        }
        resp
    }

    fn interests(&self) -> Interest {
        Interest::FOCUS_INPUT | Interest::MOUSE_INSIDE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        if !self.enabled {
            return Handled::Bubble;
        }

        let mut state = self.state.inner.borrow_mut();
        state.submitted = false;

        if let ViewEvent::MouseClicked {
            pos, inside: true, ..
        } = event
        {
            let rect = ctx.rect();
            let offset = rect.left() - 1;
            let left = pos.x - offset;

            let diff = rect.width() - state.cursor.min(state.selection) as i32;

            if diff > 0 {
                state.cursor = (left - 1).max(0).min(state.buf.width() as i32) as usize;
            } else {
                let abs = (left - diff).unsigned_abs() as usize;
                state.cursor = abs;
                state.cursor = state.cursor.max(0).min(state.buf.width());
            }

            state.selection = state.cursor;

            return Handled::Sink;
        }

        if let ViewEvent::MouseDrag {
            start,
            current,
            inside: true,
            ..
        } = event
        {
            // TODO `inertia`  (the larger the difference of start.x and current.x are -- the more we scale 't' by)
            let rect = ctx.rect();
            let offset = rect.left() + 1;

            // TODO this has to be relative to the cursor 'fit'
            let mut cursor = start.x - offset;
            let mut selection = current.x - offset;

            let (delta, ..) = Self::fit_cursor(
                &state.buf,
                cursor as usize,
                selection as usize,
                rect.width(),
            );

            cursor += delta;
            selection += delta;

            state.cursor = (cursor as usize).max(0).min(state.buf.width());
            state.selection = (selection as usize).max(0).min(state.buf.width());

            return Handled::Sink;
        }

        let ViewEvent::KeyInput { key, modifiers } = event else {
            return Handled::Bubble;
        };

        let mut buf = [0u8; 4];
        match key {
            Key::Escape => state.cancel_select(),

            Key::Char(ch) if !state.has_selection() && !modifiers.is_ctrl() => {
                state.append(ch.encode_utf8(&mut buf))
            }
            Key::Backspace if modifiers.is_none() && !state.has_selection() => state.delete(-1),
            Key::Delete if modifiers.is_none() && !state.has_selection() => state.delete(1),

            Key::Backspace if !state.has_selection() => state.delete_word(Direction::Backward),
            // ^W
            Key::Char('w') if modifiers.is_ctrl_only() => state.delete_word(Direction::Backward),

            Key::Delete if !state.has_selection() => state.delete_word(Direction::Forward),

            Key::Char(ch) if !modifiers.is_ctrl() => {
                state.overwrite_selection(ch.encode_utf8(&mut buf))
            }
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

            Key::Enter => state.submitted = true,
            _ => return Handled::Bubble,
        }

        Handled::Sink
    }

    fn primary_axis(&self) -> Axis {
        Axis::Horizontal
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        space.fit(Size::new(f32::INFINITY, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        render.surface.fill(render.theme.surface);

        let state = self.state.inner.borrow();
        if state.buf.is_empty() {
            Self::draw_placeholder(self.enabled, &state, &mut render);
            return;
        }

        Self::draw_text(self.enabled, &state, &mut render);
    }
}

impl InputView {
    fn draw_placeholder(enabled: bool, state: &Inner, render: &mut Render) {
        let Some(placeholder) = state.placeholder.as_deref().filter(|c| !c.is_empty()) else {
            Self::draw_cursors(0, state, render);
            return;
        };

        let rect = render.rect();
        let fg = if enabled {
            render.theme.secondary
        } else {
            render.theme.outline
        };

        let w = rect.width();
        let mut start = 0;
        for grapheme in placeholder.graphemes(true) {
            const TRUNCATION: char = '…';
            if (w - start - grapheme.width() as i32) <= 0 {
                render.surface.set(
                    pos2(start, 0),
                    Pixel::new(TRUNCATION).fg(fg).attribute(Attribute::ITALIC),
                );
                break;
            }
            let cell = Grapheme::new(grapheme).fg(fg).attribute(Attribute::ITALIC);
            render.surface.set(pos2(start, 0), cell);
            start += grapheme.len() as i32;
        }

        Self::draw_cursors(0, state, render);
    }

    fn draw_text(enabled: bool, state: &Inner, render: &mut Render) {
        let rect = render.rect();

        let fg = if enabled {
            render.theme.foreground
        } else {
            render.theme.outline
        };

        let (offset, start, end) = Self::fit_cursor(
            &state.buf, //
            state.cursor,
            state.selection,
            rect.width() - 1,
        );

        let mut x = offset;
        for grapheme in state.buf[start..end].graphemes(true) {
            let cell = Grapheme::new(grapheme).fg(fg);
            render.surface.set(pos2(x, 0), cell);
            x += grapheme.width() as i32;
        }

        Self::draw_cursors(offset, state, render);
    }

    fn draw_cursors(offset: i32, state: &Inner, render: &mut Render) {
        let fg = render.theme.primary;

        if state.buf.is_empty() {
            // TODO use the actual colors for this
            let cell = Pixel::new(' ').bg(fg.darken(0.4));
            render.surface.set(pos2(0, 0), cell);
            return;
        }

        // FIXME we have to find the start of the continuation run so we can highlight the main cell
        let cursor = state.cursor as i32 + offset;
        let selection = state.selection as i32 + offset;

        if state.has_selection() {
            for x in selection.min(cursor)..selection.max(cursor) {
                render.surface.patch(pos2(x, 0), |cell| {
                    // TODO use the actual colors for this
                    cell.set_bg(fg.darken(0.1));
                });
            }
            render.surface.patch(pos2(selection, 0), |cell| {
                // TODO use the actual colors for this
                cell.set_bg(fg.darken(0.4));
            });
        } else {
            render.surface.patch(pos2(cursor, 0), |cell| {
                // TODO use the actual colors for this
                cell.set_bg(fg.darken(0.4));
            });
        }
    }

    // TODO this shouldn't scroll if the difference is within some tolerance
    fn fit_cursor(data: &str, cursor: usize, selection: usize, width: i32) -> (i32, usize, usize) {
        let diff = width - selection as i32;
        if diff > 0 {
            let end = cursor.max(selection).max(width as usize);
            return (0, 0, (str_indices::chars::to_byte_idx(data, end)));
        }

        let start = str_indices::chars::to_byte_idx(data, cursor.max(selection) + width as usize);
        (diff.min(0), 0, start.min(data.len()))
    }
}

#[derive(Debug, Default)]
struct InputState {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    buf: String,
    placeholder: Option<String>,
    cursor: usize,    // char indices
    selection: usize, // char indices
    changed: bool,
    submitted: bool,
}

impl Inner {
    const fn has_selection(&self) -> bool {
        self.selection != self.cursor
    }

    fn selection_buffer(&self) -> Option<&str> {
        if !self.has_selection() {
            return None;
        }

        let min = self.cursor.min(self.selection);
        let min = str_indices::chars::to_byte_idx(&self.buf, min);

        let max = self.cursor.max(self.selection);
        let max = str_indices::chars::to_byte_idx(&self.buf, max);

        Some(&self.buf[min..max])
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
        self.selection = 0;
        self.submitted = false;
        self.changed = true;
    }

    fn cancel_select(&mut self) {
        self.cursor = self.selection;
    }

    fn reset_select(&mut self) {
        self.selection = self.cursor;
    }

    fn move_to_end(&mut self) {
        self.cursor = self.buf.width();
        self.reset_select();
    }

    fn move_to_start(&mut self) {
        self.cursor = 0;
        self.reset_select();
    }

    fn append(&mut self, data: &str) {
        let w = data.width();
        if w == 0 {
            return;
        }

        let index = str_indices::chars::to_byte_idx(&self.buf, self.cursor);
        self.buf.insert_str(index, data);

        self.cursor = str_indices::chars::from_byte_idx(&self.buf, index + data.len());
        self.reset_select();

        self.changed = true;
    }

    fn overwrite_selection(&mut self, data: &str) {
        self.delete_selection();
        self.append(data);
    }

    fn move_word(&mut self, dir: Direction) {
        self.cursor = match dir {
            Direction::Forward => {
                WordSep::find_next_word_start(
                    &self.buf, //
                    self.cursor,
                )
                .unwrap_or(self.buf.width())
            }
            Direction::Backward => WordSep::find_prev_word(
                &self.buf, //
                self.cursor,
            )
            .unwrap_or(0),
        };

        self.reset_select();
    }

    // FIXME this has to skip to the end of the grapheme cluster
    fn move_cursor(&mut self, delta: i32) {
        let total = self.buf.width() as i32;
        let mut cursor = self.selection as i32;
        let mut remaining = delta.abs();

        while remaining > 0 {
            cursor = cursor.saturating_add(delta.signum()).clamp(0, total);
            let index = str_indices::chars::to_byte_idx(&self.buf, cursor as usize);
            if self.buf.is_char_boundary(index) {
                remaining -= 1;
            }
        }

        self.selection = cursor as usize;
        if self.has_selection() {
            self.cursor = self.selection;
        }
    }

    fn select(&mut self, delta: i32) {
        self.select_range(self.selection as i32 + delta);
    }

    /// char index
    fn select_range(&mut self, pos: i32) {
        self.selection = pos.max(0).min(self.buf.width() as i32) as usize
    }

    fn select_start(&mut self) {
        self.select_range(0);
    }

    fn select_end(&mut self) {
        self.select_range(self.buf.width() as i32);
    }

    fn select_word(&mut self, dir: Direction) {
        let pos = match dir {
            Direction::Forward => WordSep::find_next_word_start(
                &self.buf,
                str_indices::chars::to_byte_idx(&self.buf, self.selection),
            )
            .unwrap_or(self.buf.len()),

            Direction::Backward => WordSep::find_prev_word(
                &self.buf,
                str_indices::chars::to_byte_idx(&self.buf, self.selection),
            )
            .unwrap_or(0),
        };

        let pos = str_indices::chars::from_byte_idx(&self.buf, pos);
        self.select_range(pos as _);
    }

    fn delete_selection(&mut self) {
        if !self.has_selection() {
            return;
        }

        let start = str_indices::chars::to_byte_idx(&self.buf, self.selection.min(self.cursor));
        let end = str_indices::chars::to_byte_idx(&self.buf, self.selection.max(self.cursor));

        self.buf.replace_range(start..end, "");

        if self.selection < self.cursor {
            self.cursor = self.selection
        }

        self.reset_select();
        self.changed = true;
    }

    fn delete_word(&mut self, dir: Direction) {
        match dir {
            Direction::Forward => {
                let end = WordSep::find_next_word_start(
                    &self.buf, //
                    str_indices::chars::to_byte_idx(&self.buf, self.cursor),
                )
                .unwrap_or(self.buf.len());

                let start = str_indices::chars::to_byte_idx(&self.buf, self.cursor);
                self.buf.replace_range(start..end, "");
            }
            Direction::Backward => {
                let start = WordSep::find_prev_word(
                    &self.buf, //
                    str_indices::chars::to_byte_idx(&self.buf, self.cursor),
                )
                .unwrap_or(0);

                let end = str_indices::chars::to_byte_idx(&self.buf, self.cursor);
                self.buf.replace_range(start..end, "");
                self.cursor = str_indices::chars::from_byte_idx(&self.buf, start);
            }
        };

        self.reset_select();
        self.changed = true;
    }

    fn delete(&mut self, delta: i32) {
        let anchor = str_indices::chars::to_byte_idx(&self.buf, self.cursor);
        let total = self.buf.len() as i32;

        let mut end = anchor as i32;
        let mut remaining = delta.abs();
        let mut len = 0;

        while remaining > 0 {
            end = end.saturating_add(delta.signum()).clamp(0, total);
            len += 1;

            if self.buf.is_char_boundary(end as usize) {
                remaining -= 1;
            }
        }

        if delta < 0 {
            self.cursor = self.cursor.saturating_sub(len);
        }

        let start = anchor.min(end as usize);
        let end = anchor.max(end as usize);

        self.buf.replace_range(start..end, "");

        self.reset_select();
        self.changed = true;
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

    /// byte offset -> byte offset
    fn find_prev_word(data: &str, start: usize) -> Option<usize> {
        let start = str_indices::chars::from_byte_idx(data, start);

        let w = data.len();
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

    /// byte offset -> byte offset
    fn find_next_word_start(data: &str, start: usize) -> Option<usize> {
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

pub const fn text_input<'a>() -> TextInput<'a> {
    TextInput {
        enabled: true,
        placeholder: None,
        initial: None,
    }
}
