//! Debug helpers for a view
//!
//! This can print trees in various forms
use compact_str::CompactString;
use slotmap::Key;
use unicode_width::UnicodeWidthStr;

use crate::{
    backend::Event,
    helpers::short_name,
    layout::{Align, Flex},
    math::{rect, vec2, Rect},
    renderer::Shape,
    view::layout::Layer,
    Str,
};

use super::{
    state::Debug, test::DebugRasterizer, Interest, LayoutNodes, State, Ui, ViewId, ViewNodes,
};

#[derive(Debug)]
pub struct DebugNode {
    id: ViewId,
    name: String,
    debug: Vec<String>,
    children: Vec<Self>,
    inner: InnerNode,
}

#[derive(Debug, Default)]
enum InnerNode {
    FoundNode {
        rect: Rect,
        flex: Flex,
        layer: Layer,
        interactive: bool,
        interest: Interest,
    },
    #[default]
    MissingLayout,
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

    fn new(root: ViewId, nodes: &ViewNodes, layout: &LayoutNodes) -> Self {
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

            let mut debug_node = DebugNode {
                id,
                name: short_name(view.type_name()),
                debug: format!("{view:#?}").split('\n').map(String::from).collect(),
                children,
                inner: InnerNode::MissingLayout,
            };

            if let Some(layout_node) = layout.get(id) {
                debug_node.inner = InnerNode::FoundNode {
                    rect: layout_node.rect,
                    flex: view.flex(),
                    interactive: layout_node.interactive,
                    layer: layout_node.layer,
                    interest: view.interests(),
                };
            };

            debug_nodes.push(debug_node);
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

            debug: format!("{view:#?}").split('\n').map(String::from).collect(),
            children,
            inner: InnerNode::FoundNode {
                rect: layout.nodes[root].rect,
                flex: view.flex(),
                interactive: false,
                layer: layout.nodes[root].layer,
                interest: view.interests(),
            },
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
        fn new(s: impl Into<Str>, align: Align) -> Self {
            Self::Label {
                align,
                text: s.into().into_inner(),
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
        fn build_labels(node: &DebugNode) -> Vec<DebugLabel> {
            let mut labels = vec![
                DebugLabel::Split {
                    min: format!("{:?}", node.id.data()).into(),
                    max: node.name.clone().into(),
                },
                DebugLabel::Header,
            ];

            match node.inner {
                InnerNode::FoundNode {
                    rect,
                    flex,
                    layer,
                    interactive,
                    interest,
                } => {
                    labels.extend([
                        DebugLabel::Split {
                            min: compact_str::format_compact!("x: {:?}", rect.min.x),
                            max: compact_str::format_compact!("w: {:?}", rect.width()),
                        },
                        DebugLabel::Split {
                            min: compact_str::format_compact!("y: {:?}", rect.min.y),
                            max: compact_str::format_compact!("h: {:?}", rect.height()),
                        },
                        DebugLabel::Separator,
                        DebugLabel::Split {
                            min: CompactString::const_new("Layer"),
                            max: CompactString::const_new(match layer {
                                Layer::Bottom => "Bottom",
                                Layer::Middle => "Middle",
                                Layer::Top => "Top",
                                Layer::Debug => "Debug",
                            }),
                        },
                        DebugLabel::Separator,
                        DebugLabel::Split {
                            min: CompactString::const_new("Interactive"),
                            max: CompactString::const_new(match interactive {
                                true => "true",
                                false => "false",
                            }),
                        },
                        DebugLabel::Separator,
                    ]);

                    if !interest.is_none() {
                        for label in format!("{:?}", interest).split(" | ") {
                            labels.push(DebugLabel::Label {
                                align: Align::Center,
                                text: label.into(),
                            });
                        }
                        labels.push(DebugLabel::Separator);
                    }

                    if flex.has_flex() {
                        let flex_fit = match flex {
                            Flex::Tight(..) => "Tight",
                            Flex::Loose(..) => "Loose",
                        };

                        labels.push(DebugLabel::Split {
                            min: CompactString::const_new("Flex:"),
                            max: compact_str::format_compact!("{:.2?}", flex.factor()),
                        });

                        labels.push(DebugLabel::Split {
                            min: CompactString::const_new("Fit:"),
                            max: CompactString::const_new(flex_fit),
                        });
                        labels.push(DebugLabel::Separator);
                    }
                }
                InnerNode::MissingLayout => {
                    labels.push(DebugLabel::Label {
                        align: Align::Center,
                        text: CompactString::const_new("!! Node not used in layout !!"),
                    });
                    labels.push(DebugLabel::Header);
                }
            }

            for debug in &node.debug {
                labels.push(DebugLabel::new(debug, Align::Min));
            }

            if node.children.len() > 1 {
                labels.push(DebugLabel::Separator);
                labels.push(DebugLabel::Split {
                    min: CompactString::const_new("Children"),
                    max: format!("{}", node.children.len()).into(),
                });
            }

            labels
        }

        fn new(node: &DebugNode, spacing: usize) -> Self {
            let labels = Self::build_labels(node);

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

/// Output from a tree generation
///
/// This types implements [`std::fmt::Display`] so you can print the tree directly
pub struct TreeOutput {
    tree: String,
    debug: Vec<String>,
    shapes: Vec<(ViewId, Shape)>,
}

impl TreeOutput {
    /// Shapes generated by the application
    pub fn shapes(&self) -> &[(ViewId, Shape)] {
        &self.shapes
    }

    /// The tree representation of the application
    pub fn tree(&self) -> &str {
        &self.tree
    }

    /// Any debug messages produced by the application
    pub fn debug(&self) -> &[String] {
        &self.debug
    }
}

impl std::fmt::Display for TreeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tree)
    }
}

fn evaluate<R: 'static>(
    mut app: impl FnMut(&Ui) -> R,
) -> (DebugNode, Vec<String>, Vec<(ViewId, Shape)>) {
    let mut state = State::default();

    let size = vec2(80, 25);
    state.event(&Event::Resize(size));
    for _ in 0..2 {
        state.build(rect(size), &mut app);
    }

    let mut raster = DebugRasterizer::default();
    state.render(&mut raster);

    let node = DebugNode::from_state(&state);

    let mut debug = vec![];
    Debug::for_each(|msg| debug.push(msg.to_owned()));
    (node, debug, raster.into_paint_list())
}

/// Generate a pretty tree of the views (nodes) for this application
///
/// Example:
/// ```rust,no_run
/// too::view::pretty_tree(|ui| {
///     ui.center(|ui| ui.label("hello world"));
///
///     ui.aligned(Align2::RIGHT_TOP, |ui| {
///         ui.button("click me");
///     });
///
///     ui.show(fill("#F0F", [10.0, 10.0]));
/// })
/// ```
/// produces:
/// ```text
///                                         ┌──────────────┐
///                                         │ 1v1     Root │
///                                         ╞══════════════╡
///                                         │ x: 0   w: 80 │
///                                         │ y: 0   h: 25 │
///                                         ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
///                                         │ Layer Bottom │
///                                         ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
///                                         │ Root         │
///                                         ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
///                                         │ Children   3 │
///                                         └──────┯───────┘
///                ╭╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╮
///     ┌──────────┷──────────┐         ┌──────────┷──────────┐     ┌───────────────┷────────────────┐
///     │ 2v1         Aligned │         │ 4v1         Aligned │     │ 6v1                       Fill │
///     ╞═════════════════════╡         ╞═════════════════════╡     ╞════════════════════════════════╡
///     │ x: 0          w: 80 │         │ x: 0          w: 80 │     │ x: 0                     w: 10 │
///     │ y: 0          h: 25 │         │ y: 0          h: 25 │     │ y: 0                     h: 10 │
///     ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥         ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥     ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
///     │ Layer        Middle │         │ Layer        Middle │     │ Layer                   Middle │
///     ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥         ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥     ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
///     │ Aligned {           │         │ Aligned {           │     │ Fill {                         │
///     │     align: Align2 { │         │     align: Align2 { │     │     bg: Some(                  │
///     │         x: Center,  │         │         x: Max,     │     │         rgb(255, 0, 255, 255), │
///     │         y: Center,  │         │         y: Min,     │     │     ),                         │
///     │     },              │         │     },              │     │     size: Size(10, 10),        │
///     │ }                   │         │ }                   │     │ }                              │
///     └──────────┯──────────┘         └──────────┯──────────┘     └────────────────────────────────┘
/// ┌──────────────┷──────────────┐ ┌──────────────┷──────────────┐
/// │ 3v1                   Label │ │ 5v1                  Button │
/// ╞═════════════════════════════╡ ╞═════════════════════════════╡
/// │ x: 35                 w: 11 │ │ x: 70                 w: 10 │
/// │ y: 12                  h: 1 │ │ y: 0                   h: 1 │
/// ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥ ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
/// │ Layer                Middle │ │ Layer                Middle │
/// ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥ ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
/// │ Label {                     │ │        MOUSE_INSIDE         │
/// │     label: "hello world",   │ ┝╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┥
/// │     class: Deferred(        │ │ Button {                    │
/// │         0x00007ff6fa0cc390, │ │     label: "click me",      │
/// │     ),                      │ │     margin: Margin {        │
/// │     main: Min,              │ │         left: 1,            │
/// │     attribute: None,        │ │         top: 0,             │
/// │ }                           │ │         right: 1,           │
/// └─────────────────────────────┘ │         bottom: 0,          │
///                                 │     },                      │
///                                 │     state: None,            │
///                                 │     disabled: false,        │
///                                 │     main: Min,              │
///                                 │     cross: Min,             │
///                                 │     class: Deferred(        │
///                                 │         0x00007ff6fa086d50, │
///                                 │     ),                      │
///                                 │ }                           │
///                                 └─────────────────────────────┘
/// ```
pub fn pretty_tree<R: 'static>(app: impl FnMut(&Ui) -> R) -> TreeOutput {
    let (node, debug, shapes) = evaluate(app);
    let tree = node.pretty_tree();
    TreeOutput {
        tree,
        debug,
        shapes,
    }
}

/// Generate a compact tree of the views (nodes) for this application
///
/// Example:
/// ```rust,no_run
/// too::view::compact_tree(|ui| {
///     ui.center(|ui| ui.label("hello world"));
///
///     ui.aligned(Align2::RIGHT_TOP, |ui| {
///         ui.button("click me");
///     });
///
///     ui.show(fill("#F0F", [10.0, 10.0]));
/// })
/// ```
/// produces:
/// ```text
/// Root(1v1)
/// ├─ Aligned(2v1)
/// │  └─ Label(3v1)
/// ├─ Aligned(4v1)
/// │  └─ Button(5v1)
/// └─ Fill(6v1)
/// ```
pub fn compact_tree<R: 'static>(app: impl FnMut(&Ui) -> R) -> TreeOutput {
    let (node, debug, shapes) = evaluate(app);
    let tree = node.compact_tree();
    TreeOutput {
        tree,
        debug,
        shapes,
    }
}

/// Generate the paint list this application would do
///
/// Example:
/// ```rust,no_run
/// too::view::render_tree(|ui| {
///     ui.center(|ui| ui.label("hello world"));
///
///     ui.aligned(Align2::RIGHT_TOP, |ui| {
///         ui.button("click me");
///     });
///
///     ui.show(fill("#F0F", [10.0, 10.0]));
/// })
/// ```
/// produces:
/// ```rust,no_run
/// ViewId(3v1): Text { rect: { x: 35, y: 12, w: 11, h: 1 }, shape: TextShape { label: "hello world", fg: Set(rgb(255, 255, 255, 255)), bg: Reuse, attribute: None } }
/// ViewId(5v1): FillBg { rect: { x: 70, y: 0, w: 10, h: 1 }, color: rgb(77, 77, 77, 255) }
/// ViewId(5v1): Text { rect: { x: 71, y: 0, w: 8, h: 1 }, shape: TextShape { label: "click me", fg: Set(rgb(255, 255, 255, 255)), bg: Reuse, attribute: None } }
/// ViewId(6v1): FillBg { rect: { x: 0, y: 0, w: 10, h: 10 }, color: rgb(255, 0, 255, 255) }
/// ```
pub fn render_tree<R: 'static>(app: impl FnMut(&Ui) -> R) -> Vec<(ViewId, Shape)> {
    let (.., shapes) = evaluate(app);
    shapes
}
