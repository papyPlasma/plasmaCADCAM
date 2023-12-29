mod canvas;
mod math;
pub mod shape;
mod shapes_pool;
pub mod types;
pub mod basic_shapes {
    // pub mod arc_ellipse;
    // pub mod cubic_bezier;
    // pub mod quad_bezier;
    pub mod segment;
}

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
