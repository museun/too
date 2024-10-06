#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rot2 {
    pub s: f32,
    pub c: f32,
}

impl Default for Rot2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Rot2 {
    pub const ZERO: Self = rot2(0.0, 0.0);
    pub const IDENTITY: Self = rot2(0.0, 1.0);

    #[must_use]
    pub fn from_angle(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        rot2(s, c)
    }

    #[must_use]
    pub fn angle(self) -> f32 {
        self.s.atan2(self.c)
    }

    #[must_use]
    pub fn length(self) -> f32 {
        self.c.hypot(self.s)
    }

    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.c.powi(2) + self.s.powi(2)
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.c.is_finite() && self.s.is_finite()
    }

    #[must_use]
    pub fn is_infinite(self) -> bool {
        self.c.is_infinite() && self.s.is_infinite()
    }

    #[must_use]
    pub fn inverse(self) -> Self {
        rot2(-self.s, self.c) / self.length_squared()
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        let l = self.length();
        rot2(self.c / l, self.s / l)
    }
}

impl std::ops::Mul for Rot2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        rot2(
            self.c * rhs.c - self.s * rhs.s,
            self.s * rhs.c + self.c * rhs.s,
        )
    }
}

impl std::ops::Mul<(f32, f32)> for Rot2 {
    type Output = (f32, f32);
    fn mul(self, (x, y): (f32, f32)) -> Self::Output {
        (self.c * x - self.s * y, self.s * x + self.c * y)
    }
}

impl std::ops::Mul<Rot2> for f32 {
    type Output = Rot2;
    fn mul(self, rhs: Rot2) -> Self::Output {
        rot2(self * rhs.c, self * rhs.s)
    }
}

impl std::ops::Mul<f32> for Rot2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        rot2(self.c * rhs, self.s * rhs)
    }
}

impl std::ops::Div<f32> for Rot2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        rot2(self.c / rhs, self.s / rhs)
    }
}

pub const fn rot2(x: f32, y: f32) -> Rot2 {
    Rot2 { s: x, c: y }
}
