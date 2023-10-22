use std::collections::{HashMap, HashSet};

use crate::{
    datapool::{DataPool, PointId, ShapeId},
    math::*,
};

use super::shapes::{
    ConstructionType, HandleType, LayerType, Shape, ShapeParameters, ShapeType, ShapesOperations,
};

#[derive(Copy, Clone)]

pub struct CubicBezier;
impl CubicBezier {
    pub fn new(
        data_pool: &mut DataPool,
        start_point: &WPoint,
        ctrl1_point: &WPoint,
        ctrl2_point: &WPoint,
        end_point: &WPoint,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *start_point;
        let start_point = *start_point - coord;
        let ctrl1_point = *ctrl1_point - coord;
        let ctrl2_point = *ctrl2_point - coord;
        let end_point = *end_point - coord;

        let (end_point, ctrl1_point, ctrl2_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    start_point + 3. * snap_distance,
                    start_point + snap_distance,
                    start_point + 2. * snap_distance,
                )
            } else {
                (end_point, ctrl1_point, ctrl2_point)
            };
        let start_id = data_pool.insert_point(&start_point);
        let ctrl1_id = data_pool.insert_point(&ctrl1_point);
        let ctrl2_id = data_pool.insert_point(&ctrl2_point);
        let end_id = data_pool.insert_point(&end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::CubicBezier,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Start, start_id);
                pts_ids.insert(HandleType::Ctrl1, ctrl1_id);
                pts_ids.insert(HandleType::Ctrl2, ctrl2_id);
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
impl ShapesOperations for CubicBezier {
    fn get_shape_construction(
        pool: &DataPool,
        shape: &Shape,
        selected: bool,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        let start_id = *shape.handles_bundles.get(&HandleType::Start).unwrap();
        let ctrl1_id = *shape.handles_bundles.get(&HandleType::Ctrl1).unwrap();
        let ctrl2_id = *shape.handles_bundles.get(&HandleType::Ctrl2).unwrap();
        let end_id = *shape.handles_bundles.get(&HandleType::End).unwrap();
        cst.push(ConstructionType::Move(
            *pool.get_point(start_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::CubicBezier(
            *pool.get_point(ctrl1_id).unwrap() + shape.coord,
            *pool.get_point(ctrl2_id).unwrap() + shape.coord,
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
        let ctrl1 = *pool
            .get_point(*shape.handles_bundles.get(&HandleType::Ctrl1).unwrap())
            .unwrap()
            + shape.coord;
        let ctrl2 = *pool
            .get_point(*shape.handles_bundles.get(&HandleType::Ctrl2).unwrap())
            .unwrap()
            + shape.coord;
        let end = *pool
            .get_point(*shape.handles_bundles.get(&HandleType::End).unwrap())
            .unwrap()
            + shape.coord;
        // if let Some((selection, _pt_id)) = ohandle_selected {
        //     match selection {
        //         Start | End => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&start, &end, true, &mut cst);
        //             push_horizontal(&start, &end, true, &mut cst);
        //             push_45_135(&start, &end, true, &mut cst);
        //         }
        //         Ctrl1 => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&ctrl1, &start, true, &mut cst);
        //             push_horizontal(&ctrl1, &start, true, &mut cst);
        //             push_45_135(&ctrl1, &start, true, &mut cst);
        //             push_vertical(&ctrl1, &ctrl2, true, &mut cst);
        //             push_horizontal(&ctrl1, &ctrl2, true, &mut cst);
        //             push_45_135(&ctrl1, &ctrl2, true, &mut cst);
        //             push_vertical(&ctrl1, &end, true, &mut cst);
        //             push_horizontal(&ctrl1, &end, true, &mut cst);
        //             push_45_135(&ctrl1, &end, true, &mut cst);
        //         }
        //         Ctrl2 => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&ctrl2, &start, true, &mut cst);
        //             push_horizontal(&ctrl2, &start, true, &mut cst);
        //             push_45_135(&ctrl2, &start, true, &mut cst);
        //             push_vertical(&ctrl2, &ctrl1, true, &mut cst);
        //             push_horizontal(&ctrl2, &ctrl1, true, &mut cst);
        //             push_45_135(&ctrl2, &ctrl1, true, &mut cst);
        //             push_vertical(&ctrl2, &end, true, &mut cst);
        //             push_horizontal(&ctrl2, &end, true, &mut cst);
        //             push_45_135(&ctrl2, &end, true, &mut cst);
        //         }
        //         _ => (),
        //     }
        // }
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        push_vertical(&start, &end, true, &mut cst);
        push_horizontal(&start, &end, true, &mut cst);
        push_45_135(&start, &end, true, &mut cst);
        push_vertical(&ctrl1, &start, true, &mut cst);
        push_horizontal(&ctrl1, &start, true, &mut cst);
        push_45_135(&ctrl1, &start, true, &mut cst);
        push_vertical(&ctrl1, &ctrl2, true, &mut cst);
        push_horizontal(&ctrl1, &ctrl2, true, &mut cst);
        push_45_135(&ctrl1, &ctrl2, true, &mut cst);
        push_vertical(&ctrl1, &end, true, &mut cst);
        push_horizontal(&ctrl1, &end, true, &mut cst);
        push_45_135(&ctrl1, &end, true, &mut cst);
        push_vertical(&ctrl2, &start, true, &mut cst);
        push_horizontal(&ctrl2, &start, true, &mut cst);
        push_45_135(&ctrl2, &start, true, &mut cst);
        push_vertical(&ctrl2, &end, true, &mut cst);
        push_horizontal(&ctrl2, &end, true, &mut cst);
        push_45_135(&ctrl2, &end, true, &mut cst);
        cst
    }
}
