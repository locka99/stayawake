# Stayawake

The `stayawake` program sends a F15 keypress on a 30 second timer interval to stop Windows sleeping. F15 is
chosen since it is rarely mapped to anything yet still counts as activity.

The program shows an icon in the tray when it is running and the program can be terminated from the tray.

It might be useful for testing purposes (e.g. automation tests) where screen locking interferes with the test
or when you want to override the default sleep behaviour of a computer. e.g. maybe you have a remote desktop
inside of a physical desktop and don't want the remote desktop to go to sleep all the time.

## Rust

Code is written in Rust, so building / running should be as simple as:

```sh
cargo build --release # or
cargo run --release
```

It should build with either MSVC or GNU backends, i.e. `stable-x86_64-pc-windows-msvc` or `stable-x86_64-pc-windows-gnu`. You will require a Windows resource compiler, either `rc.exe` or `windres` in your path depending on MSVC or GNU (MSYS2).

The release build is optimized for size, < 250Kb depending on toolchain.

### Win32 in Rust appraisal

The code uses the Microsoft auto generated bindings for Win32.

I've written plenty of Win32 in C and C++ and just wanted to experience the Rust bindings to see how they work. Generally speaking they're okay and
offer some simple safeguards over raw Win32, e.g. structures have default initialisers, functions with optional parameters will use Option<> instead of null, and even handles have some basic protections to avoid casting.

There are a few downsides I encountered - Rust supports unions in unsafe structs which is fine but initialising them seems clumsy. In addition, some macros that exist in C/C++ through header files just aren't here. e.g. MAKEINTRESOURCE() is nowhere to be found, and also helpers LOWORD(), HIWORD() etc.

Overall I thnk the bindings are pretty decent and do allow safer programming (even in `unsafe` blocks) since it's harder to miss a parameter or inadvertently pass in junk. At the same time, Win32 is horribly inconsistent so there are structs that have cbSize fields that need to be initialized or weird combinations of args or flags
that don't make obvious sense unless you read the docs. So all that remains and could be dangerous.

Also this particular program is 90% Win32 so it doesn't benefit as much from being written in Rust as another program might. I think if the program had more application logic away from Win32, that the benefits of safe programming would be instantly felt.
