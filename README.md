# macro_state

Currently, Rust does not provide the ability to keep track of any sort of global
state between macro calls. This crate contains a series of macros (`write_state`
and `read_state`) that make it trivial to save and load global state (in the form
of string keys and values) at compile time, and in particular within multiple proc
macro calls.

## How it works

The `write_state` macro stores state in flat files that live in the target/build
directory for the current project. This ensures that when you do things like run
`cargo clean`, the current state values are automatically reset as well. In other
words, this crate automatically tracks with the build artifacts of whatever is
using it.

## Installation & Usage

First add `macro_state` as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
macro_state = "0.1.1"
```

Next import the macro:
```rust
#[macro_use]
extern crate macro_state;
```

Now you can call `write_state!` and `read_state!` anywhere in your crate, including
inside of proc macros!
```rust
write_state!("top of module", "value 2");

#[test]
fn test_write_state() {
    write_state!("top of method", "value 3");
    assert_eq!(read_state!("top_of_file"), "value 1");
    assert_eq!(read_state!("top of module"), "value 2");
    assert_eq!(read_state!("top of method"), "value 3");
}
```

After writing a call to `write_state`, such as `write_state!("my key", "my value");`, the state
you wrote will now be available at the specified key for use by `read_state!("my key");`
calls further down in your source code. Note that all of this happens at compile-time, so
make sure your source code and macro calls are laid out such that your `write_state` calls
will be compiled before your `read_state` calls.
