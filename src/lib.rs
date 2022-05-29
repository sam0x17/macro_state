extern crate proc_macro;

use proc_macro::TokenStream;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn state_file_path(key: &str) -> PathBuf {
    let filename = format!("macro_state_{}", key);
    let mut buf = PathBuf::new();
    buf.push(env!("MACRO_STATE_DIR"));
    buf.push(filename.as_str());
    buf
}

/// Writes the specified value as the state for the specified key
/// # Example
/// write_state!("my key", "some value");
#[proc_macro]
pub fn write_state(items: TokenStream) -> TokenStream {
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
    let mut file = File::create(state_file_path(key.as_str())).unwrap();
    file.write_all(value.as_bytes()).unwrap();
    "{}".parse().unwrap()
}

/// Reads the state value for the specified key
/// # Example
/// read_state!("my key");
#[proc_macro]
pub fn read_state(_items: TokenStream) -> TokenStream {
    "{}".parse().unwrap()
}
