This program repeatedly sends an F15 keypress on a timer to stop Windows sleeping. F15 is
chosen since it is rarely mapped to anything yet still counts as activity.

It might be useful for testing purposes or when you want to override the default
sleep behaviour of a computer.

Code is written in Rust, so building / running it is simple as:

```
cargo build --release
cargo run --release
```
