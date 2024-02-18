pub mod bindings;
mod canvas;
mod math;
mod pools;
pub mod prefab;
pub mod shape;
pub mod types;

use canvas::create_playing_area;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    create_playing_area(window)?;
    Ok(())
}
