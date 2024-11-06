#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use std::time::{Duration, Instant};

use slotmap::{new_key_type, SlotMap};
use too::{
    layout::Axis,
    math::{pos2, rect, vec2, Pos2, Rect},
    term::{Config, Term},
    view::{
        self,
        views::{button, slider, Border},
    },
    Backend, Event, EventReader, Justification, Surface, Text,
};

fn run(mut app: impl App) -> std::io::Result<()> {
    let mut term = Term::setup(Config::default())?;
    let mut surface = Surface::new(term.size());

    let target = Duration::from_secs_f32(1.0 / 60.0);
    let mut prev = Instant::now();

    loop {
        let mut last_resize = None;

        while let Some(ev) = term.try_read_event() {
            if ev.is_quit() {
                return Ok(());
            }
            if let crate::Event::Resize(size) = ev {
                last_resize = Some(size);
                continue;
            }

            surface.update(&ev);
            app.event(ev);
        }

        if let Some(size) = last_resize {
            let ev = crate::Event::Resize(size);
            surface.update(&ev);
            app.event(ev);
        }

        let now = Instant::now();
        let dt = prev.elapsed();
        if dt >= target {
            app.draw(&mut surface);
            surface.render(&mut term.writer())?;
            prev = now;
        }

        let elapsed = prev.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }
}

fn main() -> std::io::Result<()> {
    let value = 0.5;
    let mut counter = 0;
    view::run(|ui| {
        ui.center(|ui| {
            ui.vertical(|ui| {
                if ui.show(button("click me").disabled(counter > 3)).clicked() {
                    counter += 1;
                }

                ui.label(format!("count: {counter}"));

                if ui.button("reset?").clicked() {
                    counter = 0;
                }
            });
        });
    })
}

trait App {
    fn start(&mut self, area: Rect);
    fn event(&mut self, event: Event);
    fn draw(&mut self, surface: &mut Surface);
}

fn main2() -> std::io::Result<()> {
    run(Hello::default())
}

#[derive(Default)]
struct Hello {
    tiling: Tiling,
    hovered: Option<PaneId>,
}

impl App for Hello {
    fn start(&mut self, area: Rect) {
        self.tiling.resize(area);
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::KeyPressed { key, modifiers } => match key {
                too::Key::Left if modifiers.is_shift() => {
                    self.tiling.swap_in_direction(Direction::Left);
                }
                too::Key::Right if modifiers.is_shift() => {
                    self.tiling.swap_in_direction(Direction::Right);
                }
                too::Key::Up if modifiers.is_shift() => {
                    self.tiling.swap_in_direction(Direction::Up);
                }
                too::Key::Down if modifiers.is_shift() => {
                    self.tiling.swap_in_direction(Direction::Down);
                }

                too::Key::Left => {
                    self.tiling.move_focus(Direction::Left);
                }
                too::Key::Right => {
                    self.tiling.move_focus(Direction::Right);
                }
                too::Key::Up => {
                    self.tiling.move_focus(Direction::Up);
                }
                too::Key::Down => {
                    self.tiling.move_focus(Direction::Down);
                }

                too::Key::Enter => {
                    self.tiling.create_view();
                }

                too::Key::Char('1') => {
                    self.tiling.split(Layout::Horizontal);
                }
                too::Key::Char('2') => {
                    self.tiling.split(Layout::Vertical);
                }

                too::Key::Backspace => {
                    self.tiling.remove(self.tiling.focus);
                }

                too::Key::Tab if modifiers.is_shift() => {
                    self.tiling.focus = self.tiling.find_previous();
                }
                too::Key::Tab => {
                    self.tiling.focus = self.tiling.find_next();
                }

                too::Key::Char('`') => {
                    self.tiling.transpose();
                }

                _ => {}
            },

            Event::MouseMove { pos } => {
                self.hovered = self.tiling.find_pane_by_pos(pos);
            }

            Event::MouseButtonChanged {
                pos, down: true, ..
            } => {
                if let Some(id) = self.tiling.find_pane_by_pos(pos) {
                    self.tiling.focus = id;
                }
            }

            Event::Resize(size) => {
                self.tiling.resize(rect(size));
            }
            _ => {}
        }
    }

    fn draw(&mut self, surface: &mut Surface) {
        surface.clear(surface.rect(), "#223");

        for (i, (view, focused)) in self.tiling.views().enumerate() {
            let hovered = self.hovered == Some(view.id);

            let (border, border_color) = if hovered && focused {
                (Border::DOUBLE, "#0F0")
            } else if focused {
                (Border::THICK, "#F00")
            } else if hovered {
                (Border::THIN, "#555")
            } else {
                (Border::ROUNDED, "#555")
            };
            surface.border(view.area, border, border_color);

            // surface.fill(view.area, Rgba::sine(i as f32 * 1e-1).darken(0.3));

            use slotmap::Key as _;
            surface.text(view.area, {
                Text::new(format!("{id:?}", id = view.id.data()))
                    .fg("#FFF")
                    .main(Justification::Center)
                    .cross(Justification::Center)
            });
        }
    }
}

new_key_type! {
    struct PaneId;
}

#[derive(Debug)]
struct Pane {
    parent: PaneId, //
    content: Content,
}

impl Pane {
    fn container(layout: Layout, parent: PaneId) -> Self {
        Self {
            parent,
            content: Content::Container(Container::new(layout)),
        }
    }

    fn view(view: View, parent: PaneId) -> Self {
        Self {
            parent,
            content: Content::View(view),
        }
    }

    fn id(&self) -> PaneId {
        match &self.content {
            Content::View(view) => view.id,
            Content::Container(..) => self.parent,
        }
    }

    fn parent(&self) -> PaneId {
        self.parent
    }

    fn area(&self) -> Rect {
        match &self.content {
            Content::View(view) => view.area,
            Content::Container(container) => container.area,
        }
    }

    fn get_view(&self) -> Option<&View> {
        let Content::View(view) = &self.content else {
            return None;
        };
        Some(view)
    }

    fn get_view_mut(&mut self) -> Option<&mut View> {
        let Content::View(view) = &mut self.content else {
            return None;
        };
        Some(view)
    }

    fn get_container(&self) -> Option<&Container> {
        let Content::Container(container) = &self.content else {
            return None;
        };
        Some(container)
    }

    fn get_container_mut(&mut self) -> Option<&mut Container> {
        let Content::Container(container) = &mut self.content else {
            return None;
        };
        Some(container)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

type Layout = Axis;

#[derive(Debug)]
enum Content {
    View(View),
    Container(Container),
}

#[derive(Default, Debug)]
struct View {
    id: PaneId,
    area: Rect,
}

impl View {
    const fn new(id: PaneId) -> Self {
        Self {
            id,
            area: Rect::ZERO,
        }
    }
}

#[derive(Debug)]
struct Container {
    area: Rect,
    children: Vec<PaneId>,
    layout: Layout,
}

impl Container {
    const fn new(layout: Layout) -> Self {
        Self {
            area: Rect::ZERO,
            layout,
            children: vec![],
        }
    }

    fn add(&mut self, focus: PaneId, node: PaneId) {
        let pos = self
            .children
            .iter()
            .position(|&child| child == focus)
            .map(|d| d + 1)
            .unwrap_or(0);
        self.children.insert(pos, node);
    }
}

// ui.tiling(|ui, tiling| {
//     // maybe flex(f32) | expand
//     tiling.split(direction, ratio, |ui| {
//          // show ui
//     });
// })
#[derive(Debug)]
struct Tiling {
    nodes: SlotMap<PaneId, Pane>,
    root: PaneId,
    focus: PaneId,
    area: Rect,
    stack: Vec<(PaneId, Rect)>,
}

impl Default for Tiling {
    fn default() -> Self {
        Tiling::new(Rect::ZERO) // will this cause a div by zero?
    }
}

impl Tiling {
    fn new(area: Rect) -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert_with_key(|key| Pane::container(Layout::Vertical, key));

        let mut this = Self {
            nodes,
            root,
            focus: root,
            area,
            stack: Vec::new(),
        };

        // insert a child
        this.create_view();
        this.redistribute();
        this
    }

    fn create_view(&mut self) -> PaneId {
        let focus = self.focus;
        let parent = self.nodes[focus].parent;

        let node = self
            .nodes
            .insert_with_key(|key| Pane::view(View::new(key), parent));

        self.nodes[parent]
            .get_container_mut()
            .unwrap()
            .add(focus, node);

        self.focus = node;
        self.redistribute();
        node
    }

    // why is this returning the new view?
    fn remove(&mut self, id: PaneId) -> bool {
        if id == self.root {
            return false;
        }

        let mut stack = vec![];
        if self.focus == id {
            self.focus = self.find_previous();
        }
        stack.push(id);

        // TODO BFS
        // BUG this is not remotely correct
        while let Some(id) = stack.pop() {
            let parent = self.nodes[id].parent;
            if let Some(container) = self.nodes[parent].get_container_mut() {
                if let Some(index) = container.children.iter().position(|&child| child == id) {
                    container.children.remove(index);
                    if container.children.is_empty() && parent != self.root {
                        stack.push(parent);
                    }
                }
            }

            if let Some(node) = self.nodes[id].get_container() {
                stack.extend_from_slice(&node.children);
            }

            self.nodes.remove(id);
        }

        self.redistribute();
        if self.is_empty() {
            self.create_view();
        }
        true
    }

    fn split(&mut self, layout: Layout) -> PaneId {
        let focus = self.focus;
        let parent = self.nodes[focus].parent;

        let node = self
            .nodes
            .insert_with_key(|node| Pane::view(View::new(node), PaneId::default()));

        let container = self.nodes[parent].get_container_mut().unwrap();
        if container.layout != layout {
            let split = self.nodes.insert(Pane::container(layout, parent));

            self.nodes[split]
                .get_container_mut()
                .unwrap()
                .children
                .extend([focus, node]);

            self.nodes[focus].parent = split;
            self.nodes[node].parent = split;

            *self.nodes[parent]
                .get_container_mut()
                .unwrap()
                .children
                .iter_mut()
                .find(|&&mut child| child == focus)
                .unwrap() = split;
        } else {
            container.add(focus, node);
            self.nodes[node].parent = parent;
        };

        self.focus = node;
        self.redistribute();
        node
    }

    const fn area(&self) -> Rect {
        self.area
    }

    fn get(&self, id: PaneId) -> &View {
        self.try_get(id).unwrap()
    }

    fn try_get(&self, id: PaneId) -> Option<&View> {
        self.nodes.get(id)?.get_view()
    }

    fn get_mut(&mut self, id: PaneId) -> &mut View {
        self.try_get_mut(id).unwrap()
    }

    fn try_get_mut(&mut self, id: PaneId) -> Option<&mut View> {
        self.nodes.get_mut(id)?.get_view_mut()
    }

    fn move_focus(&mut self, direction: Direction) {
        if let Some(next) = self.find_split_in_direction(self.focus, direction) {
            self.focus = next;
        }
    }

    fn find_split_in_direction(&self, id: PaneId, direction: Direction) -> Option<PaneId> {
        use Axis::*;
        use Direction::*;

        let node = &self.nodes[id];
        let parent = node.parent;

        // TODO this seems wrong
        if parent == id {
            return None;
        }

        let container = self.nodes[parent].get_container().unwrap();
        match (direction, container.layout) {
            (Up | Down, Vertical) | (Left | Right, Horizontal) => {
                self.find_split_in_direction(parent, direction)
            }
            (Up | Down, Horizontal) | (Left | Right, Vertical) => {
                let found @ Some(id) = self.find_child(id, &container.children, direction) else {
                    return self.find_split_in_direction(parent, direction);
                };
                found
            }
        }
    }

    fn find_child(&self, id: PaneId, children: &[PaneId], direction: Direction) -> Option<PaneId> {
        let &(mut child) = match direction {
            Direction::Up | Direction::Left => {
                children.iter().rev().skip_while(|&&n| n != id).nth(1)?
            }
            Direction::Down | Direction::Right => {
                children.iter().skip_while(|&&n| n != id).nth(1)?
            }
        };

        let rect = self.nodes[self.focus].get_view().unwrap().area;
        while let Content::Container(container) = &self.nodes[child].content {
            let pos = match container.layout {
                Axis::Horizontal => match direction {
                    Direction::Up => pos2(rect.center().x, rect.top()),
                    Direction::Down => pos2(rect.center().x, rect.bottom()),
                    Direction::Left | Direction::Right => rect.center(),
                },
                Axis::Vertical => match direction {
                    Direction::Left => pos2(rect.left(), rect.center().y),
                    Direction::Right => pos2(rect.right(), rect.center().y),
                    Direction::Up | Direction::Down => rect.center(),
                },
            };

            let child_area = self.nodes[id].area();
            child = *container
                .children
                .iter()
                .min_by_key(|&&id| child_area.distance_sq_to_point(pos).max(1))?
        }

        Some(child)
    }

    fn transpose(&mut self) {
        let focus = self.focus;
        let parent = self.nodes[focus].parent;
        if let Some(container) = self.nodes[parent].get_container_mut() {
            container.layout = -container.layout;
            self.redistribute();
        }
    }

    fn swap_in_direction(&mut self, direction: Direction) -> bool {
        fn swap(this: &mut Tiling, direction: Direction) -> Option<()> {
            use Content::*;
            let focus = this.focus;
            let target = this.find_split_in_direction(focus, direction)?;
            let focus_parent = this.nodes[focus].parent;
            let target_parent = this.nodes[target].parent;

            if focus_parent == target_parent {
                let [Container(parent), View(focus_view), View(target_view)] = this
                    .nodes
                    .get_disjoint_mut([focus_parent, focus, target])?
                    .map(|n| &mut n.content)
                else {
                    unreachable!()
                };

                let focus_pos = parent.children.iter().position(|&id| id == focus_view.id)?;
                let target_pos = parent
                    .children
                    .iter()
                    .position(|&id| id == target_view.id)?;

                parent.children[focus_pos] = target_view.id;
                parent.children[target_pos] = focus_view.id;
                std::mem::swap(&mut focus_view.area, &mut target_view.area);
            } else {
                let [focus_parent, target_parent, focus, target] = this
                    .nodes
                    .get_disjoint_mut([focus_parent, target_parent, focus, target])?;

                let [Container(focus_parent), Container(target_parent), View(focus_view), View(target_view)] = [
                    &mut focus_parent.content,
                    &mut target_parent.content,
                    &mut focus.content,
                    &mut target.content,
                ] else {
                    unreachable!()
                };

                let focus_pos = focus_parent
                    .children
                    .iter()
                    .position(|&id| id == focus_view.id)?;
                let target_pos = focus_parent
                    .children
                    .iter()
                    .position(|&id| id == target_view.id)?;

                std::mem::swap(
                    &mut focus_parent.children[focus_pos],
                    &mut target_parent.children[target_pos],
                );

                std::mem::swap(&mut focus.parent, &mut target.parent);
                std::mem::swap(&mut focus_view.area, &mut target_view.area);
            }

            Some(())
        }

        swap(self, direction).is_some()
    }

    fn find_pane_by_pos(&self, pos: Pos2) -> Option<PaneId> {
        self.views()
            .find_map(|(view, focused)| view.area.contains(pos).then_some(view.id))
    }

    fn find_next(&self) -> PaneId {
        match self
            .traverse_forward()
            .skip_while(|&(id, _)| id != self.focus)
            .nth(1)
        {
            Some((id, _)) => id,
            _ => self.traverse_forward().next().unwrap().0,
        }
    }

    fn find_previous(&self) -> PaneId {
        match self
            .traverse_backward()
            .skip_while(|&(id, _)| id != self.focus)
            .nth(1)
        {
            Some((id, _)) => id,
            _ => self.traverse_backward().next().unwrap().0,
        }
    }

    fn traverse_forward(&self) -> impl Iterator<Item = (PaneId, &View)> + use<'_> {
        let mut stack = vec![self.root];
        std::iter::from_fn(move || loop {
            let id = stack.pop()?;
            let node = &self.nodes[id];
            match &node.content {
                Content::View(view) => return Some((id, view)),
                Content::Container(container) => {
                    stack.extend(container.children.iter());
                }
            }
        })
    }

    fn traverse_backward(&self) -> impl Iterator<Item = (PaneId, &View)> + use<'_> {
        let mut stack = vec![self.root];
        std::iter::from_fn(move || loop {
            let id = stack.pop()?;
            let node = &self.nodes[id];
            match &node.content {
                Content::View(view) => return Some((id, view)),
                Content::Container(container) => {
                    stack.extend(container.children.iter().rev());
                }
            }
        })
    }

    fn views(&self) -> impl Iterator<Item = (&View, bool)> {
        let focus = self.focus;
        self.nodes
            .iter()
            .filter_map(move |(key, node)| Some((node.get_view()?, focus == key)))
    }

    fn views_mut(&mut self) -> impl Iterator<Item = (&mut View, bool)> {
        let focus = self.focus;
        self.nodes
            .iter_mut()
            .filter_map(move |(key, node)| Some((node.get_view_mut()?, focus == key)))
    }

    fn contains(&self, id: PaneId) -> bool {
        self.nodes.contains_key(id)
    }

    fn is_empty(&self) -> bool {
        self.nodes[self.root]
            .get_container()
            .unwrap()
            .children
            .is_empty()
    }

    fn resize(&mut self, area: Rect) -> bool {
        if self.area == area {
            return false;
        }
        self.area = area;
        self.redistribute();
        true
    }

    fn redistribute(&mut self) {
        if self.is_empty() {
            self.focus = self.root;
        }

        self.stack.push((self.root, self.area));

        while let Some((id, area)) = self.stack.pop() {
            let node = &mut self.nodes[id];
            let container = match &mut node.content {
                Content::View(view) => {
                    view.area = area;
                    continue;
                }
                Content::Container(container) => container,
            };

            container.area = area;
            match container.layout {
                // TODO axis::unpack / axis::main
                // TODO remember ratios (and use ratios)
                Axis::Horizontal => {
                    let len = container.children.len() as i32;
                    let height = (area.height() as f32 / len as f32 - 1.0).round() as i32;
                    let mut top = area.top();

                    for (i, &child) in container.children.iter().enumerate() {
                        let i = i as i32;
                        let mut area = Rect::from_min_size(
                            pos2(container.area.left(), top),
                            vec2(container.area.width(), height),
                        );

                        top += height;
                        if i == len - 1 {
                            let height =
                                container.area.top() + container.area.height() - area.top();
                            area.set_size(vec2(area.width(), height));
                        }
                        self.stack.push((child, area));
                    }
                }

                Axis::Vertical => {
                    let len = container.children.len() as i32;
                    let width = (area.width() as f32 / len as f32).round() as i32;
                    let mut left = area.left();

                    for (i, &child) in container.children.iter().enumerate() {
                        let i = i as i32;
                        let mut area = Rect::from_min_size(
                            pos2(left, container.area.top()),
                            vec2(width, container.area.height()),
                        );

                        left += width;
                        if i == len - 1 {
                            let width =
                                container.area.left() + container.area.width() - area.left();
                            area.set_size(vec2(width, area.height()));
                        }
                        self.stack.push((child, area));
                    }
                }
            }
        }
    }
}

impl std::ops::Index<PaneId> for Tiling {
    type Output = Pane;

    fn index(&self, index: PaneId) -> &Self::Output {
        &self.nodes[index]
    }
}

impl std::ops::IndexMut<PaneId> for Tiling {
    fn index_mut(&mut self, index: PaneId) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}
