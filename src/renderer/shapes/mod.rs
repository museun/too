//! Convenient 'shapes' for drawing to a [`Surface`](crate::Surface)

mod fill;
pub use fill::Fill;

mod border;
pub use border::Border;

mod line;
pub use line::Line;

mod label;
pub use label::Label;

mod text;
pub use text::Text;

mod anonymous;
pub use anonymous::{anonymous, anonymous_ctx};
