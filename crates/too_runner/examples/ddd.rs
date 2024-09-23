use std::usize;

use too_crossterm::{Config, Term};

use too_events::{Keybind, Modifiers};
use too_layout::{Align, Anchor2, LinearAllocator, LinearLayout};
use too_math::{vec2, Rect};
use too_runner::{
    color::Rgba,
    events::{Event, Key},
    math::{vec3, Pos2, Vec2, Vec3},
    pixel::Pixel,
    App, AppRunner, Backend, Context, SurfaceMut,
};

use rayon::iter::*;
use too_shapes::{Fill, Text};

trait RotVec3 {
    fn rotate(self, rot: Self) -> Self;
    fn rotate_x(self, angle: f32) -> Self;
    fn rotate_y(self, angle: f32) -> Self;
    fn rotate_z(self, angle: f32) -> Self;
}

impl RotVec3 for Vec3 {
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

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    color: Vec3,
}

impl Triangle {
    const fn new(v2: [f32; 3], v1: [f32; 3], v0: [f32; 3], color: Vec3) -> Self {
        Self {
            v2: vec3(v2[0], v2[1], v2[2]),
            v1: vec3(v1[0], v1[1], v1[2]),
            v0: vec3(v0[0], v0[1], v0[2]),
            color,
        }
    }

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

impl Extend<Triangle> for Mesh {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Triangle>,
    {
        self.triangles.extend(iter);
    }
}

struct Camera {
    pos: Vec3,
    focus: f32,
    view_distance: f32,
    rot: Vec3,
}

struct Keybinds {
    reset: Keybind,
    toggle_help: Keybind,
    toggle_fps: Keybind,
    jump: Keybind,
    move_forward: Keybind,
    move_backward: Keybind,
    move_left: Keybind,
    move_right: Keybind,
    rotate_camera_up: Keybind,
    rotate_camera_down: Keybind,
    rotate_camera_left: Keybind,
    rotate_camera_right: Keybind,
}

struct Demo {
    camera: Camera,
    screen: Screen,
    map: Mesh,
    velocity: Vec3,
    keybinds: Keybinds,
    show_help: bool,
    space_was_pressed: bool,
}

impl Demo {
    fn new(map: Mesh, pos: PlayerPos, keybinds: Keybinds) -> Self {
        Self {
            camera: Camera {
                pos: vec3(pos.x, -20.0, pos.z),
                focus: 2.0,
                view_distance: 1.0,
                rot: vec3(1.75, 0.0, 0.0),
            },
            screen: Screen::new(Vec2::ZERO),
            map,
            velocity: Vec3::ZERO,
            keybinds,
            show_help: false,
            space_was_pressed: false,
        }
    }

    fn event(&mut self, ev: Event, mut ctx: Context<'_, impl Backend>) {
        const SPEED: f32 = 2.0;

        const FORWARD: Vec3 = vec3(0.0, 0.0, SPEED);
        const BACK: Vec3 = vec3(0.0, 0.0, -SPEED);
        const LEFT: Vec3 = vec3(-SPEED, 0.0, 0.0);
        const RIGHT: Vec3 = vec3(SPEED, 0.0, 0.0);

        self.space_was_pressed = false;

        if ev.is_keybind_pressed(self.keybinds.reset) {
            self.camera.view_distance = 1.0;
        }

        if ev.is_keybind_pressed(self.keybinds.toggle_help) {
            self.show_help = !self.show_help
        }

        if ev.is_keybind_pressed(self.keybinds.toggle_fps) {
            ctx.toggle_fps()
        }
        if ev.is_keybind_pressed(self.keybinds.jump) {
            self.space_was_pressed = true
        }

        if ev.is_keybind_pressed(self.keybinds.move_forward) {
            self.camera.pos += FORWARD.rotate_y(self.camera.rot.x)
        }
        if ev.is_keybind_pressed(self.keybinds.move_backward) {
            self.camera.pos += BACK.rotate_y(self.camera.rot.x)
        }
        if ev.is_keybind_pressed(self.keybinds.move_left) {
            self.camera.pos += LEFT.rotate_y(self.camera.rot.x)
        }
        if ev.is_keybind_pressed(self.keybinds.move_right) {
            self.camera.pos += RIGHT.rotate_y(self.camera.rot.x)
        }

        if ev.is_keybind_pressed(self.keybinds.rotate_camera_up) && self.camera.rot.y < 1.5 {
            self.camera.rot.y += 0.05
        }
        if ev.is_keybind_pressed(self.keybinds.rotate_camera_down) && self.camera.rot.y > -1.5 {
            self.camera.rot.y -= 0.05
        }
        if ev.is_keybind_pressed(self.keybinds.rotate_camera_left) {
            self.camera.rot.x -= 0.05
        }
        if ev.is_keybind_pressed(self.keybinds.rotate_camera_right) {
            self.camera.rot.x += 0.05
        }

        if let Event::MouseScroll { delta, .. } = ev {
            self.camera.view_distance += -delta.y as f32 * 0.1;
            self.camera.view_distance = self.camera.view_distance.clamp(0.5, 10.0);
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

        self.camera.pos.y += self.velocity.y * dt + (GRAVITY * dt.powi(2)) / 2.0;
        self.velocity.y += GRAVITY * dt
    }

    fn render_scene(&mut self, surface: &mut SurfaceMut) {
        self.screen.raytrace(&self.camera, &self.map);

        let mut pos = Pos2::ZERO;
        for y in 0..self.screen.buffer.len() {
            for x in 0..self.screen.buffer[y].len() {
                let item = self.screen.buffer[y][x].0;
                let bg = Rgba::new(item.x as u8, item.y as u8, item.z as u8, 255);
                surface.put(pos, Pixel::new(' ').bg(bg));
                pos.x += 1;
            }
            pos.x = 0;
            pos.y += 1;
        }
    }

    fn show_keybinds(&mut self, surface: &mut SurfaceMut) {
        let rect = surface.rect();

        let column = &[
            ("move_forward", self.keybinds.move_forward.to_string()),
            ("move_backward", self.keybinds.move_backward.to_string()),
            ("move_left", self.keybinds.move_left.to_string()),
            ("move_right", self.keybinds.move_right.to_string()),
            ("camera_up", self.keybinds.rotate_camera_up.to_string()),
            ("camera_down", self.keybinds.rotate_camera_down.to_string()),
            ("camera_left", self.keybinds.rotate_camera_left.to_string()),
            (
                "camera_right",
                self.keybinds.rotate_camera_right.to_string(),
            ),
            ("reset", self.keybinds.reset.to_string()),
            ("toggle_help", self.keybinds.toggle_help.to_string()),
            ("toggle_fps", self.keybinds.toggle_fps.to_string()),
            ("jump", self.keybinds.jump.to_string()),
        ];

        let (max_left, max_right) = column.iter().fold((0, 0), |(max_left, max_right), (a, b)| {
            (max_left.max(a.len() + 1), max_right.max(b.len()))
        });

        let rect = Rect::from_center_size(
            rect.center(),
            vec2((max_left + max_right + 3) as i32, column.len() as _),
        );

        surface
            .crop(rect.expand2(vec2(1, 0)))
            .draw(Fill::new("#222D"));

        let mut surface = surface.crop(rect);
        let mut y = 0;

        for (label, bind) in column {
            let shape = Text::new(format!(
                "{label}{s: <pad$}| {bind}",
                s = ' ',
                pad = max_left + 1 - label.len()
            ));
            surface.crop(rect.translate(vec2(0, y))).draw(shape);
            y += 1;
        }
    }
}

struct Geom;
impl Geom {
    const NEAR: f32 = 15.0;
    const FAR: f32 = Self::NEAR * 3.0;

    const WALL_COLOR: Vec3 = vec3(255.0, 182.0, 172.0);
    const FLOOR_COLOR: Vec3 = vec3(255.0, 255.0, 255.0);
    const HOLE_COLOR: Vec3 = vec3(127.0, 127.0, 127.0);
    const START_COLOR: Vec3 = vec3(0.0, 0.0, 255.0);
    const END_COLOR: Vec3 = vec3(255.0, 0.0, 0.0);

    // TODO make primitives and construct these from those
    const WALL_FRONT: [Triangle; 2] = [
        Triangle::new(
            [0.0, 0.0, 0.0],
            [15.0, 0.0, 0.0],
            [15.0, 15.0, 0.0],
            Self::WALL_COLOR,
        ),
        Triangle::new(
            [0.0, 0.0, 0.0],
            [15.0, 15.0, 0.0],
            [0.0, 15.0, 0.0],
            Self::WALL_COLOR,
        ),
    ];
    const WALL_BACK: [Triangle; 2] = [
        Triangle::new(
            [0.0, 0.0, 15.0],
            [15.0, 0.0, 15.0],
            [15.0, 15.0, 15.0],
            Self::WALL_COLOR,
        ),
        Triangle::new(
            [0.0, 0.0, 15.0],
            [15.0, 15.0, 15.0],
            [0.0, 15.0, 15.0],
            Self::WALL_COLOR,
        ),
    ];
    const WALL_LEFT: [Triangle; 2] = [
        Triangle::new(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 15.0],
            [0.0, 15.0, 0.0],
            Self::WALL_COLOR,
        ),
        Triangle::new(
            [0.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [0.0, 0.0, 15.0],
            Self::WALL_COLOR,
        ),
    ];
    const WALL_RIGHT: [Triangle; 2] = [
        Triangle::new(
            [15.0, 0.0, 0.0],
            [15.0, 0.0, 15.0],
            [15.0, 15.0, 0.0],
            Self::WALL_COLOR,
        ),
        Triangle::new(
            [15.0, 15.0, 0.0],
            [15.0, 15.0, 15.0],
            [15.0, 0.0, 15.0],
            Self::WALL_COLOR,
        ),
    ];
    const FLOOR_FACE: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 0.0],
            Self::FLOOR_COLOR,
        ),
        Triangle::new(
            [15.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 15.0],
            Self::FLOOR_COLOR,
        ),
    ];
    const HOLE_FRONT: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 0.0],
            [15.0, 15.0, 0.0],
            [15.0, 135.0, 0.0],
            Self::HOLE_COLOR,
        ),
        Triangle::new(
            [0.0, 15.0, 0.0],
            [15.0, 135.0, 0.0],
            [0.0, 135.0, 0.0],
            Self::HOLE_COLOR,
        ),
    ];
    const HOLE_BACK: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 15.0],
            [15.0, 135.0, 15.0],
            Self::HOLE_COLOR,
        ),
        Triangle::new(
            [0.0, 15.0, 15.0],
            [15.0, 135.0, 15.0],
            [0.0, 135.0, 15.0],
            Self::HOLE_COLOR,
        ),
    ];
    const HOLE_LEFT: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [0.0, 135.0, 0.0],
            Self::HOLE_COLOR,
        ),
        Triangle::new(
            [0.0, 135.0, 0.0],
            [0.0, 135.0, 15.0],
            [0.0, 15.0, 15.0],
            Self::HOLE_COLOR,
        ),
    ];
    const HOLE_RIGHT: [Triangle; 2] = [
        Triangle::new(
            [15.0, 15.0, 0.0],
            [15.0, 15.0, 15.0],
            [15.0, 135.0, 0.0],
            Self::HOLE_COLOR,
        ),
        Triangle::new(
            [15.0, 135.0, 0.0],
            [15.0, 135.0, 15.0],
            [15.0, 15.0, 15.0],
            Self::HOLE_COLOR,
        ),
    ];
    const START_FACE: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 0.0],
            Self::START_COLOR,
        ),
        Triangle::new(
            [15.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 15.0],
            Self::START_COLOR,
        ),
    ];
    const END_FACE: [Triangle; 2] = [
        Triangle::new(
            [0.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 0.0],
            Self::END_COLOR,
        ),
        Triangle::new(
            [15.0, 15.0, 0.0],
            [0.0, 15.0, 15.0],
            [15.0, 15.0, 15.0],
            Self::END_COLOR,
        ),
    ];
}

struct PlayerPos {
    x: f32,
    z: f32,
}

fn load_map(map: &str) -> (Mesh, PlayerPos) {
    fn check_faces<const M: u8>(
        rows: &[&[u8]],
        len: usize,
        (z, x): (usize, usize),
        grid: &mut Mesh,
        faces: [[Triangle; 2]; 4],
    ) {
        if z != 0 && rows[z - 1][x] != M {
            grid.extend(faces[0])
        }
        if z != rows.len() - 1 && rows[z + 1][x] != M {
            grid.extend(faces[1])
        }
        if x != 0 && rows[z][x - 1] != M {
            grid.extend(faces[2])
        }
        if x != len - 1 && rows[z][x + 1] != M {
            grid.extend(faces[3])
        }
    }

    let rows = map
        .lines()
        .map(<str>::trim)
        .filter(|c| !c.is_empty())
        .map(<str>::as_bytes)
        .collect::<Vec<_>>();

    let mut mesh = Mesh::default();

    let mut pos = PlayerPos { x: 0.0, z: 0.0 };

    for (z, row) in rows.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            let mut grid = Mesh::default();
            match ch {
                b'.' => grid.extend(Geom::FLOOR_FACE),

                b'S' => {
                    pos.x = x as f32 * Geom::NEAR + Geom::NEAR / 2.0;
                    pos.z = z as f32 * Geom::NEAR + Geom::NEAR / 2.0;
                    grid.extend(Geom::START_FACE)
                }

                b'E' => grid.extend(Geom::END_FACE),

                b'W' => {
                    let wall = [
                        Geom::WALL_FRONT,
                        Geom::WALL_BACK,
                        Geom::WALL_LEFT,
                        Geom::WALL_RIGHT,
                    ];
                    check_faces::<b'W'>(&rows, row.len(), (z, x), &mut grid, wall)
                }

                b'H' => {
                    let hole = [
                        Geom::HOLE_FRONT,
                        Geom::HOLE_BACK,
                        Geom::HOLE_LEFT,
                        Geom::HOLE_RIGHT,
                    ];
                    check_faces::<b'H'>(&rows, row.len(), (z, x), &mut grid, hole)
                }

                s => panic!("invalid map: `{s}`", s = s.escape_ascii()),
            }

            let v = vec3(x as f32 * Geom::NEAR, 0.0, z as f32 * Geom::NEAR);
            for triangle in &mut grid.triangles {
                triangle.v0 += v;
                triangle.v1 += v;
                triangle.v2 += v;
            }

            mesh.extend(grid.triangles);
        }
    }

    (mesh, pos)
}

struct Screen {
    size: Vec2,
    // TODO use a 1d buffer
    buffer: Vec<Vec<(Vec3, f32)>>,
}

impl Screen {
    fn new(size: Vec2) -> Self {
        Self {
            size,
            buffer: vec![vec![(Vec3::ZERO, 0.0); size.x as usize]; size.y as usize],
        }
    }

    fn resize(&mut self, size: Vec2) {
        *self = Self::new(size)
    }

    fn raytrace(&mut self, camera: &Camera, mesh: &Mesh) {
        self.buffer.par_iter_mut().enumerate().for_each(|(y, row)| {
            for (x, pixel) in row.iter_mut().enumerate() {
                pixel.1 = 0.0;

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

                color = color * (1.0 - min_distance / (camera.view_distance * Geom::FAR));
                *pixel = (color, min_distance)
            }
        });
    }
}

impl App for Demo {
    fn initial_size(&mut self, size: Vec2) {
        self.screen = Screen::new(size)
    }

    fn event(&mut self, event: Event, ctx: Context<'_, impl Backend>, _size: Vec2) {
        self.event(event, ctx);
    }

    fn update(&mut self, dt: f32, _size: Vec2) {
        self.integrate(dt);
    }

    fn render(&mut self, surface: &mut too_renderer::SurfaceMut) {
        self.render_scene(surface);

        if self.show_help {
            self.show_keybinds(surface);
        }
    }
}

fn main() -> std::io::Result<()> {
    let (map, start_pos) = load_map(
        r#"
        WWWWWWWWWWWW
        WS.........W
        W.....HWWWWW
        WH.H..H....W
        W.W........W
        W.W...W...EW
        WWWWWWWWWWWW
        "#,
    );

    let keybinds = Keybinds {
        reset: 'r'.into(),
        toggle_help: '?'.into(),
        toggle_fps: 't'.into(),
        jump: ' '.into(),
        move_forward: 'w'.into(),
        move_backward: 's'.into(),
        move_left: 'a'.into(),
        move_right: 'd'.into(),
        rotate_camera_up: Key::Up.into(),
        rotate_camera_down: Key::Down.into(),
        rotate_camera_left: Key::Left.into(),
        rotate_camera_right: Key::Right.into(),
    };

    let term = Term::setup(Config::default().hook_panics(true))?;
    Demo::new(map, start_pos, keybinds).run(term)
}
