use std::collections::{HashMap, HashSet};

use crate::math::*;

use super::shapes::{
    ConstructionType, HandleType, LayerType, Shape, ShapeParameters, ShapeType, ShapesOperations,
};

#[derive(Copy, Clone)]
pub struct Rectangle;
impl Rectangle {
    pub fn new(
        data_pool: &mut DataPool,
        bl: &WPoint,
        w: f64,
        h: f64,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *bl;
        let bl = *bl - coord;

        let (w, h) = if w == 0. || h == 0. {
            (5. * snap_distance, 5. * snap_distance)
        } else {
            (w, h)
        };
        let bl_id = data_pool.insert_point(&bl);
        let tl_id = data_pool.insert_point(&WPoint::new(0., h));
        let tr_id = data_pool.insert_point(&WPoint::new(w, h));
        let br_id = data_pool.insert_point(&WPoint::new(w, 0.));
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Rectangle,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::BL, bl_id);
                pts_ids.insert(HandleType::TL, tl_id);
                pts_ids.insert(HandleType::TR, tr_id);
                pts_ids.insert(HandleType::BR, br_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, tr_id)))
    }
}
impl ShapesOperations for Rectangle {
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
        let bl_id = shape.handles_bundles.get(&HandleType::BL).unwrap();
        let tl_id = shape.handles_bundles.get(&HandleType::TL).unwrap();
        let tr_id = shape.handles_bundles.get(&HandleType::TR).unwrap();
        let br_id = shape.handles_bundles.get(&HandleType::BR).unwrap();
        cst.push(ConstructionType::Move(
            *pool.get_point(bl_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::Line(
            *pool.get_point(tl_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::Line(
            *pool.get_point(tr_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::Line(
            *pool.get_point(br_id).unwrap() + shape.coord,
        ));
        cst.push(ConstructionType::Line(
            *pool.get_point(bl_id).unwrap() + shape.coord,
        ));

        cst
    }
    fn get_helpers_construction(
        pool: &DataPool,
        shape: &Shape,
        _ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let bl = *pool
            .get_point(shape.handles_bundles.get(&HandleType::BL).unwrap())
            .unwrap()
            + shape.coord;
        let tl = *pool
            .get_point(shape.handles_bundles.get(&HandleType::TL).unwrap())
            .unwrap()
            + shape.coord;
        let tr = *pool
            .get_point(shape.handles_bundles.get(&HandleType::TR).unwrap())
            .unwrap()
            + shape.coord;
        let br = *pool
            .get_point(shape.handles_bundles.get(&HandleType::BR).unwrap())
            .unwrap()
            + shape.coord;
        // if let Some((selection, _pt_id)) = ohandle_selected {
        //     match selection {
        //         BL => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_45_135(&bl, &tr, true, &mut cst);
        //         }
        //         TL => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_45_135(&tl, &br, true, &mut cst);
        //         }
        //         TR => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_45_135(&tr, &bl, true, &mut cst);
        //         }
        //         BR => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_45_135(&br, &tl, true, &mut cst);
        //         }
        //         _ => (),
        //     }
        // }
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        push_45_135(&bl, &tr, true, &mut cst);
        push_45_135(&tl, &br, true, &mut cst);
        cst
    }
}
