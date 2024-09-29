use crate::{vec3, Rgba, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gradient {
    pub offset: Vec3,
    pub amp: Vec3,
    pub freq: Vec3,
    pub phase: Vec3,
}

impl Gradient {
    pub const fn new(offset: Vec3, amp: Vec3, freq: Vec3, phase: Vec3) -> Self {
        Self {
            offset,
            amp,
            freq,
            phase,
        }
    }

    pub fn as_rgba(&self, time: f32) -> Rgba {
        Rgba::gradient(time, self.offset, self.amp, self.freq, self.phase)
    }
}

impl Gradient {
    pub const RAINBOW1: Self = Self {
        offset: vec3(0.5, 0.5, 0.5),
        amp: vec3(0.5, 0.5, 0.5),
        freq: vec3(1.0, 1.0, 1.0),
        phase: vec3(0.0, 0.3, 0.6),
    };

    pub const RAINBOW2: Self = Self {
        offset: vec3(0.5, 0.5, 0.5),
        amp: vec3(0.6, 0.6, 0.6),
        freq: vec3(1.0, 1.0, 1.0),
        phase: vec3(0.0, 0.3, 0.6),
    };

    pub const RAINBOW3: Self = Self {
        offset: vec3(0.5, 0.5, 0.5),
        amp: vec3(0.75, 0.75, 0.75),
        freq: vec3(1.0, 1.0, 1.0),
        phase: vec3(0.0, 0.3, 0.6),
    };

    pub const RAINBOW4: Self = Self {
        offset: vec3(0.5, 0.5, 0.5),
        amp: vec3(1.0, 1.0, 1.0),
        freq: vec3(1.0, 1.0, 1.0),
        phase: vec3(0.0, 0.3, 0.6),
    };

    pub const YELLOW_MAGENTA_CYAN: Self = Self {
        offset: vec3(1.0, 0.5, 0.5),
        amp: vec3(0.5, 0.5, 0.5),
        freq: vec3(0.75, 1.0, 0.6),
        phase: vec3(0.8, 1.0, 0.3),
    };

    pub const ORANGE_BLUE: Self = Self {
        offset: vec3(0.5, 0.5, 0.5),
        amp: vec3(0.5, 0.5, 0.5),
        freq: vec3(0.8, 0.8, 0.5),
        phase: vec3(0.0, 0.2, 0.5),
    };

    pub const GREEN_MAGENTA: Self = Self {
        offset: vec3(0.6, 0.5, 0.5),
        amp: vec3(0.5, 0.6, 0.5),
        freq: vec3(0.6, 0.6, 0.5),
        phase: vec3(0.2, 0.0, 0.5),
    };

    pub const GREEN_RED: Self = Self {
        offset: vec3(0.5, 0.5, 0.0),
        amp: vec3(0.5, 0.5, 0.0),
        freq: vec3(0.5, 0.5, 0.0),
        phase: vec3(0.5, 0.0, 0.0),
    };

    pub const GREEN_CYAN: Self = Self {
        offset: vec3(0.0, 0.5, 0.5),
        amp: vec3(0.0, 0.5, 0.5),
        freq: vec3(0.0, 0.3, 0.5),
        phase: vec3(0.0, 0.6, 0.5),
    };

    pub const YELLOW_RED: Self = Self {
        offset: vec3(0.5, 0.5, 0.0),
        amp: vec3(0.5, 0.5, 0.0),
        freq: vec3(0.1, 0.5, 0.0),
        phase: vec3(0.0, 0.0, 0.0),
    };

    pub const BLUE_CYAN: Self = Self {
        offset: vec3(0.0, 0.5, 0.5),
        amp: vec3(0.0, 0.5, 0.5),
        freq: vec3(0.0, 0.5, 0.3),
        phase: vec3(0.0, 0.5, 0.6),
    };

    pub const RED_BLUE: Self = Self {
        offset: vec3(0.5, 0.0, 0.5),
        amp: vec3(0.5, 0.0, 0.5),
        freq: vec3(0.5, 0.0, 0.5),
        phase: vec3(0.0, 0.0, 0.5),
    };

    pub const YELLOW_GREEN_BLUE: Self = Self {
        offset: vec3(0.650, 0.5, 0.310),
        amp: vec3(-0.650, 0.5, 0.6),
        freq: vec3(0.3, 0.278, 0.278),
        phase: vec3(0.660, 0.0, 0.667),
    };

    pub const BLUE_WHITE_RED: Self = Self {
        offset: vec3(0.660, 0.56, 0.680),
        amp: vec3(0.718, 0.438, 0.720),
        freq: vec3(0.520, 0.8, 0.520),
        phase: vec3(-0.430, -0.397, -0.083),
    };

    pub const CYAN_MAGENTA: Self = Self {
        offset: vec3(0.610, 0.498, 0.650),
        amp: vec3(0.388, 0.498, 0.350),
        freq: vec3(0.530, 0.498, 0.620),
        phase: vec3(3.438, 3.012, 4.025),
    };

    pub const YELLOW_PURPLE_MAGENTA: Self = Self {
        offset: vec3(0.731, 1.098, 0.192),
        amp: vec3(0.358, 1.090, 0.657),
        freq: vec3(1.077, 0.360, 0.328),
        phase: vec3(0.965, 2.265, 0.837),
    };

    pub const GREEN_BLUE_ORANGE: Self = Self {
        offset: vec3(0.892, 0.725, 0.000),
        amp: vec3(0.878, 0.278, 0.725),
        freq: vec3(0.332, 0.518, 0.545),
        phase: vec3(2.440, 5.043, 0.732),
    };

    pub const ORANGE_MAGENTA_BLUE: Self = Self {
        offset: vec3(0.821, 0.328, 0.242),
        amp: vec3(0.659, 0.481, 0.896),
        freq: vec3(0.612, 0.340, 0.296),
        phase: vec3(2.820, 3.026, -0.273),
    };

    pub const BLUE_MAGENTA_ORANGE: Self = Self {
        offset: vec3(0.938, 0.328, 0.718),
        amp: vec3(0.659, 0.438, 0.328),
        freq: vec3(0.388, 0.388, 0.296),
        phase: vec3(2.538, 2.478, 0.168),
    };

    pub const MAGENTA_GREEN: Self = Self {
        offset: vec3(0.590, 0.811, 0.120),
        amp: vec3(0.410, 0.392, 0.590),
        freq: vec3(0.940, 0.548, 0.278),
        phase: vec3(-4.242, -6.611, -4.045),
    };
}
