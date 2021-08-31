# Mega Drive Util crate

Useful functions and features that aid in development

## Modules
The util crate provides two modules: the [random number generator mod](src/rng.rs) and the [panic mod](src/panic.rs)

### The `rng` module
The rng module provides a public struct that keeps track of a random number that is permuted by the horizontal video
counter. Instantiate and use as follows:

```rust
use megadrive_util::rng::PseudoRng;

#[no_mangle]
pub fn main() -> ! {
    let mut rng = PseudoRng::from_seed(42);

    // game loop
    loop {
        _current_random = rng.random();
    }
}
```

### The `panic` module
The panic module exposes no public functions, types or structs, but implements the `#[panic_handler]`, so you don't have
to. `#[no_std]` libs require a panic handler, as there is no standard library to provide it for you and the Rust 
compiler requires to know what to do in case of a panic.

The default panic handler provided by the `util` crate does more than just boilerplate - it actually shows an error
message when encountering a panic, so it can help debug the problem. At this point, however, there is no support yet for
passing error messages from `.expect()` calls, it will just show a generic "Panic!" message.

The panic handler is behind a `#[cfg(feature = 'panic_handler']` feature gate that is enabled by default. So, you can
disable this with `--default-features` on installing the `util` crate or specifying
```toml
[dependencies]
megadrive-util = { path = "path/to/libs/megadrive-util", default-features = false }
```
to opt out of the default panic handler.
