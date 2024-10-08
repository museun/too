use crate::overlay::DebugOverlay;

use super::{Properties, ViewId};

pub struct UpdateCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub properties: &'a mut Properties,
    pub(super) debug: &'a mut DebugOverlay,
}
