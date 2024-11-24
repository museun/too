# too

[![Documentation][docs_badge]][docs]
[![Crates][crates_badge]][crates]

too -- a different kind of tui library

## Feature flags

| Flag       | Description                                                                         | Default |
| ---------- | ----------------------------------------------------------------------------------- | ------- |
| `terminal` | enable the terminal backend                                                         | `true`  |
| `sync`     | enable `Send`+`Sync` wrappers                                                       | `false` |
| `profile`  | enable [`profiling`](https://docs.rs/profiling/1.0.16/profiling/index.html) support | `false` |

---

## Simple examples

### Centering some text:

```rust
fn main() -> std::io::Result<()> {
    too::run(|ui| {
        ui.center(|ui| ui.label("hello world"));
    })
}
```

### A pair of buttons to increment and decrement a counter

```rust
fn main() -> std::io::Result<()> {
    let mut counter = 0;
    too::run(|ui| {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                if ui.button("add 1").clicked() {
                    counter += 1;
                }
                if ui.button("subtract 1").clicked() {
                    counter -= 1;
                }
            });
            ui.label(counter)
        });
    })
}
```

### Storing state in a struct

```rust
use too::view::Ui;

#[derive(Default)]
struct App {
    value: f32
}

impl App {
    fn view(&mut self, ui: &Ui) {
        ui.slider(&mut self.value);
    }
}

fn main() -> std::io::Result<()> {
    let mut app = App::default();
    too::run(|ui| app.view(ui))
}
```

### Storing state seperately from an application

```rust
use too::view::Ui;

#[derive(Default)]
struct State {
    value: f32
}

struct App;

impl App {
    fn view(&self, state: &mut State, ui: &Ui) {
        ui.slider(&mut state.value);
    }
}

fn main() -> std::io::Result<()> {
    let app = App;
    let mut state = State::default();
    too::run(|ui| app.view(&mut state, ui))
}
```

Some pre-made views are provided in: [`too::views`](crate::views)

---

License: MIT OR Apache-2.0

[docs_badge]: https://docs.rs/too/badge.svg
[crates_badge]: https://img.shields.io/crates/v/too.svg
[docs]: https://docs.rs/too
[crates]: https://crates.io/crates/too
