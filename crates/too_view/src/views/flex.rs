use crate::{view::Context, UpdateCtx, View, ViewExt};

#[derive(Copy, Clone, Debug, Default)]
pub struct Flex {
    pub amount: f32,
    pub tight: bool,
}

impl Flex {
    pub const fn amount(amount: f32) -> Self {
        Self {
            amount,
            tight: false,
        }
    }
    pub const fn tight(mut self, tight: bool) -> Self {
        self.tight = tight;
        self
    }

    pub fn show<T: 'static, R>(
        self,
        ctx: &mut Context<T>,
        show: impl FnOnce(&mut Context<T>) -> R,
    ) -> R {
        let (_, resp) = FlexView::show_children(self, ctx, show);
        resp
    }
}

impl From<f32> for Flex {
    fn from(value: f32) -> Self {
        Self {
            amount: value,
            tight: false,
        }
    }
}

struct FlexView {
    params: Flex,
}

impl<T: 'static> View<T> for FlexView {
    type Args<'a> = Flex;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        ctx.properties.insert_for(self.params, ctx.current_id);
        self.params = args;
        // for &child in ctx.children {
        //     ctx.properties.insert_for(self.params, child);
        // }
    }
}

pub fn flex<T: 'static, R>(ctx: &mut Context<T>, show: impl FnOnce(&mut Context<T>) -> R) -> R {
    let args = const {
        Flex {
            amount: 1.0,
            tight: false,
        }
    };
    args.show(ctx, show)
}

pub fn expand<T: 'static, R>(ctx: &mut Context<T>, show: impl FnOnce(&mut Context<T>) -> R) -> R {
    let args = const {
        Flex {
            amount: 1.0,
            tight: true,
        }
    };
    args.show(ctx, show)
}
