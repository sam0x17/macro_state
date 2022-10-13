extern crate proc_macro;

#[macro_use]
extern crate lazy_static;

extern crate quote;
extern crate syn;

extern crate derive_syn_parse;
use derive_syn_parse::Parse;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use proc_macro::TokenStream;
use quote::quote;
use syn::token::Comma;
use syn::{parse_macro_input, LitStr};

lazy_static! {
    static ref COMPILE_TIME: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
}

fn state_file_path(key: &str) -> PathBuf {
    let ctime = COMPILE_TIME.clone();
    let filename = format!("macro_state_{}_{}", key, ctime);
    let mut buf = PathBuf::new();
    buf.push(env!("MACRO_STATE_DIR"));
    buf.push(filename.as_str());
    buf
}

fn quote_io_error(e: Error) -> TokenStream {
    let msg = e.to_string();
    quote!(compile_error!(#msg)).into()
}

#[derive(Parse)]
struct WriteStateInput {
    key: LitStr,
    _comma: Comma,
    value: LitStr,
}

/// Writes the specified `value` as the state for the specified state `key`. `macro_state`
/// itself functions as a compile-time key-value store, and this is how you write a value to a
/// specific key.
///
/// # Example
/// ```rust
/// write_state!("my key", "some value");
/// ```
#[proc_macro]
pub fn write_state(items: TokenStream) -> TokenStream {
    let args = parse_macro_input!(items as WriteStateInput);
    let state_file = state_file_path(args.key.value().as_str());
    match File::create(state_file) {
        Ok(mut file) => match file.write_all(args.value.value().as_bytes()) {
            Ok(_) => quote!().into(),
            Err(e) => quote_io_error(e),
        },
        Err(e) => quote_io_error(e),
    }
}

/// Like [`write_state!`], but instead appends the specified `value` (newline-delimited) to the
/// state file. Newlines contained in the `value` are automatically escaped so you can think of
/// this as appending to a [`Vec<String>`] for all intents and purposes. Calling [`append_state!`]
/// is also more efficient than re-writing an entire state file via [`write_state!`] since the
/// low level append IO option is not used by [`write_state!`].
///
/// If no state file for the specified `key` exists, it will be created automatically. In this
/// way, [`append_state!`] functions similar to how [`init_state!`] functions, especially in the
/// no-existing-file case.
///
/// Note that if [`read_state!`] is called on an [`append_state!`]-based state file, newlines
/// will be returned in the response.
///
/// # Example
/// ```
/// append_state!("my_key", "apples");
/// append_state!("my_key", "pears");
/// append_state!("my_key", "oh my!");
/// assert_eq!(read_state!("my_key"), "apples\npears\noh my!\n");
/// ```
#[proc_macro]
pub fn append_state(items: TokenStream) -> TokenStream {
    let args = parse_macro_input!(items as WriteStateInput);
    let state_file = state_file_path(args.key.value().as_str());
    let value = args.value.value().replace("\n", "\\n");
    let value = format!("{}\n", value);
    match OpenOptions::new()
        .append(true)
        .create(true)
        .open(state_file)
    {
        Ok(mut file) => match file.write_all(value.as_bytes()) {
            Ok(_) => quote!().into(),
            Err(e) => quote_io_error(e),
        },
        Err(e) => quote_io_error(e),
    }
}

/// Reads the state value for the specified `key`. Since `macro_state` functions as a
/// compile-time key-value store, [`read_state!`] attempts to read the state value for the
/// specified `key`.
///
/// The macro will expand into a string literal representing the state value in the event that
/// a value exists for the provided key. If no value can be found for the provided key (or in
/// the event of any sort of IO error), the macro will raise a compile-time IO error.
///
/// # Example
/// ```rust
/// read_state!("my key"); // => "something"
/// ```
#[proc_macro]
pub fn read_state(items: TokenStream) -> TokenStream {
    let key = parse_macro_input!(items as LitStr).value();
    let state_file = state_file_path(key.as_str());
    match fs::read_to_string(state_file) {
        Ok(value) => quote!(#value).into(),
        Err(err) => quote_io_error(err),
    }
}

/// Reads the state value for the specified key and parses it as a [`Vec<String>`] where each new
/// line is treated as a separate element in the [`Vec`]. Should be used in conjunction with
/// [`append_state!`] to read and write lists of values from macro state storage.
///
/// Returns: a [`Vec<String>`]. Throws a compiler error if the specified state key cannot be found.
///
/// Note: If you need to initialize a state vec to an empty list, you can use
/// `init_state("key", "\n")` which should result in an empty [`Vec<String>`] when the state
/// file is read by [`read_state_vec!`].
///
/// Note: the [`Vec<String>`] returned by this macro is in literal form, e.g.
/// ```
/// vec!["item 1", "item 2", "item 3"];
/// ```
///
/// Note: This macro is infallible -- if any issue occurs trying to read the specified key, it
/// is assumed that we should return an empty [`Vec`].
///
/// # Example
/// ```
/// append_state!("my_key", "first item");
/// append_state!("my_key", "2nd item");
/// assert_eq!(read_state_vec!("my_key"), vec!["first item", "2nd item"]);
/// ```
#[proc_macro]
pub fn read_state_vec(items: TokenStream) -> TokenStream {
    let key = parse_macro_input!(items as LitStr).value();
    let state_file = state_file_path(key.as_str());
    match fs::read_to_string(state_file) {
        Ok(mut value) => {
            if let Some(last) = value.as_str().chars().last() {
                if last == '\n' {
                    value = value[0..(value.len() - 1)].to_string();
                }
            }
            let items: Vec<String> = value
                .split("\n")
                .map(|item| item.replace("\\n", "\n"))
                .collect();
            quote!(vec![#(#items), *]).into()
        }
        Err(_) => quote!(Vec::<String>::new()).into(),
    }
}

/// Checks if an existing state value can be found for the specified `key`.
///
/// # Example
/// ```
/// has_state!("my key"); // => bool
/// ```
#[proc_macro]
pub fn has_state(items: TokenStream) -> TokenStream {
    let key = parse_macro_input!(items as LitStr).value();
    let state_file = state_file_path(key.as_str());
    match fs::read_to_string(state_file) {
        Ok(_) => quote!(true).into(),
        Err(_) => quote!(false).into(),
    }
}

/// Clears the value for the specified `key`, if it exists
///
/// # Example
/// ```
/// write_state!("my key", "test");
/// read_state!("my key"); // => "test"
/// clear_state!("my key");
/// has_state!("my key"); // => false
/// ```
#[proc_macro]
pub fn clear_state(items: TokenStream) -> TokenStream {
    let key = parse_macro_input!(items as LitStr).value();
    let state_file = state_file_path(key.as_str());
    match fs::remove_file(state_file) {
        Ok(_) => {}
        Err(_) => {}
    }
    quote!().into()
}

/// Returns the value for the specified key, if it exists. If it does not exist, the key is
/// created and set to the specified value, and then the value is returned.
///
/// # Example
/// ```
/// write_state!("my key", "A");
/// init_state!("my key", "B"); // => "A"
/// init_state!("other key", "B"); // => "B"
/// ```
#[proc_macro]
pub fn init_state(items: TokenStream) -> TokenStream {
    let args = parse_macro_input!(items as WriteStateInput);
    let key = args.key.value().to_string();
    let value = args.value.value().to_string();
    let state_file = state_file_path(key.as_str());
    match fs::read_to_string(state_file) {
        Ok(string) => quote!(#string).into(),
        Err(_) => match File::create(state_file_path(key.as_str())) {
            Ok(mut file) => match file.write_all(value.as_bytes()) {
                Ok(_) => quote!(#value).into(),
                Err(err) => quote_io_error(err),
            },
            Err(err) => quote_io_error(err),
        },
    }
}
