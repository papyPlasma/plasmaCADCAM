use std::collections::{HashMap, HashSet};

use crate::math::*;

use super::shapes::{
    ConstructionType, HandleType, LayerType, Shape, ShapeParameters, ShapeType, ShapesOperations,
};

#[derive(Copy, Clone)]
pub struct Ellipse;
impl Ellipse {
    pub fn new(
        data_pool: &mut DataPool,
        center: &WPoint,
        radius: &WPoint,
        start_angle: f64,
        end_angle: f64,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *center;
        let center = *center - coord;
        let radius = *radius - coord;
        let radius = if radius.wx == 0. || radius.wy == 0. {
            WPoint::new(5. * snap_distance, -5. * snap_distance)
        } else {
            radius
        };
        let center_id = data_pool.insert_point(&center);
        let radius_id = data_pool.insert_point(&(radius + center));
        let h_start_angle = get_point_from_angle(&(radius + center), -start_angle);
        let h_end_angle = get_point_from_angle(&(radius + center), -end_angle);
        let h_start_angle_id = data_pool.insert_point(&h_start_angle);
        let h_end_angle_id = data_pool.insert_point(&h_end_angle);

        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Ellipse,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Center, center_id);
                pts_ids.insert(HandleType::Radius, radius_id);
                pts_ids.insert(HandleType::StartAngle, h_start_angle_id);
                pts_ids.insert(HandleType::EndAngle, h_end_angle_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, radius_id)))
    }
}
impl ShapesOperations for Ellipse {
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
        let center_id = shape.handles_bundles.get(&HandleType::Center).unwrap();
        let radius_id = shape.handles_bundles.get(&HandleType::Radius).unwrap();
        let h_start_angle_id = shape.handles_bundles.get(&HandleType::StartAngle).unwrap();
        let h_end_angle_id = shape.handles_bundles.get(&HandleType::EndAngle).unwrap();

        let center = *pool.get_point(center_id).unwrap();
        let radius = *pool.get_point(radius_id).unwrap();
        let h_start_angle = *pool.get_point(h_start_angle_id).unwrap();
        let h_end_angle = *pool.get_point(h_end_angle_id).unwrap();

        cst.push(ConstructionType::Move(shape.coord + center + h_start_angle));

        let start_angle = -center.angle_on_ellipse(&h_start_angle, &radius);
        let end_angle = -center.angle_on_ellipse(&h_end_angle, &radius);
        cst.push(ConstructionType::Ellipse(
            shape.coord + center,
            radius.abs(),
            0.,
            start_angle,
            end_angle,
            false,
        ));

        cst
    }
    fn get_helpers_construction(
        pool: &DataPool,
        shape: &Shape,
        _ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let center = *pool
            .get_point(shape.handles_bundles.get(&HandleType::Center).unwrap())
            .unwrap()
            + shape.coord;
        let radius = *pool
            .get_point(shape.handles_bundles.get(&HandleType::Radius).unwrap())
            .unwrap()
            + shape.coord;
        let h_sa = *pool
            .get_point(shape.handles_bundles.get(&HandleType::StartAngle).unwrap())
            .unwrap()
            + shape.coord;
        let h_ea = *pool
            .get_point(shape.handles_bundles.get(&HandleType::EndAngle).unwrap())
            .unwrap()
            + shape.coord;
        // if let Some((selection, _pt_id)) = ohandle_selected {
        //     match selection {
        //         Center => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&center, &h_sa, true, &mut cst);
        //             push_horizontal(&center, &h_sa, true, &mut cst);
        //             push_45_135(&center, &h_sa, true, &mut cst);
        //             push_vertical(&center, &h_ea, true, &mut cst);
        //             push_horizontal(&center, &h_ea, true, &mut cst);
        //             push_45_135(&center, &h_ea, true, &mut cst);
        //             push_vertical(&center, &radius, true, &mut cst);
        //             push_horizontal(&center, &radius, true, &mut cst);
        //             push_45_135(&center, &radius, true, &mut cst);
        //         }
        //         Radius => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&center, &radius, true, &mut cst);
        //             push_horizontal(&center, &radius, true, &mut cst);
        //             push_45_135(&center, &radius, true, &mut cst);
        //         }
        //         StartAngle => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&h_sa, &center, true, &mut cst);
        //             push_horizontal(&h_sa, &center, true, &mut cst);
        //             push_45_135(&h_sa, &center, true, &mut cst);
        //             push_vertical(&h_sa, &h_ea, true, &mut cst);
        //             push_horizontal(&h_sa, &h_ea, true, &mut cst);
        //             push_45_135(&h_sa, &h_ea, true, &mut cst);
        //         }
        //         EndAngle => {
        //             cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        //             push_vertical(&h_ea, &center, true, &mut cst);
        //             push_horizontal(&h_ea, &center, true, &mut cst);
        //             push_45_135(&h_ea, &center, true, &mut cst);
        //             push_vertical(&h_ea, &h_sa, true, &mut cst);
        //             push_horizontal(&h_ea, &h_sa, true, &mut cst);
        //             push_45_135(&h_ea, &h_sa, true, &mut cst);
        //         }
        //         _ => (),
        //     }
        // }
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        push_vertical(&center, &h_sa, true, &mut cst);
        push_horizontal(&center, &h_sa, true, &mut cst);
        push_45_135(&center, &h_sa, true, &mut cst);
        push_vertical(&center, &h_ea, true, &mut cst);
        push_horizontal(&center, &h_ea, true, &mut cst);
        push_45_135(&center, &h_ea, true, &mut cst);
        push_vertical(&center, &radius, true, &mut cst);
        push_horizontal(&center, &radius, true, &mut cst);
        push_45_135(&center, &radius, true, &mut cst);
        push_vertical(&h_sa, &h_ea, true, &mut cst);
        push_horizontal(&h_sa, &h_ea, true, &mut cst);
        push_45_135(&h_sa, &h_ea, true, &mut cst);
        cst
    }
}
