use crate::{
    shape::{Shape, ShapeTypes},
    types::*,
};

pub fn arrow_right(pos: WPos, size: f64, cst: &mut Vec<ConstructionType>) {
    let edges = vec![
        pos.addxy(size, 0.),
        pos.addxy(size + -5., -5.),
        pos.addxy(size + -5., 5.),
        pos.addxy(size, 0.),
    ];
    let mut shape = Shape::create(ShapeTypes::Segment(pos, edges[0]));
    edges.iter().for_each(|pos| _ = shape.add_line(*pos));

    shape.get_bss_constructions(cst);
}

pub fn arrow_down(pos: WPos, size: f64, cst: &mut Vec<ConstructionType>) {
    let edges = vec![
        pos.addxy(0., size),
        pos.addxy(-5., size + -5.),
        pos.addxy(5., size + -5.),
        pos.addxy(0., size),
    ];
    let mut shape = Shape::create(ShapeTypes::Segment(pos, edges[0]));
    edges.iter().for_each(|pos| _ = shape.add_line(*pos));

    shape.get_bss_constructions(cst);
}
