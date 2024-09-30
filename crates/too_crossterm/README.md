## too_crossterm

This ia a `Backend` and `EventReader` implementation for [`too`](https://crates.io/too).

Usage:

```rust
use too_crossterm::{Term, Config};
let term = Term::setup(Config::default())?;
```

See docs for [`Config`](https://docs.rs/too_crossterm) for enabling different behaviors.

---

License: APACHE or MIT
