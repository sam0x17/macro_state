#[allow(unused_imports)]
#[macro_use]
extern crate macro_state_macros;

#[macro_use]
extern crate lazy_static;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Result, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub use macro_state_macros::*;

lazy_static! {
    static ref COMPILE_TIME: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
}

/// A constant that will always resolve to the directory `macro_state`
/// will use to store state files. This is typically some sub-directory
/// of the `target` directory for the specified build environment.
/// You should never use this directly unless you know what you're doing.
pub const STATE_DIR: &'static str = env!("MACRO_STATE_DIR");

/// Returns the path of the internal file that would be used to
/// store state for the specified key, as a [PathBuf](std::path::PathBuf).
/// You should never use this directly unless you know what you're doing.
pub fn state_file_path(key: &str) -> PathBuf {
    let ctime = COMPILE_TIME.clone();
    let filename = format!("macro_state_{}_{}", key, ctime);
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
    match proc_read_state(key) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Clears the state value for the specified `key`, whether it exists or not
///
/// This should only be called from within proc macros!
pub fn proc_clear_state(key: &str) {
    let state_file = state_file_path(key);
    let state_file_path = state_file.to_str().unwrap();
    if proc_has_state(key) {
        fs::remove_file(state_file.clone())
            .expect(format!("could not delete file {}", state_file_path).as_str());
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

/// An analogue for [`append_state!`] that should only be used within proc macros.
///
/// Like [`proc_write_state`], but instead appends the specified `value` (newline-delimited) to
/// the state file. Newlines contained in the `value` are automatically escaped so you can
/// think of this as appending to a [`Vec<String>`] for all intents and purposes. Calling
/// [`proc_append_state`] is also more efficient than re-writing an entire state file via
/// [`proc_write_state`] since the low level append IO option is not used by
/// [`proc_write_state`].
///
/// If no state file for the specified `key` exists, it will be created automatically. In this
/// way, [`proc_append_state`] functions similar to how [`proc_init_state`] functions,
/// especially in the no-existing-file case.
///
/// Note that if [`proc_read_state`] is called on a [`proc_append_state`]-based state file,
/// newlines will be returned in the response.
///
/// # Examples
///
/// ```
/// use macro_state::*;
///
/// proc_append_state("my_key", "apples");
/// proc_append_state("my_key", "pears");
/// proc_append_state("my_key", "oh my!");
/// assert_eq!(proc_read_state("my_key").unwrap(), "apples\npears\noh my!\n");
/// assert_eq!(proc_read_state_vec("my_key"), vec!["apples", "pears", "oh my!"]);
/// ```
pub fn proc_append_state(key: &str, value: &str) -> Result<()> {
    let value = format!("{}\n", value.replace("\n", "\\n"));
    let state_file = state_file_path(key);
    match OpenOptions::new()
        .append(true)
        .create(true)
        .open(state_file)
    {
        Ok(mut file) => return file.write_all(value.as_bytes()),
        Err(e) => Err(e),
    }
}

/// An analogue for [`read_state_vec!`] that should only be used within proc macros.
///
/// Reads the state value for the specified key and parses it as a [`Vec<String>`] where each
/// new line is treated as a separate element in the [`Vec`]. Should be used in conjunction
/// with [`proc_append_state`] to read and write lists of values from macro state storage.
///
/// Returns: a [`Vec<String>`]. Throws a compiler error if the specified state key cannot be
/// found.
///
/// Note: If you need to initialize a state vec to an empty list, you can use
/// `proc_init_state("key", "\n")` which should result in an empty [`Vec<String>`] when the
/// state file is read by [`proc_read_state_vec`].
///
/// Note: This function is infallible -- if any issue occurs trying to read the specified key,
/// it is assumed that we should return an empty [`Vec`].
///
/// # Example
/// ```
/// use macro_state::*;
///
/// proc_append_state("my_key", "first item").unwrap();
/// proc_append_state("my_key", "2nd item").unwrap();
/// assert_eq!(proc_read_state_vec("my_key"), vec!["first item", "2nd item"]);
/// ```
pub fn proc_read_state_vec(key: &str) -> Vec<String> {
    let state_file = state_file_path(key);
    match fs::read_to_string(state_file) {
        Ok(mut value) => {
            if let Some(last) = value.as_str().chars().last() {
                if last == '\n' {
                    value = value[0..(value.len() - 1)].to_string();
                }
            }
            value
                .split("\n")
                .map(|item| item.replace("\\n", "\n"))
                .collect::<Vec<String>>()
        }
        Err(_) => Vec::<String>::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
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

    #[test]
    fn test_append_state() {
        append_state!("append_key", "first line");
        assert_eq!(read_state!("append_key"), "first line\n");
        append_state!("append_key", "2nd line");
        assert_eq!(read_state!("append_key"), "first line\n2nd line\n");
        append_state!("append_key", "3rd line");
        assert_eq!(
            read_state!("append_key"),
            "first line\n2nd line\n3rd line\n"
        );
        write_state!("append_key", "");
        append_state!("append_key", "first line");
        assert_eq!(read_state!("append_key"), "first line\n");
    }

    #[test]
    fn test_proc_append_state() {
        proc_append_state("append_key", "first line").unwrap();
        assert_eq!(proc_read_state("append_key").unwrap(), "first line\n");
        proc_append_state("append_key", "2nd line").unwrap();
        assert_eq!(
            proc_read_state("append_key").unwrap(),
            "first line\n2nd line\n"
        );
        proc_append_state("append_key", "3rd line").unwrap();
        assert_eq!(
            proc_read_state("append_key").unwrap(),
            "first line\n2nd line\n3rd line\n"
        );
        proc_write_state("append_key", "").unwrap();
        proc_append_state("append_key", "first line").unwrap();
        assert_eq!(proc_read_state("append_key").unwrap(), "first line\n");
    }

    #[test]
    fn test_read_state_vec() {
        append_state!("append2", "line 1");
        assert_eq!(read_state_vec!("append2"), vec!["line 1"]);
        append_state!("append2", "line 2");
        assert_eq!(read_state_vec!("append2"), vec!["line 1", "line 2"]);
        append_state!("append2", "line 3");
        assert_eq!(
            read_state_vec!("append2"),
            vec!["line 1", "line 2", "line 3"]
        );
        append_state!("append2", "");
        assert_eq!(
            read_state_vec!("append2"),
            vec!["line 1", "line 2", "line 3", ""]
        );
        assert_eq!(read_state_vec!("append748"), Vec::<String>::new());
    }

    #[test]
    fn test_proc_read_state_vec() {
        proc_append_state("append2", "line 1").unwrap();
        assert_eq!(proc_read_state_vec("append2"), vec!["line 1"]);
        proc_append_state("append2", "line 2").unwrap();
        assert_eq!(proc_read_state_vec("append2"), vec!["line 1", "line 2"]);
        proc_append_state("append2", "line 3").unwrap();
        assert_eq!(
            proc_read_state_vec("append2"),
            vec!["line 1", "line 2", "line 3"]
        );
        proc_append_state("append2", "").unwrap();
        assert_eq!(
            proc_read_state_vec("append2"),
            vec!["line 1", "line 2", "line 3", ""]
        );
    }

    #[test]
    fn test_append_state_newline_escaping() {
        append_state!("append3", "line 1");
        append_state!("append3", "hey\nwhat");
        append_state!("append3", "line 3");
        assert_eq!(
            read_state_vec!("append3"),
            vec!["line 1", "hey\nwhat", "line 3"]
        );
        append_state!("append4", "\n");
        assert_eq!(read_state_vec!("append4"), vec!["\n"]);
    }

    #[test]
    fn test_proc_append_state_newline_escaping() {
        proc_append_state("append3", "line 1").unwrap();
        proc_append_state("append3", "hey\nwhat").unwrap();
        proc_append_state("append3", "line 3").unwrap();
        assert_eq!(
            proc_read_state_vec("append3"),
            vec!["line 1", "hey\nwhat", "line 3"]
        );
        proc_append_state("append4", "\n").unwrap();
        assert_eq!(proc_read_state_vec("append4"), vec!["\n"]);
    }

    #[test]
    fn test_proc_state_functions() {
        assert_eq!(proc_has_state("proc A"), false);
        assert!(proc_read_state("proc B").is_err());
        proc_write_state("proc A", "val A").unwrap();
        assert!(proc_has_state("proc A"));
        assert_eq!(proc_read_state("proc A").unwrap(), "val A");
        assert_eq!(proc_init_state("proc A", "val B").unwrap(), "val A");
        proc_init_state("proc B", "val B").unwrap();
        assert_eq!(proc_read_state("proc B").unwrap(), "val B");
        assert!(proc_has_state("proc B"));
        proc_clear_state("proc B");
        proc_clear_state("proc A");
        assert_eq!(proc_has_state("proc A"), false);
        assert_eq!(proc_has_state("proc B"), false);
        assert!(proc_read_state("proc B").is_err());
        assert!(proc_read_state("proc A").is_err());
    }
}
