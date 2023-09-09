mod canvas;
use canvas::{create_playing_area, PlayingArea};
use std::cell::RefCell;
use std::rc::Rc;
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

pub fn init_canvas(_pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    Ok(())
}
