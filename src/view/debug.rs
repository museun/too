use compact_str::CompactString;
use slotmap::Key;
use unicode_width::UnicodeWidthStr;

use crate::{
    layout::{Align, Flex},
    math::{rect, vec2, Rect},
    Event, Surface,
};

use super::{
    helpers::short_name,
    state::{Debug, LayoutNodes, ViewNodes},
    Interest, State, Ui, ViewId,
};

#[derive(Debug)]
pub struct DebugNode {
    id: ViewId,
    name: String,
    debug: Vec<String>,
    rect: Rect,
    flex: Flex,
    interest: Interest,
    children: Vec<Self>,
}

impl DebugNode {
    pub fn from_state(state: &State) -> Self {
        DebugNode::new(state.root(), &state.nodes, &state.layout)
    }

    pub fn compact_tree(&self) -> String {
        render_compact_tree(self)
    }

    pub fn pretty_tree(&self) -> String {
        render_pretty_tree(self)
    }

    pub fn new(root: ViewId, nodes: &ViewNodes, layout: &LayoutNodes) -> Self {
        fn build(
            debug_nodes: &mut Vec<DebugNode>,
            id: ViewId,
            nodes: &ViewNodes,
            layout: &LayoutNodes,
        ) {
            let mut children = vec![];
            let node = nodes.get(id).unwrap();
            for &child in &node.children {
                build(&mut children, child, nodes, layout)
            }

            let view = &node.view.borrow();
            debug_nodes.push(DebugNode {
                id,
                name: short_name(view.type_name()),
                rect: layout.get(id).map(|c| c.rect).unwrap_or_default(),
                debug: format!("{view:#?}").split('\n').map(String::from).collect(),
                children,
                flex: view.flex(),
                interest: view.interests(),
            });
        }

        let mut children = vec![];
        let node = nodes.get(root).unwrap();

        for &node in &node.children {
            build(&mut children, node, nodes, layout);
        }

        let view = &node.view.borrow();
        Self {
            id: root,
            name: short_name(view.type_name()),
            rect: layout.get(root).map(|c| c.rect).unwrap_or_default(),
            debug: format!("{view:#?}").split('\n').map(String::from).collect(),
            children,
            flex: view.flex(),
            interest: view.interests(),
        }
    }
}

fn render_compact_tree(node: &DebugNode) -> String {
    use std::fmt::Write as _;
    fn print(children: &[DebugNode], prefix: &str, out: &mut impl std::fmt::Write) {
        for (i, node) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let connector = if is_last { "└─ " } else { "├─ " };

            _ = writeln!(
                out,
                "{prefix}{connector}{name}({id:?})",
                name = node.name,
                id = node.id.data(),
            );

            let prefix = if is_last {
                format!("{prefix}   ")
            } else {
                format!("{prefix}│  ")
            };

            print(&node.children, &prefix, out)
        }
    }

    let mut out = String::new();
    let _ = writeln!(out, "{name}({id:?})", name = node.name, id = node.id.data());
    print(&node.children, "", &mut out);
    out
}

fn render_pretty_tree(node: &DebugNode) -> String {
    enum DebugLabel {
        Header,
        Separator,
        Split {
            min: CompactString,
            max: CompactString,
        },
        Label {
            align: Align,
            text: CompactString,
        },
    }

    impl DebugLabel {
        fn new(s: impl compact_str::ToCompactString, align: Align) -> Self {
            Self::Label {
                align,
                text: s.to_compact_string(),
            }
        }

        fn split(&self) -> impl Iterator<Item = Self> + '_ {
            let (mut a, mut b, mut c, mut d) = (None, None, None, None);
            match self {
                Self::Header => d = Some(std::iter::once(Self::Header)),
                Self::Separator => a = Some(std::iter::once(Self::Separator)),
                Self::Split { min, max } => {
                    b = Some(std::iter::once(Self::Split {
                        min: min.clone(),
                        max: max.clone(),
                    }));
                }
                Self::Label { align, text } => {
                    c = Some(text.split('\n').map(|s| Self::new(s, *align)))
                }
            };
            std::iter::from_fn(move || match self {
                Self::Separator => a.as_mut()?.next(),
                Self::Header => d.as_mut()?.next(),
                Self::Split { .. } => b.as_mut()?.next(),
                Self::Label { .. } => c.as_mut()?.next(),
            })
        }

        fn len(&self) -> usize {
            match self {
                Self::Separator | Self::Header => 0,
                Self::Split { min, max } => min.width() + max.width() + 1,
                Self::Label { text, .. } => text.width(),
            }
        }
    }

    struct Node {
        center: usize,
        width: usize,
        height: usize,
        total_width: usize,
        total_height: usize,
        labels: Vec<DebugLabel>,
        children: Vec<Self>,
    }

    impl Node {
        fn new(node: &DebugNode, spacing: usize) -> Self {
            let mut labels = vec![
                DebugLabel::Split {
                    min: format!("{:?}", node.id.data()).into(),
                    max: node.name.clone().into(),
                },
                DebugLabel::Header,
                DebugLabel::Split {
                    min: format!("x: {:?}", node.rect.min.x).into(),
                    max: format!("w: {:?}", node.rect.width()).into(),
                },
                DebugLabel::Split {
                    min: format!("y: {:?}", node.rect.min.y).into(),
                    max: format!("h: {:?}", node.rect.height()).into(),
                },
                DebugLabel::Separator,
            ];

            if !node.interest.is_none() {
                for label in format!("{:?}", node.interest).split(" | ") {
                    labels.push(DebugLabel::Label {
                        align: Align::Center,
                        text: label.into(),
                    });
                }
                labels.push(DebugLabel::Separator);
            }

            if node.flex.has_flex() {
                let flex = match node.flex {
                    Flex::Tight(..) => "Tight",
                    Flex::Loose(..) => "Loose",
                };

                labels.push(DebugLabel::Split {
                    min: "Flex:".into(),
                    max: format!("{:.2?}", node.flex.factor()).into(),
                });

                labels.push(DebugLabel::Split {
                    min: "Fit:".into(),
                    max: flex.into(),
                });
                labels.push(DebugLabel::Separator);
            }

            for debug in &node.debug {
                labels.push(DebugLabel::new(debug, Align::Min));
            }

            if node.children.len() > 1 {
                labels.push(DebugLabel::Separator);
                labels.push(DebugLabel::Split {
                    min: "Children".into(),
                    max: format!("{}", node.children.len()).into(),
                });
            }

            let labels = labels.iter().flat_map(|s| s.split()).collect::<Vec<_>>();

            let node_width = labels.iter().map(|c| c.len()).max().unwrap() + 4;
            let node_height = labels.len() + 2;

            let children = node
                .children
                .iter()
                .map(|node| Self::new(node, spacing))
                .collect::<Vec<_>>();
            let children_width = Self::children_width(&children, spacing);

            let total_width = std::cmp::max(node_width, children_width);
            let mut total_height = node_height;
            if !node.children.is_empty() {
                let children_height = children.iter().map(|c| c.total_height).max().unwrap_or(0);
                total_height = if node.children.len() == 1 {
                    node_height + children_height
                } else {
                    node_height + children_height + 1
                };
            };

            let cx = (node_width - 1) / 2;
            let center = match (children.first(), children.last()) {
                (Some(first), Some(last)) => {
                    cx.max(first.center + children_width - last.total_width + last.center) / 2
                }
                _ => cx,
            };

            Self {
                center,
                width: node_width,
                height: node_height,
                total_width,
                total_height,
                labels,
                children,
            }
        }

        fn children_width(children: &[Node], spacing: usize) -> usize {
            if children.is_empty() {
                return 0;
            }
            children.iter().map(|c| c.total_width).sum::<usize>() + (children.len() - 1) * spacing
        }

        fn print(&self, grid: &mut Vec<Vec<char>>, (x0, y0): (usize, usize), spacing: usize) {
            let left = x0 + self.center.saturating_sub((self.width - 1) / 2);
            let right = left + self.width;

            for x in left + 1..right - 1 {
                grid[y0][x] = '─';
                grid[y0 + self.height - 1][x] = '─';
            }

            // for clippy
            for grid in grid.iter_mut().take(y0 + self.height - 1).skip(y0 + 1) {
                grid[left] = '│';
                grid[right - 1] = '│';
            }

            grid[y0][left] = '┌';
            grid[y0][right - 1] = '┐';
            grid[y0 + self.height - 1][left] = '└';
            grid[y0 + self.height - 1][right - 1] = '┘';

            for (row, label) in self.labels.iter().enumerate() {
                match label {
                    DebugLabel::Header => {
                        grid[y0 + row + 1][left] = '╞';
                        grid[y0 + row + 1][left + self.width - 1] = '╡';
                        for i in 1..self.width - 1 {
                            grid[y0 + row + 1][left + i] = '═';
                        }
                    }
                    DebugLabel::Separator => {
                        grid[y0 + row + 1][left] = '┝';
                        grid[y0 + row + 1][left + self.width - 1] = '┥';
                        for i in 1..self.width - 1 {
                            grid[y0 + row + 1][left + i] = '╌';
                        }
                    }
                    DebugLabel::Split { min, max } => {
                        let start = left + 2;
                        for (i, ch) in min.char_indices() {
                            grid[y0 + row + 1][start + i] = ch
                        }
                        let start = start + (self.width - max.width()) - 4;
                        for (i, ch) in max.char_indices() {
                            grid[y0 + row + 1][start + i] = ch
                        }
                    }
                    DebugLabel::Label { align, text } => {
                        let offset = match align {
                            Align::Min => 2,
                            Align::Center => (self.width - text.width()) / 2,
                            Align::Max => (self.width - text.width()) - 2,
                        };

                        let start = left + offset;
                        for (i, ch) in text.char_indices() {
                            grid[y0 + row + 1][start + i] = ch
                        }
                    }
                }
            }

            if (x0, y0) != (0, 0) {
                grid[y0][x0 + self.center] = '┷';
            }

            self.visit_children(grid, (x0, y0), spacing)
        }

        fn visit_children(
            &self,
            grid: &mut Vec<Vec<char>>,
            (x0, y0): (usize, usize),
            spacing: usize,
        ) {
            if self.children.is_empty() {
                return;
            }

            grid[y0 + self.height - 1][x0 + self.center] = '┯';

            let cy0 = if self.children.len() > 1 {
                y0 + self.height + 1
            } else {
                y0 + self.height
            };

            let children = Self::children_width(&self.children, spacing);
            let mut cx0 = if self.children.is_empty() || children > self.width {
                x0
            } else {
                x0 + (self.width / children) / 2
            };

            for id in 0..self.children.len() {
                let child = &self.children[id];
                child.print(grid, (cx0, cy0), spacing);

                if id == self.children.len() - 1 {
                    continue;
                }

                let start = cx0 + child.center + 1;
                let end = cx0 + child.total_width + spacing + self.children[id + 1].center;

                for x in start..end {
                    grid[y0 + self.height][x] = if x != x0 + self.center { '╌' } else { '┴' };
                }

                if id == 0 {
                    grid[y0 + self.height][start - 1] = '╭';
                }

                let merge = if id == self.children.len() - 2 {
                    '╮'
                } else if end == x0 + self.center {
                    '┼'
                } else {
                    '┬'
                };

                grid[y0 + self.height][end] = merge;

                cx0 += child.total_width + spacing
            }
        }

        fn render(&self, spacing: usize) -> String {
            let mut grid = vec![vec![' '; self.total_width]; self.total_height];
            self.print(&mut grid, (0, 0), spacing);
            grid.into_iter().fold(String::new(), |mut a, mut c| {
                if !a.is_empty() {
                    a.push('\n');
                }
                let pos = c.iter().rposition(|&c| c != ' ').unwrap_or(0);
                c.truncate(pos + 1);
                a.extend(c);
                a
            })
        }
    }

    Node::new(node, 1).render(1)
}

// pub fn render_tree<R: 'static>(surface: &mut Surface, mut app: impl FnMut(&Ui) -> R) -> String {
//     let mut state = State::new();
//     state.build(rect(surface.rect().size()), &mut app);
//     state.render(&mut (), surface, &mut AnimationManager::new());

//     let mut debug = DebugRenderer::new();
//     surface.render(&mut debug).unwrap();
//     debug.to_string()
// }

pub struct TreeOutput {
    tree: String,
    debug: Vec<String>,
}

impl TreeOutput {
    pub fn tree(&self) -> &str {
        &self.tree
    }

    pub fn debug(&self) -> &[String] {
        &self.debug
    }
}

impl std::fmt::Display for TreeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tree)
    }
}

fn evaluate<R: 'static>(mut app: impl FnMut(&Ui) -> R) -> (DebugNode, Vec<String>) {
    let mut state = State::default();

    let size = vec2(80, 25);
    state.event(&Event::Resize(size));

    let mut surface = Surface::new(size);
    // for i in 0..1 {
    state.build(rect(size), &mut app);
    state.render(&mut surface);
    // }

    let node = DebugNode::from_state(&state);

    let mut debug = vec![];
    Debug::for_each(|msg| debug.push(msg.to_owned()));
    (node, debug)
}

pub fn pretty_tree<R: 'static>(app: impl FnMut(&Ui) -> R) -> TreeOutput {
    let (node, debug) = evaluate(app);
    let tree = node.pretty_tree();
    TreeOutput { tree, debug }
}

pub fn compact_tree<R: 'static>(app: impl FnMut(&Ui) -> R) -> TreeOutput {
    let (node, debug) = evaluate(app);
    let tree = node.compact_tree();
    TreeOutput { tree, debug }
}
