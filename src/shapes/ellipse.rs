use std::{collections::HashMap, f64::consts::PI};

use super::shapes::{ConstructionType, LayerType, PtIdProp, Shape};
use crate::{
    datapool::{DataPools, PointId, PointProperty, PointType, ShapeId, ShapePool, WPoint},
    math::*,
};

#[derive(Clone)]
pub struct Ellipse {
    pts_ids: PtIdProp,
    init: bool,
}
impl Ellipse {
    pub fn new(
        data_pools: &mut DataPools,
        center_point: &WPoint,
        radius_point: &WPoint,
        start_angle: f64,
        end_angle: f64,
        snap_distance: f64,
    ) -> (ShapeId, (PointId, PointProperty)) {
        let position = *center_point;
        let center = *center_point - position;
        let radius = *radius_point - position;

        let radius = if radius.wx == 0. || radius.wy == 0. {
            WPoint::new(5. * snap_distance, -5. * snap_distance)
        } else {
            radius
        };
        let s_angle = get_point_from_angle(&radius, -start_angle);
        let e_angle = get_point_from_angle(&radius, -end_angle);

        let pos_id = data_pools.points_pool.insert(&position);
        let center_id = data_pools.points_pool.insert(&center);
        let radius_id = data_pools.points_pool.insert(&radius);
        let sa_id = data_pools.points_pool.insert(&s_angle);
        let ea_id = data_pools.points_pool.insert(&e_angle);

        let mut pts_ids = HashMap::new();
        pts_ids.insert(
            PointType::Position,
            (pos_id, PointProperty::new(false, false)),
        );
        pts_ids.insert(
            PointType::Center,
            (center_id, PointProperty::new(false, true)),
        );
        let pt_radius_id_prop = (radius_id, PointProperty::new(false, true));
        pts_ids.insert(PointType::Radius, pt_radius_id_prop);
        pts_ids.insert(
            PointType::StartAngle,
            (sa_id, PointProperty::new(true, true)),
        );
        pts_ids.insert(PointType::EndAngle, (ea_id, PointProperty::new(true, true)));

        let ellipse = Ellipse {
            pts_ids,
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(ellipse);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        data_pools.pts_to_shs_pool.insert(center_id, sh_id);
        data_pools.pts_to_shs_pool.insert(radius_id, sh_id);
        data_pools.pts_to_shs_pool.insert(sa_id, sh_id);
        data_pools.pts_to_shs_pool.insert(ea_id, sh_id);
        (sh_id, pt_radius_id_prop)
    }
    fn ellipse_line_intersection(&self, center: &WPoint, radius: &WPoint, pt: &WPoint) -> WPoint {
        let m = (pt.wy - center.wy) / (pt.wx - center.wx);
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;
        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;
        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;
        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;
        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);
        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);
        if pt.dist(&pt1) < pt.dist(&pt2) {
            pt1
        } else {
            pt2
        }
    }
    fn ellipse_angle_intersection(&self, center: &WPoint, radius: &WPoint, angle: f64) -> WPoint {
        if angle == -PI / 2. {
            return WPoint {
                wx: 0.,
                wy: radius.wy,
            };
        }
        if angle == PI / 2. {
            return WPoint {
                wx: 0.,
                wy: -radius.wy,
            };
        }
        let m = angle.tan();
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;
        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;
        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;
        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;
        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);
        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);
        if angle >= 0. {
            if m >= 0. {
                pt1
            } else {
                pt2
            }
        } else {
            if m >= 0. {
                pt2
            } else {
                pt1
            }
        }
    }
    pub fn angle_on_ellipse(&self, center: &WPoint, point: &WPoint, radius: &WPoint) -> f64 {
        f64::atan2(
            (point.wy - center.wy) / radius.wy,
            (point.wx - center.wx) / radius.wx,
        )
    }
}
impl Shape for Ellipse {
    fn is_init(&self) -> bool {
        self.init
    }
    fn get_pos_id(&self) -> (PointId, PointProperty) {
        *self.pts_ids.get(&PointType::Position).unwrap()
    }
    fn init_done(&mut self) {
        self.init = false;
    }
    fn get_points_ids(&self) -> PtIdProp {
        self.pts_ids.clone()
    }
    fn is_point_on_shape(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        pt: &WPoint,
        precision: f64,
    ) -> bool {
        let position = pts_pos.get(&PointType::Position).unwrap().1;
        let center = pts_pos.get(&PointType::Center).unwrap().1;
        let radius = pts_pos.get(&PointType::Radius).unwrap().1;
        let sa = pts_pos.get(&PointType::StartAngle).unwrap().1;
        let ea = pts_pos.get(&PointType::EndAngle).unwrap().1;
        let pt = *pt - position;
        let pt_int = self.ellipse_line_intersection(&center, &radius, &pt);
        if pt_int.dist(&pt) > precision {
            return false;
        }
        if sa.dist(&ea) < 1. {
            return true;
        }
        let start_angle = -self.angle_on_ellipse(&center, &sa, &radius);
        let end_angle = -self.angle_on_ellipse(&center, &ea, &radius);
        let angle = -self.angle_on_ellipse(&center, &pt, &radius);
        if end_angle > start_angle {
            angle >= start_angle && angle <= end_angle
        } else {
            !(angle >= end_angle && angle <= start_angle)
        }
    }
    fn update_points_pos(
        &self,
        pts_pos: &mut HashMap<PointType, (PointId, WPoint)>,
        pt_id: &PointId,
        pick_pt: &WPoint,
        snap_distance: f64,
    ) {
        let (position_id, mut position) = pts_pos.get(&PointType::Position).cloned().unwrap();
        let rel_pick_point = *pick_pt - position;

        let (center_id, center) = pts_pos.get(&PointType::Center).cloned().unwrap();
        let (radius_id, mut radius) = pts_pos.get(&PointType::Radius).cloned().unwrap();
        let (sa_id, mut s_angle) = pts_pos.get(&PointType::StartAngle).cloned().unwrap();
        let (ea_id, mut e_angle) = pts_pos.get(&PointType::EndAngle).cloned().unwrap();

        let start_angle = get_atan2(&(s_angle - center));
        let end_angle = get_atan2(&(e_angle - center));

        if *pt_id == center_id {
            position = position + rel_pick_point;
        }

        if *pt_id == radius_id {
            radius = rel_pick_point;
            if radius.wx <= center.wx {
                radius.wx = center.wx + snap_distance;
            }
            if radius.wy >= center.wy {
                radius.wy = center.wy - snap_distance;
            }
            s_angle = self.ellipse_angle_intersection(&center, &radius, start_angle);
            e_angle = self.ellipse_angle_intersection(&center, &radius, end_angle);
        }
        if *pt_id == sa_id {
            let angle = get_atan2(&(rel_pick_point - center));
            s_angle = get_point_from_angle(&radius, angle);
        }
        if *pt_id == ea_id {
            let angle = get_atan2(&(rel_pick_point - center));
            e_angle = get_point_from_angle(&radius, angle);
        }

        pts_pos.insert(PointType::Center, (position_id, position));
        pts_pos.insert(PointType::Radius, (radius_id, radius));
        pts_pos.insert(PointType::StartAngle, (sa_id, s_angle));
        pts_pos.insert(PointType::EndAngle, (ea_id, e_angle));
    }
    fn get_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        selected: bool,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (_, center) = pts_pos.get(&PointType::Center).unwrap();
        let (_, radius) = pts_pos.get(&PointType::Radius).unwrap();
        let (_, s_angle) = pts_pos.get(&PointType::StartAngle).unwrap();
        let (_, e_angle) = pts_pos.get(&PointType::EndAngle).unwrap();

        let start_angle = -self.angle_on_ellipse(center, s_angle, radius);
        let end_angle = -self.angle_on_ellipse(center, e_angle, radius);

        cst.push(ConstructionType::Move(position + s_angle));
        cst.push(ConstructionType::Ellipse(
            position + center,
            WPoint::new(radius.wx, -radius.wy),
            0.,
            start_angle,
            end_angle,
            false,
        ));
        cst
    }
    fn get_handles_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        opt_sel_id_prop: &Option<(PointId, PointProperty)>,
        size_handle: f64,
    ) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        let mut hdles = Vec::new();

        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (center_id, center) = pts_pos.get(&PointType::Center).unwrap();
        let (radius_id, radius) = pts_pos.get(&PointType::Radius).unwrap();
        let (sa_id, s_angle) = pts_pos.get(&PointType::StartAngle).unwrap();
        let (ea_id, e_angle) = pts_pos.get(&PointType::EndAngle).unwrap();

        hdles.push((*center_id, position + center));
        hdles.push((*radius_id, position + radius));
        hdles.push((*sa_id, position + s_angle));
        hdles.push((*ea_id, position + e_angle));
        push_handles(&mut cst, &hdles, opt_sel_id_prop, size_handle);
        cst
    }
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (_, center) = pts_pos.get(&PointType::Center).unwrap();
        let (_, radius) = pts_pos.get(&PointType::Radius).unwrap();
        let (_, s_angle) = pts_pos.get(&PointType::StartAngle).unwrap();
        let (_, e_angle) = pts_pos.get(&PointType::EndAngle).unwrap();

        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        if is_aligned_45_or_135(&center, &radius) {
            helper_45_135(&(position + center), &(position + radius), true, &mut cst);
        }
        if is_aligned_45_or_135(&center, &s_angle) {
            helper_45_135(&(position + center), &(position + s_angle), true, &mut cst);
        }
        if is_aligned_vert(&center, &s_angle) {
            helper_vertical(&(position + center), &(position + s_angle), true, &mut cst);
        }
        if is_aligned_hori(&center, &s_angle) {
            helper_horizontal(&(position + center), &(position + s_angle), true, &mut cst);
        }
        if is_aligned_45_or_135(&center, &e_angle) {
            helper_45_135(&(position + center), &(position + e_angle), true, &mut cst);
        }
        if is_aligned_vert(&center, &e_angle) {
            helper_vertical(&(position + center), &(position + e_angle), true, &mut cst);
        }
        if is_aligned_hori(&center, &e_angle) {
            helper_horizontal(&(position + center), &(position + e_angle), true, &mut cst);
        }
        cst
    }
}
impl ShapePool for Ellipse {}
