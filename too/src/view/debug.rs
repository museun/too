use super::{geom::Rectf, views::Flex, App, Node, Properties, Ui, ViewId};
use crate::layout::Align;
use std::marker::PhantomData;

struct DebugNode<T: 'static> {
    id: ViewId,
    name: String,
    rect: Rectf,
    properties: Vec<String>,
    children: Vec<Self>,
    _marker: PhantomData<T>,
}

impl<T> std::fmt::Debug for DebugNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("rect", &self.rect)
            .field("properties", &self.properties)
            .field("children", &self.children)
            .finish()
    }
}

impl<T: 'static> DebugNode<T> {
    fn new(root: ViewId, nodes: &thunderdome::Arena<Node<T>>, properties: &Properties) -> Self {
        fn build<T: 'static>(
            nodes: &mut Vec<DebugNode<T>>,
            id: ViewId,
            view_nodes: &thunderdome::Arena<Node<T>>,
            properties: &Properties,
        ) {
            let mut children = vec![];
            let node = &*view_nodes[id.0];
            for &child in &node.children {
                build(&mut children, child, view_nodes, properties)
            }

            let mut props = vec![];
            props.extend(properties.get_for::<Flex>(id).map(|c| format!("{c:#?}")));

            nodes.push(DebugNode {
                id,
                name: node.view.type_name().to_string(),
                rect: node.rect,
                properties: props,
                children,
                _marker: PhantomData,
            });
        }

        let mut children = vec![];
        let node = &*nodes[root.0];

        for &node in &node.children {
            build(&mut children, node, nodes, properties);
        }

        let mut props = vec![];
        props.extend(properties.get_for::<Flex>(root).map(|c| format!("{c:#?}")));

        Self {
            id: root,
            name: node.view.type_name().to_string(),
            rect: node.rect,
            properties: props,
            children,
            _marker: PhantomData,
        }
    }
}

fn render_flat_tree<T: 'static>(node: &DebugNode<T>) -> String {
    use std::fmt::Write as _;
    impl<T: 'static> std::fmt::Display for DebugNode<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}({:?}): {}, {}, {}, {}",
                self.name,
                self.id,
                self.rect.min.x,
                self.rect.min.y,
                self.rect.max.x,
                self.rect.max.y
            )
        }
    }

    fn print<T: 'static>(children: &[DebugNode<T>], prefix: &str, out: &mut impl std::fmt::Write) {
        for (i, node) in children.iter().enumerate() {
            if i < children.len() - 1 {
                _ = writeln!(out, "{prefix}- {node}",);
                print(&node.children, &format!("{prefix}|"), out);
            } else if i < children.len()
                && children.last().filter(|c| c.children.is_empty()).is_some()
            {
                _ = writeln!(out, "{prefix}-\\ {node}",);
                print(&node.children, &format!("{prefix} "), out);
            } else {
                _ = writeln!(out, "{prefix}\\ {node}",);
                print(&node.children, &format!("{prefix} "), out);
            }
        }
    }

    let mut out = String::new();
    let _ = writeln!(out, "{node}",);
    print(&node.children, "", &mut out);
    out
}

fn render_flow_tree<T: 'static>(node: &DebugNode<T>) -> String {
    // ─ │ ┈ ┌ ┐ ┝ ┥ ┬ ┯ ┴ ┷ ┼ ╌ ╭ ╮ ╯ ╰

    enum Label {
        Separator,
        Label { align: Align, text: String },
    }

    impl Label {
        fn new(s: impl ToString, align: Align) -> Self {
            Self::Label {
                align,
                text: s.to_string(),
            }
        }

        fn split(&self) -> impl Iterator<Item = Self> + '_ {
            let (mut a, mut b) = (None, None);
            match self {
                Self::Separator => a = Some(std::iter::once(Self::Separator)),
                Self::Label { align, text } => {
                    b = Some(text.split('\n').map(|s| Self::new(s, *align)))
                }
            };
            std::iter::from_fn(move || match self {
                Self::Separator => a.as_mut()?.next(),
                Self::Label { align, text } => b.as_mut()?.next(),
            })
        }

        fn len(&self) -> usize {
            match self {
                Self::Separator => 0,
                Self::Label { text, .. } => text.len(),
            }
        }
    }

    struct Node {
        center: usize,
        width: usize,
        height: usize,
        total_width: usize,
        total_height: usize,
        labels: Vec<Label>,
        children: Vec<Self>,
    }

    impl Node {
        fn new<T: 'static>(node: &DebugNode<T>, spacing: usize) -> Self {
            let mut labels = vec![
                Label::new(format!("{:?}", node.id), Align::Center),
                Label::Separator,
                Label::new(&node.name, Align::Center),
                Label::Separator,
                Label::new(format!("x: {:?}", node.rect.min.x), Align::Min),
                Label::new(format!("y: {:?}", node.rect.min.y), Align::Min),
                Label::Separator,
                Label::new(format!("w: {:?}", node.rect.width()), Align::Max),
                Label::new(format!("h: {:?}", node.rect.height()), Align::Max),
            ];

            let len = labels.len();

            if !node.properties.is_empty() {
                labels.push(Label::Separator);
                labels.push(Label::new("Properties", Align::Center));
            }

            for props in &node.properties {
                labels.push(Label::Separator);
                labels.push(Label::new(props, Align::Min));
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

            for y in y0 + 1..y0 + self.height - 1 {
                grid[y][left] = '│';
                grid[y][right - 1] = '│';
            }

            grid[y0][left] = '╭';
            grid[y0][right - 1] = '╮';
            grid[y0 + self.height - 1][left] = '╰';
            grid[y0 + self.height - 1][right - 1] = '╯';

            for (row, label) in self.labels.iter().enumerate() {
                match label {
                    Label::Separator => {
                        grid[y0 + row + 1][left] = '┝';
                        grid[y0 + row + 1][left + self.width - 1] = '┥';
                        for i in 1..self.width - 1 {
                            grid[y0 + row + 1][left + i] = '┈';
                        }
                    }
                    Label::Label { align, text } => {
                        let offset = match align {
                            Align::Min => 2,
                            Align::Center => (self.width - text.len()) / 2,
                            Align::Max => (self.width - text.len()) - 2,
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
                    grid[y0 + self.height][start - 1] = '┌';
                }

                let merge = if id == self.children.len() - 2 {
                    '┐'
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

fn run<T: App>(mut app: T) -> DebugNode<T> {
    let mut ui = <Ui<T>>::new();
    ui.rect = (80.0, 25.0).into();
    ui.scope(&mut app, []);
    DebugNode::new(ui.root, &ui.nodes, &ui.properties)
}

pub fn debug_flow_tree<T: App>(app: T) -> String {
    render_flow_tree(&run(app))
}

pub fn debug_flat_tree<T: App>(app: T) -> String {
    render_flat_tree(&run(app))
}
