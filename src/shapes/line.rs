use std::collections::{HashMap, HashSet};

use crate::{
    datapool::{DataPool, PointId, ShapeId},
    math::*,
};

use super::shapes::{
    ConstructionType, HandleType, LayerType, Shape, ShapeParameters, ShapeType, ShapesOperations,
};

#[derive(Copy, Clone)]
pub struct Line;
impl Line {
    pub fn new(
        data_pool: &mut DataPool,
        start_point: &WPoint,
        end_point: &WPoint,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *start_point;
        let start_point = *start_point - coord;
        let end_point = *end_point - coord;
        let end_point = if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
            start_point + snap_distance
        } else {
            end_point
        };
        let start_id = data_pool.insert_point(&start_point);
        let end_id = data_pool.insert_point(&end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Line,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Start, start_id);
                pts_ids.insert(HandleType::End, end_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, end_id)))
    }
}
impl ShapesOperations for Line {
    fn get_shape_construction(
        pool: &DataPool,
        shape: &Shape,
        selected: bool,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let start_id = *shape.handles_bundles.get(&HandleType::Start).unwrap();
        let end_id = *shape.handles_bundles.get(&HandleType::End).unwrap();
        if !selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        cst.push(ConstructionType::Move(
            *pool.get_point(start_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::Line(
            *pool.get_point(end_id).unwrap() + shape.coord,
        ));
        cst
    }
    fn get_helpers_construction(
        pool: &DataPool,
        shape: &Shape,
        _ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let start = *pool
            .get_point(*shape.handles_bundles.get(&HandleType::Start).unwrap())
            .unwrap()
            + shape.coord;
        let end = *pool
            .get_point(*shape.handles_bundles.get(&HandleType::End).unwrap())
            .unwrap()
            + shape.coord;
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        push_vertical(&start, &end, true, &mut cst);
        push_horizontal(&start, &end, true, &mut cst);
        push_45_135(&start, &end, true, &mut cst);
        cst
    }
}
