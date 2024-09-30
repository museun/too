//! Math types and  helpers used by this crate

mod rect;
pub use rect::{rect, Rect};

mod vec2;
pub use vec2::{vec2, Vec2};

mod rot2;
pub use rot2::{rot2, Rot2};

mod vec3;
pub use vec3::{vec3, Vec3};

mod pos2;
pub use pos2::{pos2, Pos2};

mod num;
pub use num::Num;
pub use num::{almost_eq, inverse_lerp, lerp, midpoint};
