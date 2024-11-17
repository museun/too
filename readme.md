# too

too -- a different kind of tui library

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
        ui.slider(&mut value);
    }
}

fn main() -> std::io::Result<()> {
    let mut app = App::default()
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

struct App ;

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


License: MIT OR Apache-2.0
