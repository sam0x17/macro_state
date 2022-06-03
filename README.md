# macro_state

![crates.io](https://img.shields.io/crates/v/macro_state.svg) ![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/sam0x17/macro_state/CI%20Checks/main)

Currently, Rust does not provide the ability to keep track of any sort of global
state between macro calls out of the box.

This crate contains a series of macros that make it trivial to save and load global
state (in the form of string keys and values) at compile time and from within proc
macros. State that was set at compile-time can also be read directly by runtime
code, if needed.

## Functionality

The `write_state!` macro stores state in flat files that live in the target build
directory for the current project. This ensures that when you do things like run
`cargo clean`, the current state values are automatically reset as well. In other
words, this crate automatically tracks with the build artifacts of whatever is
using it.

Currently, we offer the following macros:
* `write_state!("key", "value")`: write `"value"` as the value for the key "key"
* `read_state!("key")`: returns the value for the key "key", panicking if it can't be found
* `init_state!("key", "value")`: if the key "key" has a value, returns it, otherwise sets it to "value" and also returns it. This can be used to quickly initialize a key/value pair that may have existing data
* `has_state!("key")`: returns a boolean indicating whether a value has been stored for the key "key"

Non-macro analogue functions (`write_state`, `read_state`, etc) are provided for
all of the above macros. Note that these non-macro analogues should _only_ be called
from within a proc macro. They will not work properly if you use them outside of
proc macro land.

## Installation

First add `macro_state` as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
macro_state = "0.1.3"
```

Next import the macro:
```rust
#[macro_use]
extern crate macro_state;
```

## Usage

Now you can call `write_state!` and `read_state!` anywhere in your crate, including
inside of proc macros!
```rust
write_state!("top of module", "value 1");

#[test]
fn test_write_state() {
    write_state!("top of method", "value 2");
    assert_eq!(read_state!("top of module"), "value 1");
    assert_eq!(read_state!("top of method"), "value 2");
}
```

After writing a call to `write_state`, such as `write_state!("my key", "my value");`, the state
you wrote will now be available at the specified key for use by `read_state!("my key");`
calls further down in your source code. Note that all of this happens at compile-time, so
make sure your source code and macro calls are laid out such that your `write_state` calls
will be compiled before your `read_state` calls.
