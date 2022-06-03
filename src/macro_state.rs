#[allow(unused_imports)]
#[macro_use]
extern crate macros;
use std::fs;
use std::fs::File;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};

pub use macros::*;

/// A constant that will always resolve to the directory `macro_state`
/// will use to store state files. This is typically some sub-directory
/// of the `target` directory for the specified build environment.
/// You should never use this directly unless you know what you're doing.
pub const STATE_DIR: &'static str = env!("MACRO_STATE_DIR");

/// Returns the path of the internal file that would be used to
/// store state for the specified key, as a [PathBuf](std::path::PathBuf).
/// You should never use this directly unless you know what you're doing.
pub fn state_file_path(key: &str) -> PathBuf {
    let filename = format!("macro_state_{}", key);
    let mut buf = PathBuf::new();
    buf.push(STATE_DIR);
    buf.push(filename.as_str());
    buf
}

/// Attempts to write `value` as the value for the key `key`.
///
/// This should only be called from within proc macros!
pub fn proc_write_state(key: &str, value: &str) -> Result<()> {
    let mut file = File::create(state_file_path(key))?;
    file.write_all(value.as_bytes())
}

/// Attempts to read the value for the specified `key`
///
/// This should only be called from within proc macros!
pub fn proc_read_state(key: &str) -> Result<String> {
    let state_file = state_file_path(key);
    fs::read_to_string(state_file)
}

/// Checks whether a value has been defined for the specified `key`
///
/// This should only be called from within proc macros!
pub fn proc_has_state(key: &str) -> bool {
    let state_file = state_file_path(key);
    Path::exists(&state_file) && Path::is_file(&state_file)
}

/// Clears the state value for the specified `key`, whether it exists or not
///
/// This should only be called from within proc macros!
pub fn proc_clear_state(key: &str) {
    if proc_has_state(key) {
        fs::remove_file(key).expect(format!("could not delete file {}", key).as_str());
    }
}

/// If a state value is already defined for `key`, returns it, otherwise
/// writes `default_value` as the state value for `key` and returns `default_value`
///
/// This should only be called from within proc macros!
pub fn proc_init_state(key: &str, default_value: &str) -> Result<String> {
    match proc_read_state(key) {
        Ok(existing) => Ok(existing),
        Err(_) => match proc_write_state(key, default_value) {
            Ok(_) => Ok(String::from(default_value)),
            Err(err) => Err(err),
        },
    }
}

#[cfg(test)]
mod tests {
    write_state!("top of module", "value 2");

    #[test]
    fn test_write_state() {
        write_state!("top of method", "value 3");
        assert_eq!(read_state!("top of module"), "value 2");
        assert_eq!(read_state!("top of method"), "value 3");
    }

    #[test]
    fn test_rewriting_state() {
        write_state!("key 1", "value 4");
        assert_eq!(read_state!("key 1"), "value 4");
        write_state!("key 1", "value 5");
        assert_eq!(read_state!("key 1"), "value 5");
    }

    #[test]
    fn test_has_state() {
        assert_eq!(has_state!("key A"), false);
        write_state!("key A", "value 6");
        assert_eq!(has_state!("key A"), true);
        assert_eq!(read_state!("key A"), "value 6");
    }

    #[test]
    fn test_clear_state() {
        write_state!("key B", "value 7");
        assert_eq!(read_state!("key B"), "value 7");
        clear_state!("key B");
        assert_eq!(has_state!("key B"), false);
    }

    #[test]
    fn test_init_state() {
        write_state!("key C", "value 8");
        assert_eq!(init_state!("key C", "value -8"), "value 8");
        assert_eq!(init_state!("key D", "value 9"), "value 9");
        assert_eq!(init_state!("key C", "value -8"), "value 8");
        assert_eq!(init_state!("key D", "value 9"), "value 9");
    }
}
