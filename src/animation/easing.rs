//! Easing functions
use std::f32::consts::{PI, TAU};

pub type Easing = fn(f32) -> f32;

pub fn linear(t: f32) -> f32 {
    t
}

pub fn reverse(t: f32) -> f32 {
    1.0 - t
}

pub fn round_trip(t: f32) -> f32 {
    let t = if t < 0.5 { t } else { reverse(t) };
    t * 2.0
}

pub fn sine_in(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}

pub fn sine_out(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

pub fn sine_in_out(t: f32) -> f32 {
    -((PI * t).cos() - 1.0) / 2.0
}

pub fn exponential_in(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }

    10.0f32.mul_add(t, -10.0).exp2()
}

pub fn exponential_out(t: f32) -> f32 {
    if 1.0 <= t {
        return 1.0;
    }
    1.0 - (-10.0 * t).exp2()
}

pub fn exponential_in_out(t: f32) -> f32 {
    match t {
        t if t <= 0.0 => 0.0,
        t if 1.0 <= t => 1.0,
        t if t < 0.5 => 20.0f32.mul_add(t, -10.0).exp2() / 2.0,
        _ => (2.0 - (-20.0f32).mul_add(t, 10.0).exp2()) / 2.0,
    }
}

pub fn circular_int(t: f32) -> f32 {
    1.0 - t.mul_add(-t, 1.0).sqrt()
}

pub fn circular_out(t: f32) -> f32 {
    (t - 1.0).mul_add(-(t - 1.0), 1.0).sqrt()
}

pub fn circular_in_out(t: f32) -> f32 {
    if t < 0.5 {
        return (1.0 - (2.0 * t).mul_add(-2.0 * t, 1.0).sqrt()) / 2.0;
    }

    let f = (-2.0f32).mul_add(t, 2.0);
    (f.mul_add(-f, 1.0).sqrt() + 1.0) / 2.0
}

pub fn elastic_in(t: f32) -> f32 {
    match t {
        t if t <= 0.0 => 0.0,
        t if 1.0 <= t => 1.0,
        _ => {
            let f = 10.0f32.mul_add(t, -10.0).exp();
            -f * (t.mul_add(10.0, -10.75) * TAU / 3.0).sin()
        }
    }
}

pub fn elastic_out(t: f32) -> f32 {
    match t {
        t if t <= 0.0 => 0.0,
        t if 1.0 <= t => 1.0,
        _ => {
            let f = t.mul_add(10.0, -0.75) * TAU / 3.0;
            (10.0 * t).exp2().mul_add(f.sin(), 1.0)
        }
    }
}

pub fn elastic_in_out(t: f32) -> f32 {
    match t {
        t if t <= 0.0 => 0.0,
        t if 1.0 <= t => 1.0,
        t if t < 0.5 => {
            let f = 20.0f32.mul_add(t, -11.125) * TAU / 4.5;
            let e = 20.0f32.mul_add(t, -10.0).exp2();
            -e * f.sin() / 2.0
        }
        _ => {
            let f = 20.0f32.mul_add(t, -11.125) * TAU / 4.5;
            let e = (-20.0f32).mul_add(t, 10.0).exp2();
            (e * f.sin()) / 2.0 + 1.0
        }
    }
}

pub fn bounce_in(t: f32) -> f32 {
    1.0 - bounce_out(1.0 - t)
}

pub fn bounce_out(t: f32) -> f32 {
    const T: f32 = 11.0 / 4.0;
    const S: f32 = T * T;
    match t {
        t if t < 1.0 / T => S * t.powi(2),
        t if t < 2.0 / T => S.mul_add((t - 1.5 / T).powi(2), const { 3.0 / 4.0 }),
        t if t < 2.5 / T => S.mul_add((t - 2.25 / T).powi(2), const { 15.0 / 16.0 }),
        _ => S.mul_add((t - 2.265 / T).powi(2), const { 63.0 / 64.0 }),
    }
}

pub fn bounce_in_out(t: f32) -> f32 {
    if t < 0.5 {
        return (1.0 - bounce_out(2.0f32.mul_add(-t, 1.0))) / 2.0;
    }
    (1.0 + bounce_out(2.0f32.mul_add(t, -1.0))) / 2.0
}

pub fn back_in(t: f32) -> f32 {
    const K: f32 = 1.70158;
    let b = -(K * t.powi(2));
    (K + 1.0).mul_add(t.powi(3), b)
}

pub fn back_out(t: f32) -> f32 {
    const K: f32 = 1.70158;
    let a = (t - 1.0).powi(3);
    let b = (K + 1.0).mul_add(a, 1.0);
    K.mul_add((t - 1.0).powi(2), b)
}

pub fn back_in_out(t: f32) -> f32 {
    const K: f32 = 1.70158;
    const T: f32 = K * 1.525;
    if t < 0.5 {
        let b = const { (T + 1.0) * 2.0 }.mul_add(t, -T);
        return (2.0 * t).powi(2) * b / 2.0;
    }

    let a = (2.0f32.mul_add(t, -2.0)).powi(2);
    let b = (T + 1.0).mul_add(t.mul_add(2.0, -2.0), T);
    a.mul_add(b, 2.0) / 2.0
}

// cubic, quartic, quintic, etc
