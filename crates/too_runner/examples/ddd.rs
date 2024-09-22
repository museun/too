use too_crossterm::{Config, Term};

use too_runner::{
    math::{Pos2, Vec2},
    App, Context, Event, Key, Pixel, Rgba, Surface,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

#[derive(Copy, Clone, Debug, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const ZERO: Self = vec3(0.0, 0.0, 0.0);

    const fn new(x: f32, y: f32, z: f32) -> Self {
        vec3(x, y, z)
    }

    const fn splat(d: f32) -> Self {
        Self::new(d, d, d)
    }

    #[inline]
    fn cross(self, other: Self) -> Self {
        vec3(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    fn normalize(self) -> Self {
        self / self.length()
    }

    #[inline]
    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    #[inline]
    fn rotate(self, rot: Self) -> Self {
        self.rotate_z(rot.z).rotate_x(rot.y).rotate_y(rot.x)
    }

    #[inline]
    fn rotate_x(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }

    #[inline]
    fn rotate_y(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    #[inline]
    fn rotate_z(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }
}

const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        vec3(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        vec3(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl std::ops::Mul for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        vec3(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}
impl std::ops::Div for Vec3 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        vec3(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl std::ops::Add<f32> for Vec3 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self::Output {
        self + Vec3::splat(rhs)
    }
}
impl std::ops::Sub<f32> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        self - Vec3::splat(rhs)
    }
}
impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        self * Vec3::splat(rhs)
    }
}
impl std::ops::Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        self / Vec3::splat(rhs)
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}
impl std::ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}
impl std::ops::DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    color: Vec3,
}

impl Triangle {
    fn normal(self) -> Vec3 {
        let a = self.v1 - self.v0;
        let b = self.v2 - self.v0;
        a.cross(b).normalize()
    }

    fn hit(self, ro: Vec3, rd: Vec3) -> Option<f32> {
        let (e1, e2) = (self.v1 - self.v0, self.v2 - self.v0);

        let p = rd.cross(e2);
        let det = p.dot(e1);
        if det.abs() < 0.001 {
            return None;
        }

        let t = ro - self.v0;
        let inv_dev = 1.0 / det;
        let u = t.dot(p) * inv_dev;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross(e1);
        let v = rd.dot(q) * inv_dev;
        if v < 0.0 || v + u > 1.0 {
            return None;
        }

        let t = e2.dot(q) * inv_dev;
        if t < 0.0 {
            return None;
        }

        Some(t)
    }
}

#[derive(Debug, Default)]
struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn new(iter: impl IntoIterator<Item = Vec3>) -> Self {
        let mut i = 3;
        let mut triangles = vec![];

        let verts = iter.into_iter().collect::<Vec<_>>();
        let l = verts.len();

        while i < l {
            let triangle = Triangle {
                v0: vec3(verts[i - 1].x, verts[i - 1].y, verts[i - 1].z),
                v1: vec3(verts[i - 2].x, verts[i - 2].y, verts[i - 2].z),
                v2: vec3(verts[i - 3].x, verts[i - 3].y, verts[i - 3].z),
                color: vec3(verts[i].x, verts[i].y, verts[i].z),
            };
            triangles.push(triangle);
            i += 4
        }

        Self { triangles }
    }
}

impl std::ops::Add for Mesh {
    type Output = Self;
    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.triangles.append(&mut rhs.triangles);
        self
    }
}

struct Camera {
    pos: Vec3,
    focus: f32,
    rot: Vec3, // TODO Rot3 instead of Vec3
}

struct Demo {
    camera: Camera,
    screen: Screen,
    map: Mesh,
    velocity: Vec3,
    use_mask: bool,
    use_textures: bool,

    space_was_pressed: bool,
}

impl Demo {
    fn new(map: Mesh, (x, z): (f32, f32), size: Vec2) -> Self {
        Self {
            camera: Camera {
                pos: vec3(x, -20.0, z),
                focus: 2.0,
                rot: vec3(1.75, 0.0, 0.0),
            },
            screen: Screen::new(size),
            map,
            velocity: Vec3::ZERO,
            use_mask: false,
            use_textures: true,
            space_was_pressed: false,
        }
    }

    fn event(&mut self, ev: Event) {
        const SPEED: f32 = 2.0;

        const FORWARD: Vec3 = vec3(0.0, 0.0, SPEED);
        const BACK: Vec3 = vec3(0.0, 0.0, -SPEED);
        const LEFT: Vec3 = vec3(-SPEED, 0.0, 0.0);
        const RIGHT: Vec3 = vec3(SPEED, 0.0, 0.0);

        self.space_was_pressed = false;
        if let Event::KeyPressed { key, .. } = ev {
            match key {
                Key::Char('r') => self.camera.focus = 2.0,
                Key::Char('m') => self.use_mask = !self.use_mask,
                Key::Char('t') => self.use_textures = !self.use_textures,

                Key::Char(' ') => self.space_was_pressed = true,

                Key::Up if self.camera.rot.y < 1.5 => self.camera.rot.y += 0.05,
                Key::Down if self.camera.rot.y > -1.5 => self.camera.rot.y -= 0.05,

                Key::Left => self.camera.rot.x -= 0.05,
                Key::Right => self.camera.rot.x += 0.05,

                Key::Char('w') => self.camera.pos += FORWARD.rotate_y(self.camera.rot.x),
                Key::Char('s') => self.camera.pos += BACK.rotate_y(self.camera.rot.x),
                Key::Char('a') => self.camera.pos += LEFT.rotate_y(self.camera.rot.x),
                Key::Char('d') => self.camera.pos += RIGHT.rotate_y(self.camera.rot.x),

                _ => {}
            }
        }

        if let Event::MouseScroll { delta, .. } = ev {
            self.camera.focus += -delta.y as f32 * 0.05;
            self.camera.focus = self.camera.focus.clamp(0.5, 2.5);
        }

        if let Event::MouseDragHeld { delta, .. } = ev {
            self.camera.rot.x += delta.x as f32 * 0.05;
            self.camera.rot.y += -delta.y as f32 * 0.05;
        }

        if let Event::Resize(new_size) = ev {
            self.screen.resize(new_size);
        }
    }

    fn integrate(&mut self, dt: f32) {
        const GRAVITY: f32 = 80.0;

        if self.camera.pos.y > -0.5 {
            self.velocity.y = 0.0;
            if self.space_was_pressed {
                self.camera.pos.y = -0.6;
                self.velocity += vec3(0.0, -40.0, 0.0)
            }
            return;
        }

        self.camera.pos += Vec3 {
            y: self.velocity.y * dt + (GRAVITY * dt.powi(2)) / 2.0,
            ..Vec3::ZERO
        };
        self.velocity.y += GRAVITY * dt
    }
}

const CLOSE: f32 = 15.0;
const FAR: f32 = CLOSE * 3.0;
const DISTANT: f32 = FAR * 3.0;

struct Wall;
impl Wall {
    const COLOR: Vec3 = vec3(255.0, 182.0, 172.0);
    const FRONT: [Vec3; 8] = [
        vec3(0.0, 0.0, 0.0),
        vec3(CLOSE, 0.0, 0.0),
        vec3(CLOSE, CLOSE, 0.0),
        Self::COLOR,
        vec3(0.0, 0.0, 0.0),
        vec3(CLOSE, CLOSE, 0.0),
        vec3(0.0, CLOSE, 0.0),
        Self::COLOR,
    ];

    const BACK: [Vec3; 8] = [
        vec3(0.0, 0.0, CLOSE),
        vec3(CLOSE, 0.0, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        Self::COLOR,
        vec3(0.0, 0.0, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        vec3(0.0, CLOSE, CLOSE),
        Self::COLOR,
    ];

    const LEFT: [Vec3; 8] = [
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, CLOSE),
        vec3(0.0, CLOSE, 0.0),
        Self::COLOR,
        vec3(0.0, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(0.0, 0.0, CLOSE),
        Self::COLOR,
    ];

    const RIGHT: [Vec3; 8] = [
        vec3(CLOSE, 0.0, 0.0),
        vec3(CLOSE, 0.0, CLOSE),
        vec3(CLOSE, CLOSE, 0.0),
        Self::COLOR,
        vec3(CLOSE, CLOSE, 0.0),
        vec3(CLOSE, CLOSE, CLOSE),
        vec3(CLOSE, 0.0, CLOSE),
        Self::COLOR,
    ];
}

struct Floor;
impl Floor {
    const COLOR: Vec3 = vec3(255.0, 255.0, 255.0);
    const FACE: [Vec3; 8] = [
        vec3(0.0, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, 0.0),
        Self::COLOR,
        vec3(CLOSE, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        Self::COLOR,
    ];
}

struct Hole;
impl Hole {
    const COLOR: Vec3 = vec3(127.0, 127.0, 127.0);

    const FRONT: [Vec3; 8] = [
        vec3(0.0, CLOSE, 0.0),
        vec3(CLOSE, CLOSE, 0.0),
        vec3(CLOSE, DISTANT, 0.0),
        Self::COLOR,
        vec3(0.0, CLOSE, 0.0),
        vec3(CLOSE, DISTANT, 0.0),
        vec3(0.0, DISTANT, 0.0),
        Self::COLOR,
    ];

    const BACK: [Vec3; 8] = [
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        vec3(CLOSE, DISTANT, CLOSE),
        Self::COLOR,
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, DISTANT, CLOSE),
        vec3(0.0, DISTANT, CLOSE),
        Self::COLOR,
    ];

    const LEFT: [Vec3; 8] = [
        vec3(0.0, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(0.0, DISTANT, 0.0),
        Self::COLOR,
        vec3(0.0, DISTANT, 0.0),
        vec3(0.0, DISTANT, CLOSE),
        vec3(0.0, CLOSE, CLOSE),
        Self::COLOR,
    ];

    const RIGHT: [Vec3; 8] = [
        vec3(CLOSE, CLOSE, 0.0),
        vec3(CLOSE, CLOSE, CLOSE),
        vec3(CLOSE, DISTANT, 0.0),
        Self::COLOR,
        vec3(CLOSE, DISTANT, 0.0),
        vec3(CLOSE, DISTANT, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        Self::COLOR,
    ];
}

struct Start;
impl Start {
    const COLOR: Vec3 = vec3(0.0, 0.0, 255.0);
    const FACE: [Vec3; 8] = [
        vec3(0.0, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, 0.0),
        Self::COLOR,
        vec3(CLOSE, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        Self::COLOR,
    ];
}

struct End;
impl End {
    const COLOR: Vec3 = vec3(255.0, 0.0, 0.0);
    const FACE: [Vec3; 8] = [
        vec3(0.0, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, 0.0),
        Self::COLOR,
        vec3(CLOSE, CLOSE, 0.0),
        vec3(0.0, CLOSE, CLOSE),
        vec3(CLOSE, CLOSE, CLOSE),
        Self::COLOR,
    ];
}

fn load_map(map: &str) -> (Mesh, (f32, f32)) {
    let rows = map
        .lines()
        .map(<str>::trim)
        .filter(|c| !c.is_empty())
        .map(<str>::as_bytes)
        .collect::<Vec<_>>();

    let mut mesh = Mesh::default();
    let mut start = (0.0, 0.0);

    // TODO actually turn this into lexemes
    for (z, row) in rows.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            let mut grid = Mesh::default();
            match ch {
                b'X' => {
                    if z != 0 && rows[z - 1][x] != b'X' {
                        grid = grid + Mesh::new(Wall::FRONT)
                    }
                    if z != rows.len() - 1 && rows[z + 1][x] != b'X' {
                        grid = grid + Mesh::new(Wall::BACK)
                    }
                    if x != 0 && rows[z][x - 1] != b'X' {
                        grid = grid + Mesh::new(Wall::LEFT)
                    }
                    if x != row.len() - 1 && rows[z][x + 1] != b'X' {
                        grid = grid + Mesh::new(Wall::RIGHT)
                    }
                }

                b'.' => grid = Mesh::new(Floor::FACE),

                b'S' => {
                    start = (
                        x as f32 * CLOSE + CLOSE / 2.0,
                        z as f32 * CLOSE + CLOSE / 2.0,
                    );
                    grid = Mesh::new(Start::FACE)
                }

                b'E' => grid = Mesh::new(End::FACE),

                b' ' => {
                    if z != 0 && rows[z - 1][x] != b' ' {
                        grid = grid + Mesh::new(Hole::FRONT)
                    };
                    if z != rows.len() - 1 && rows[z + 1][x] != b' ' {
                        grid = grid + Mesh::new(Hole::BACK)
                    };
                    if x != 0 && rows[z][x - 1] != b' ' {
                        grid = grid + Mesh::new(Hole::LEFT)
                    };
                    if x != row.len() - 1 && rows[z][x + 1] != b' ' {
                        grid = grid + Mesh::new(Hole::RIGHT)
                    };
                }
                s => panic!("invalid map: `{s}`", s = s.escape_ascii()),
            };

            let v = vec3(x as f32 * CLOSE, 0.0, z as f32 * CLOSE);
            for triangle in &mut grid.triangles {
                triangle.v0 += v;
                triangle.v1 += v;
                triangle.v2 += v;
            }

            mesh.triangles.extend(grid.triangles);
        }
    }
    (mesh, start)
}

struct Screen {
    size: Vec2,
    buffer: Vec<Vec<(Vec3, f32)>>,
}

impl Screen {
    fn new(size: Vec2) -> Self {
        Self {
            size,
            buffer: vec![vec![(Vec3::ZERO, 0.0); size.x as usize]; size.y as usize],
        }
    }

    // TODO not this
    fn resize(&mut self, size: Vec2) {
        *self = Self::new(size)
    }

    fn update(&mut self, camera: &Camera, mesh: &Mesh) {
        // for (y, row) in self.buffer.iter_mut().enumerate() {
        self.buffer.par_iter_mut().enumerate().for_each(|(y, row)| {
            for (x, pixel) in row.iter_mut().enumerate() {
                let mut min_distance = f32::MAX;
                let mut color = Vec3::ZERO;

                let min_dim = self.size.x.min(self.size.y * 2) as f32 / 2.0;
                let pos = vec3(
                    (x as f32 - self.size.x as f32 / 2.0) / min_dim,
                    (y as f32 * 2.0 - self.size.y as f32 / 2.0) / min_dim,
                    camera.focus,
                )
                .rotate(camera.rot);

                let dist = pos.length();
                let dir = pos;
                let offset = camera.pos + pos;

                let mut index = None;
                for (i, triangle) in mesh.triangles.iter().enumerate() {
                    if let Some(distance) = triangle.hit(offset, dir) {
                        let distance = distance - dist;
                        if distance < min_distance {
                            min_distance = distance;
                            index = Some(i);
                        }
                    }
                }

                if let Some(index) = index {
                    let triangle = mesh.triangles[index];
                    let n = triangle.normal();
                    color = triangle.color
                        * (n.dot(dir * -1.0) / (dir.length() * n.length()))
                            .abs()
                            .clamp(0.5, 0.75);
                }

                color = color * (1.0 - min_distance / FAR);
                *pixel = (color, min_distance)
            }
        });
        // }
    }

    fn render(&mut self, surface: &mut Surface, use_textures: bool, use_mask: bool) {
        let mut pos = Pos2::ZERO;
        for y in 0..self.buffer.len() {
            for x in 0..self.buffer[y].len() {
                let item = self.buffer[y][x].0;
                let bg = Rgba::new(item.x as u8, item.y as u8, item.z as u8, 255);

                if use_textures {
                    surface.put(pos, Pixel::new(' ').bg(bg));
                }
                if use_mask {
                    let bg = if !use_textures {
                        Rgba::from_static("#000")
                    } else {
                        bg
                    };

                    draw_mask(item, bg, pos, surface);
                }
                pos.x += 1;
            }
            pos.x = 0;
            pos.y += 1;
        }

        // TODO not this
        *self = Self::new(self.size);
    }
}

fn draw_mask(item: Vec3, bg: Rgba, pos: Pos2, surface: &mut Surface) {
    const CHARS: [char; 15] = [
        ' ', '.', 'X', ':', '_', '~', '/', 'c', 'r', 'x', '*', '%', '#', '8', '@',
    ];

    let index = item.x + item.y + item.z;
    let index = index * CHARS.len() as f32 / (255 * 3) as f32;
    let index = (index as usize).clamp(0, CHARS.len() - 1);
    let pixel = Pixel::new(CHARS[index]).bg(bg);
    surface.put(pos, pixel);
}

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default().hook_panics(true))?;
    too_runner::run(DemoApp::new, term)
}

struct DemoApp {
    demo: Demo,
}

impl DemoApp {
    fn new(size: Vec2) -> Self {
        let (map, start) = load_map(
            r#"
            XXXXXXXXXXXX
            XS.........X
            X..... XXXXX
            X . .. ....X
            X.X........X
            X.X...X...EX
            XXXXXXXXXXXX
            "#,
        );

        Self {
            demo: Demo::new(map, start, size),
        }
    }
}

impl App for DemoApp {
    fn event(&mut self, event: Event, _: Context<'_>, _size: Vec2) {
        self.demo.event(event);
    }

    fn update(&mut self, dt: f32, _size: Vec2) {
        self.demo.integrate(dt);
    }

    fn render(&mut self, surface: &mut too_renderer::Surface) {
        self.demo.screen.update(&self.demo.camera, &self.demo.map);
        self.demo
            .screen
            .render(surface, self.demo.use_textures, self.demo.use_mask);
    }
}
