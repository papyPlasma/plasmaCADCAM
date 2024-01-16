use crate::{
    shape::{Shape, ShapeTypes},
    types::*,
};

pub fn arrow_right(pos: WPos, size: f64, cst: &mut Vec<ConstructionType>) {
    let vertices = vec![
        pos,
        pos.addxy(size, 0.),
        pos.addxy(size + -5., -5.),
        pos.addxy(size + -5., 5.),
        pos.addxy(size, 0.),
    ];

    polyline(&vertices, cst);
}

pub fn arrow_down(pos: WPos, size: f64, cst: &mut Vec<ConstructionType>) {
    let vertices = vec![
        pos,
        pos.addxy(0., size),
        pos.addxy(-5., size + -5.),
        pos.addxy(5., size + -5.),
        pos.addxy(0., size),
    ];
    polyline(&vertices, cst);
}

pub fn polyline(vertices: &Vec<WPos>, cst: &mut Vec<ConstructionType>) {
    let mut last_vertex = vertices[0];
    for vertex in vertices.iter().skip(1) {
        let shape = Shape::create(ShapeTypes::Segment(last_vertex, *vertex));
        shape.get_bss_constructions(cst);
        last_vertex = *vertex;
    }
}
