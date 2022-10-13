# macro_state

![crates.io](https://img.shields.io/crates/v/macro_state.svg) ![GitHub Workflow Status
(branch)](https://img.shields.io/github/workflow/status/sam0x17/macro_state/CI%20Checks/main)
![docs.rs](https://img.shields.io/docsrs/macro_state)

Currently, Rust does not provide the ability to keep track of any sort of global state between
macro calls out of the box.

This crate contains a series of macros that make it trivial to save and load global state (in
the form of string keys and values) at compile time and from within proc macros. State that was
set at compile-time can also be read directly by runtime code, if needed.

## Functionality

The [`write_state!`](https://docs.rs/macro_state/latest/macro_state/macro.write_state.html) and
[`append_state!`](https://docs.rs/macro_state/latest/macro_state/macro.append_state.html)
macros store state in flat files that live in the target build directory for the current
project. This ensures that when you do things like run `cargo clean` or change your code, the
current state values are automatically reset as well. In other words, this crate automatically
tracks with the build artifacts of whatever is using it.

After compilation, whatever values were present at compile-time are baked into the resulting
binary.

### Macros

Currently, we offer the following macros:
* [`write_state!("key","value")`](https://docs.rs/macro_state/latest/macro_state/macro.write_state.html)
  writes `"value"` as the value for the key `"key"`
* [`read_state!("key")`](https://docs.rs/macro_state/latest/macro_state/macro.read_state.html)
  returns the value for the key `"key"`, issuing a compiler error if it can't be found
* [`init_state!("key","value")`](https://docs.rs/macro_state/latest/macro_state/macro.init_state.html)
  if the key `"key"` has a value, returns it, otherwise sets it to `"value"` and also returns
  it. This can be used to quickly initialize a key value pair that may have existing data
* [`has_state!("key")`](https://docs.rs/macro_state/latest/macro_state/macro.has_state.html)
  returns a boolean indicating whether a value has been stored for the key `"key"`
* [`clear_state!("key")`](https://docs.rs/macro_state/latest/macro_state/macro.clear_state.html)
  clears any existing state value for key `"key"`, if it exists
* [`append_state!("key","value")`](https://docs.rs/macro_state/latest/macro_state/macro.append_state.html)
  appends `"value"` to the value list for the specified key. Used in conjunction with
  `read_state_vec!` to add to and manage lists within state files.
* [`read_state_vec!("key")`](https://docs.rs/macro_state/latest/macro_state/macro.read_state_vec.html)
  reads the state file for key `"key"` as a `Vec<String>`. Used in conjunction with
  `append_state!` to manage manage lists within state files.

### Within Proc Macros

Non-macro analogues for all of the macros listed above can be found
[here](https://docs.rs/macro_state/latest/macro_state/index.html#functions). These analogues
all begin with `proc_`, such as `proc_read_state`, and **should only be used within proc
macros**.

Using these functions anywhere but within a proc macro will result in broken/undefined
behavior.

## Installation

First add `macro_state` as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
macro_state = "0.1.9"
```

Next import the macro:
```rust
#[macro_use]
extern crate macro_state;
```

## Usage

Now you can call `write_state!` and `read_state!` anywhere in your crate, including inside of
proc macros!

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
you wrote will now be available at the specified key for use by `read_state!("my key");` calls
further down in your source code. Note that all of this happens at compile-time, so make sure
your source code and macro calls are laid out such that your `write_state` calls will be
compiled before your `read_state` calls.
