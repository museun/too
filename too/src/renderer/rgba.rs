use std::f32::consts::{PI, TAU};

use crate::math::{inverse_lerp, Vec3};

// TODO a nice color! macro

// TODO should this use linear srgb?
// TODO should we store [f32;4] instead of [u8;4]?
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const TRANSPARENT: Self = Self(0, 0, 0, 0);
    pub const OPAQUE: Self = Self(255, 255, 255, 255);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    #[must_use]
    pub const fn red(&self) -> u8 {
        let Self(r, ..) = *self;
        r
    }

    #[must_use]
    pub const fn blue(&self) -> u8 {
        let Self(_, g, ..) = *self;
        g
    }

    #[must_use]
    pub const fn green(&self) -> u8 {
        let Self(.., b, _) = *self;
        b
    }

    #[must_use]
    pub const fn alpha(&self) -> u8 {
        let Self(.., a) = *self;
        a
    }

    #[must_use]
    pub const fn from_u16(color: u16) -> Self {
        let a = (color >> 12) & ((1 << 4) - 1);
        let is_16 = a == 0;
        let offset = if is_16 { 4 } else { 0 };

        let r = ((color >> (12 - offset)) & 0xF) as u8;
        let g = ((color >> (8 - offset)) & 0xF) as u8;
        let b = ((color >> (4 - offset)) & 0xF) as u8;
        let a = if is_16 { 0xF } else { (color & 0xF) as u8 };

        Self((r << 4) | r, (g << 4) | g, (b << 4) | b, (a << 4) | a)
    }

    #[must_use]
    pub const fn hex(color: &'static str) -> Self {
        #[track_caller]
        const fn to_digit(d: u8) -> u8 {
            assert!(d.is_ascii_hexdigit(), "invalid hex digit");
            match d.wrapping_sub(b'0') {
                d if d < 10 => d,
                _ => d.to_ascii_lowercase().saturating_sub(b'a') + 10,
            }
        }

        #[track_caller]
        const fn pack(high: u8, low: u8) -> u8 {
            to_digit(high) << 4 | to_digit(low)
        }

        let color = color.as_bytes();
        let len = color.len();
        let mut start = 0;
        while matches!(color[start], b' ' | b'\t' | b'\n') {
            start += 1;
        }

        let mut end = start;
        while end < color.len() && !matches!(color[end], b' ' | b'\t' | b'\n') {
            end += 1;
        }

        let (_, mut color) = color.split_at(start);
        if end - start < len {
            (color, _) = color.split_at(end - start);
        }

        // TODO support rgb(r,g,b) | rgba(r,g,b,a)
        let ((rh, gh, bh, ah), (rl, gl, bl, al)) = match color {
            &[b'#', rh, rl, gh, gl, bh, bl] => ((rh, gh, bh, b'F'), (rl, gl, bl, b'F')),
            &[b'#', rh, rl, gh, gl, bh, bl, ah, al] => ((rh, gh, bh, ah), (rl, gl, bl, al)),
            &[b'#', r, g, b] => ((r, g, b, b'F'), (r, g, b, b'F')),
            &[b'#', r, g, b, a] => ((r, g, b, a), (r, g, b, a)),
            [a, d @ ..] if !matches!(a, b'#') && matches!(d.len(), 7 | 5 | 3 | 2) => {
                panic!("missing '#' prefix")
            }
            &[] => panic!("empty string"),
            _ => panic!("invalid color. syntax: #RRGGBB | #RRGGBBAA | #RGB | #RGBA"),
        };

        Self(pack(rh, rl), pack(gh, gl), pack(bh, bl), pack(ah, al))
    }

    #[must_use]
    pub const fn to_opaque(mut self) -> Self {
        self.3 = 255;
        self
    }

    #[must_use]
    pub const fn with_alpha(mut self, alpha: u8) -> Self {
        self.3 = alpha;
        self
    }

    /// This is between 0% and 100%
    #[must_use]
    pub fn to_transparent(mut self, alpha: f32) -> Self {
        let t = inverse_lerp(0.0, 100.0, alpha).unwrap_or(1.0);
        self.3 = (t * 255.0) as u8;
        self
    }

    #[must_use]
    pub fn to_float(&self) -> [f32; 4] {
        let Self(r, g, b, a) = *self;
        let scale = |d| (d as f32 / 256.0);
        [scale(r), scale(g), scale(b), scale(a)]
    }

    #[must_use]
    pub fn from_float([r, g, b, a]: [f32; 4]) -> Self {
        let scale = |d: f32| (255.0_f32 * d).round() as u8;
        Self(scale(r), scale(g), scale(b), scale(a))
    }

    #[must_use]
    pub fn mix(self, left: f32, other: Self, right: f32) -> Self {
        let [r1, g1, b1, a1] = self.to_float();
        let [r2, g2, b2, a2] = other.to_float();
        let ratio = left + right;
        Self::from_float([
            left.mul_add(r1, right * r2) / ratio,
            left.mul_add(g1, right * g2) / ratio,
            left.mul_add(b1, right * b2) / ratio,
            a1.max(a2),
        ])
    }

    #[must_use]
    pub fn blend(&self, other: Self, mix: f32) -> Self {
        self.mix(mix, other, mix)
    }

    #[must_use]
    pub fn blend_alpha(&self, other: Self) -> Self {
        const fn blend(a: i32, l: u8, r: u8) -> u8 {
            ((a * l as i32 + (255 - a) * r as i32) / 255) as u8
        }

        let a = match self.alpha() as i32 {
            0 => return other,
            255 => return *self,
            a => a,
        };
        let r = blend(a, self.red(), other.red());
        let g = blend(a, self.blue(), other.blue());
        let b = blend(a, self.green(), other.green());
        Self(r, g, b, 255)
    }

    #[must_use]
    pub fn blend_linear(&self, other: Self, mix: f32) -> Self {
        let [r1, g1, b1, a1] = self.to_float();
        let [r2, g2, b2, a2] = other.to_float();
        Self::from_float([
            (r2 - r1).mul_add(mix, r1),
            (g2 - g1).mul_add(mix, g1),
            (b2 - b1).mul_add(mix, b1),
            a1.max(a2),
        ])
    }

    #[must_use]
    pub fn gradient(t: f32, offset: Vec3, amp: Vec3, freq: Vec3, phase: Vec3) -> Self {
        let v = offset + amp * ((freq * t + phase) * TAU).cos();
        Self::from_float([v.x, v.y, v.z, 1.0])
    }

    #[must_use]
    pub fn sine(t: f32) -> Self {
        let h = t * ((1.0 + 5.0_f32.sqrt()) / 2.0);
        let h = (h + 0.5) * -1.0;
        let r = (PI * h).sin();
        let g = (PI * (h + 0.3)).sin();
        let b = (PI * (h + 0.6)).sin();
        Self::from_float([r * r, g * g, b * b, 1.0])
    }
}

impl From<u16> for Rgba {
    fn from(rgb: u16) -> Self {
        Self::from_u16(rgb)
    }
}

impl From<&'static str> for Rgba {
    fn from(rgba: &'static str) -> Self {
        Self::hex(rgba)
    }
}

impl std::fmt::Display for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b, a) = self;
        if self.alpha() == 0 {
            write!(f, "rgb({r}, {g}, {b})")
        } else {
            write!(f, "rgb({r}, {g}, {b}, {a})")
        }
    }
}

impl std::fmt::LowerHex for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b, a) = self;
        write!(f, "0x{r:02x}{g:02x}{b:02x}")?;
        if self.alpha() != 0xFF {
            write!(f, "{a:02x}")?;
        }
        Ok(())
    }
}

impl std::fmt::UpperHex for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b, a) = self;
        write!(f, "0x{r:02X}{g:02X}{b:02X}")?;
        if self.alpha() != 0xFF {
            write!(f, "{a:02X}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Rgba {
    type Err = &'static str;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const ERR: &str = "color must be in the form of \
            rgb(r, g, b) \
            rgb(r, g, b, a) \
            or #RRGGBB \
            or #RRGGBBAA \
            or #RGB \
            or #RGBA";

        if let Some(input) = input.strip_prefix('#') {
            return match input.len() {
                3 | 4 => u16::from_str_radix(input, 16)
                    .map_err(|_| "invalid hex digits")
                    .map(Self::from_u16),
                6 | 8 => u32::from_str_radix(input, 16)
                    .map_err(|_| "invalid hex digits")
                    .map(|num| {
                        let [r, g, b, a] = num.to_be_bytes();
                        Self(r, g, b, a)
                    }),
                _ => Err(ERR),
            };
        }

        if input.starts_with("rgb(") && input.ends_with(')') {
            let input = &input[4..input.len() - 1];
            let mut iter = input.split_terminator(',').map(|s| s.trim().parse());
            let r = iter
                .next()
                .and_then(Result::ok)
                .ok_or("invalid red channel")?;
            let g = iter
                .next()
                .and_then(Result::ok)
                .ok_or("invalid green channel")?;
            let b = iter
                .next()
                .and_then(Result::ok)
                .ok_or("invalid blue channel")?;

            let a = match iter.next() {
                None => 0xFF,
                Some(Ok(a)) => a,
                Some(Err(..)) => return Err("invalid alpha channel"),
            };

            return Ok(Self(r, g, b, a));
        }

        Err(ERR)
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self::OPAQUE
    }
}
