fn main() {
    println!(
        "cargo:rustc-env=MACRO_STATE_DIR={}",
        std::env::var("OUT_DIR").unwrap()
    )
}
