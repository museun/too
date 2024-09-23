mod buffer;
pub use buffer::Buffer;

mod renderer;
pub use renderer::{DebugRenderer, Renderer, TermRenderer};

mod surface;
pub use surface::{Surface, SurfaceMut};

mod shape;
pub use shape::{anonymous, Shape};

mod pixel;
pub use pixel::{Attribute, Color, Pixel};

mod rgba;
pub use rgba::Rgba;

mod gradient;
pub use gradient::Gradient;

mod backend;
pub use backend::{Backend, Command, CurrentScreen};
