use napi::bindgen_prelude::*;

#[napi]
pub fn greet(name: String) -> String {
    format!("Hello, {}! From Rust.", name)
}
