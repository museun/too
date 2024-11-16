use crate::{
    math::{Size, Space},
    view::{Builder, Layout, Render, Ui, View},
};

use super::ToggleResponse;

pub const fn toggle(state: bool) -> Toggle {
    Toggle { state }
}

#[derive(Debug)]
pub struct Toggle {
    state: bool,
}

impl<'v> Builder<'v> for Toggle {
    type View = Self;
}

impl View for Toggle {
    type Args<'v> = Self;
    type Response = ToggleResponse;

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        let prev = self.state;
        self.state ^= args.state;
        ToggleResponse {
            changed: self.state != prev,
        }
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        if !self.state {
            return Size::ZERO;
        }
        self.default_layout(layout, space)
    }

    fn draw(&mut self, render: Render) {
        if !self.state {
            return;
        }
        self.default_draw(render);
    }
}
