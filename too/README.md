# too

## A simple TUI library.

This provides an optimized renderer/surface implementation with abstractions over various terminal backends. A basic `App` trait can be used to implement your low-level applications.

### Examples

See some [examples](https://github.com/museun/too/tree/dev/too/examples)

A description of some of the examples:
|name|description|
|---|---|
|[ddd](https://github.com/museun/too/tree/dev/too/examples/ddd.rs)|A 3d raytracing demo (note, run this with `--release`)|
|[gradient](https://github.com/museun/too/tree/dev/too/examples/gradient.rs)|A nice gradient visualization|
|[hello](https://github.com/museun/too/tree/dev/too/examples/hello.rs)|Drag a square to see transparency effects|
|[layout](https://github.com/museun/too/tree/dev/too/examples/layout.rs)|Dynamic linear run-packing layouts|
|[rect_split](https://github.com/museun/too/tree/dev/too/examples/rect_split.rs)|Splitting a rectangle and resizing it|
|[torch](https://github.com/museun/too/tree/dev/too/examples/torch.rs)|like `less` with but an accurate dark mode|

### Backend implementations

- [too_crossterm](https://crates.io/too_crossterm)

---

License: APACHE or MIT
