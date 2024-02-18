// #![cfg(not(test))]
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

use kurbo::{BezPath, Circle, PathEl, Point, Shape};

use crate::types::*;

pub fn arrow_right(pos: Point, size: f64) -> BezPath {
    use PathEl::*;
    let v: Vec<PathEl> = vec![
        MoveTo(pos),
        LineTo(pos + (size, 0.)),
        LineTo(pos + (size + -5., -5.)),
        LineTo(pos + (size + -5., 5.)),
        LineTo(pos + (size, 0.)),
    ];
    BezPath::from_vec(v)
}

pub fn arrow_down(pos: Point, size: f64) -> BezPath {
    use PathEl::*;
    let v: Vec<PathEl> = vec![
        MoveTo(pos),
        LineTo(pos + (0., size)),
        LineTo(pos + (-5., size + -5.)),
        LineTo(pos + (5., size + -5.)),
        LineTo(pos + (0., size)),
    ];
    BezPath::from_vec(v)
}

pub fn handle(vx: &Vertex, size: f64, scale: f64) -> BezPath {
    let tol = 0.01;
    Circle::new(vx.pt, size / 2. / scale).to_path(tol)
}
