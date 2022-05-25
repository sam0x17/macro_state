extern crate proc_macro;

use proc_macro::TokenStream;

// write_state!(key, value)
#[proc_macro]
pub fn write_state(_items: TokenStream) -> TokenStream {
    println!("{}", env!("MACRO_STATE_DIR"));
    "println!(\"hello world\");".parse().unwrap()
}
