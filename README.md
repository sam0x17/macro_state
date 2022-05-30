# macro_state

Currently, Rust does not provide the ability to keep track of any sort of global
state between macro calls. This crate contains a series of macros (`write_state`
and `read_state`) that make it trivial to save and load global state (in the form
of string keys and values) at compile time, and in particular within multiple proc
macro calls.

## Storage

The `write_state` macro stores state in flat files that live in the target/build
directory for the current project. This ensures that when you do things like run
`cargo clean`, the current state values are automatically reset as well. In other
words, this crate automatically tracks with the build artifacts of whatever is
using it.

## Usage

Using `macro_state` is simple. Just import that macros and write a call to `write_state`,
such as `write_state!("my key", "my value");`. The state you wrote will now be available
at the specified key for use by `read_state!("my key");` calls further down in your source
code. Note that all of this happens at compile-time, so make sure your source code and
macro calls are laid out such that your `write_state` calls will be compiled before your
`read_state` calls.
