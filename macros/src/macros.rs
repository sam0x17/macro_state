extern crate proc_macro;

#[macro_use]
extern crate lazy_static;

extern crate quote;
extern crate syn;

extern crate derive_syn_parse;
use derive_syn_parse::Parse;

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, Error};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
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

/// Writes the specified value as the state for the specified key
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

/// Reads the state value for the specified key
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

/// Checks if an existing state value can be found for the specified key
/// # Example
/// ```rust
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

/// Clears the value for the specified key, if it exists
/// # Example
/// ```rust
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

/// Returns the value for the specified key, if it exists. If
/// it does not exist, the key is created and set to the
/// specified value, and then the value is returned.
/// # Example
/// ```rust
/// write_state!("my key", "A");
/// init_state!("my key", "B"); // => "A"
/// init_state!("other key", "B"); // => "B"
/// ```
#[proc_macro]
pub fn init_state(items: TokenStream) -> TokenStream {
    let mut key = String::new();
    let mut value = String::new();
    let mut i = 0;
    for item in items {
        let token = item.to_string();
        match i {
            0 => {
                // first token
                match item {
                    proc_macro::TokenTree::Literal(literal) => {
                        key = literal.to_string();
                    }
                    _ => {
                        panic!("unexpected token {}", token);
                    }
                }
            }
            1 => {
                // second token
                match item {
                    proc_macro::TokenTree::Punct(punc) => {
                        if punc.as_char() != ',' {
                            panic!("unexpected token {}", token);
                        }
                    }
                    _ => {
                        panic!("unexpected token {}", token);
                    }
                }
            }
            2 => {
                // third token
                match item {
                    proc_macro::TokenTree::Literal(literal) => {
                        value = literal.to_string();
                    }
                    _ => {
                        panic!("unexpected token {}", token);
                    }
                }
            }
            _ => {
                panic!("unexpected token {}", token);
            }
        }
        i += 1;
    }
    let state_file = state_file_path(key.as_str());
    let output = match fs::read_to_string(state_file) {
        Ok(st) => st,
        Err(_err) => {
            let mut file = File::create(state_file_path(key.as_str()))
                .expect("error: cannot write state file!");
            file.write_all(value.as_bytes()).unwrap();
            value
        }
    };
    output.parse().unwrap()
}
