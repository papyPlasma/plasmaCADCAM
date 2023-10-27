use std::collections::HashMap;

use super::shapes::{ConstructionType, LayerType, PtIdProp, Shape};
use crate::{
    datapool::{DataPools, PointId, PointProperty, PointType, ShapeId, ShapePool, WPoint},
    math::*,
};

#[derive(Clone)]
pub struct QuadBezier {
    pts_ids: PtIdProp,
    init: bool,
}
impl QuadBezier {
    pub fn new(
        data_pools: &mut DataPools,
        start_point: &WPoint,
        ctrl_point: &WPoint,
        end_point: &WPoint,
        snap_distance: f64,
    ) -> (ShapeId, (PointId, PointProperty)) {
        let position = *start_point;
        let start = *start_point - position;
        let ctrl = *ctrl_point - position;
        let end = *end_point - position;

        let (end_point, ctrl_point) = if start.wx == end.wx || start.wy == end.wy {
            (start + 2. * snap_distance, start + snap_distance)
        } else {
            (end, ctrl)
        };
        let pos_id = data_pools.points_pool.insert(&position);
        let s_id = data_pools.points_pool.insert(&start);
        let c_id = data_pools.points_pool.insert(&ctrl_point);
        let e_id = data_pools.points_pool.insert(&end_point);

        let mut pts_ids = HashMap::new();
        pts_ids.insert(
            PointType::Position,
            (pos_id, PointProperty::new(false, false)),
        );
        pts_ids.insert(PointType::Start, (s_id, PointProperty::new(true, true)));
        pts_ids.insert(PointType::Ctrl, (c_id, PointProperty::new(false, true)));
        let pt_end_id_prop = (e_id, PointProperty::new(true, true));
        pts_ids.insert(PointType::End, pt_end_id_prop);

        let quadbezier = QuadBezier {
            pts_ids,
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(quadbezier);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        data_pools.pts_to_shs_pool.insert(s_id, sh_id);
        data_pools.pts_to_shs_pool.insert(c_id, sh_id);
        data_pools.pts_to_shs_pool.insert(e_id, sh_id);
        (sh_id, pt_end_id_prop)
    }
}
impl Shape for QuadBezier {
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
        let start = pts_pos.get(&PointType::Start).unwrap().1;
        let ctrl = pts_pos.get(&PointType::Ctrl).unwrap().1;
        let end = pts_pos.get(&PointType::End).unwrap().1;

        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;
        let pt = *pt - position;

        for _i in 0..MAX_ITERATIONS {
            // max iterations can be adjusted
            let t_mid = (t_min + t_max) / 2.;
            let bt = get_point_on_quad_bezier(t_mid, &start, &ctrl, &end);
            let dist = bt.dist(&pt);
            if dist < min_dist {
                min_dist = dist;
            }
            if dist < precision {
                return true; // We found a sufficiently close point
            }
            // Using gradient to decide the next tMid for the next iteration.
            let gradient =
                (bt.wx - pt.wx) * (end.wx - start.wx) + (bt.wy - pt.wy) * (end.wy - start.wy);
            if gradient > 0. {
                t_max = t_mid;
            } else {
                t_min = t_mid;
            }
        }
        min_dist <= precision
    }
    fn update_points_pos(
        &self,
        pts_pos: &mut HashMap<PointType, (PointId, WPoint)>,
        pt_id: &PointId,
        pick_pt: &WPoint,
        snap_distance: f64,
    ) {
        let (_, position) = pts_pos.get(&PointType::Position).cloned().unwrap();
        let rel_pick_point = *pick_pt - position;
        let (start_id, mut start_pos) = pts_pos.get(&PointType::Start).cloned().unwrap();
        let (ctrl_id, mut ctrl_pos) = pts_pos.get(&PointType::Ctrl).cloned().unwrap();
        let (end_id, mut end_pos) = pts_pos.get(&PointType::End).cloned().unwrap();

        if self.init {
            if *pt_id == end_id {
                end_pos = rel_pick_point;
                if end_pos == start_pos {
                    end_pos += 2. * snap_distance;
                }
                ctrl_pos = end_pos / 2.;
                pts_pos.insert(PointType::Ctrl, (ctrl_id, ctrl_pos));
                pts_pos.insert(PointType::End, (end_id, end_pos));
            }
        } else {
            if *pt_id == start_id {
                start_pos = rel_pick_point;
                if start_pos == end_pos {
                    start_pos += 2. * snap_distance;
                }
                pts_pos.insert(PointType::Start, (start_id, start_pos));
            }
            if *pt_id == ctrl_id {
                ctrl_pos = rel_pick_point;
                if ctrl_pos == end_pos || ctrl_pos == start_pos {
                    ctrl_pos += snap_distance;
                }
                pts_pos.insert(PointType::Ctrl, (ctrl_id, ctrl_pos));
            }
            if *pt_id == end_id {
                end_pos = rel_pick_point;
                if end_pos == start_pos {
                    end_pos += 2. * snap_distance;
                }
                pts_pos.insert(PointType::End, (end_id, end_pos));
            }
        }
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
        let (_, start_pos) = pts_pos.get(&PointType::Start).unwrap();
        let (_, ctrl_pos) = pts_pos.get(&PointType::Ctrl).unwrap();
        let (_, end_pos) = pts_pos.get(&PointType::End).unwrap();
        cst.push(ConstructionType::Move(position + start_pos));
        cst.push(ConstructionType::QuadBezier(
            position + ctrl_pos,
            position + end_pos,
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
        let (start_id, start_pos) = pts_pos.get(&PointType::Start).unwrap();
        let (ctrl_id, ctrl_pos) = pts_pos.get(&PointType::Ctrl).unwrap();
        let (end_id, end_pos) = pts_pos.get(&PointType::End).unwrap();

        hdles.push((*start_id, position + start_pos));
        hdles.push((*ctrl_id, position + ctrl_pos));
        hdles.push((*end_id, position + end_pos));
        push_handles(&mut cst, &hdles, opt_sel_id_prop, size_handle);
        cst
    }
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];

        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (_, start_pos) = pts_pos.get(&PointType::Start).unwrap();
        let (_, ctrl_pos) = pts_pos.get(&PointType::Ctrl).unwrap();
        let (_, end_pos) = pts_pos.get(&PointType::End).unwrap();

        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));

        if is_aligned_vert(&start_pos, &end_pos) {
            helper_vertical(
                &(position + start_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&start_pos, &end_pos) {
            helper_horizontal(
                &(position + start_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&start_pos, &end_pos) {
            helper_45_135(
                &(position + start_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }

        if is_aligned_vert(&ctrl_pos, &end_pos) {
            helper_vertical(
                &(position + ctrl_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl_pos, &end_pos) {
            helper_horizontal(
                &(position + ctrl_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl_pos, &end_pos) {
            helper_45_135(
                &(position + ctrl_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }

        if is_aligned_vert(&ctrl_pos, &start_pos) {
            helper_vertical(
                &(position + ctrl_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl_pos, &start_pos) {
            helper_horizontal(
                &(position + ctrl_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl_pos, &start_pos) {
            helper_45_135(
                &(position + ctrl_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        cst
    }
}
impl ShapePool for QuadBezier {}
