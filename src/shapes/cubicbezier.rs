use std::collections::HashMap;

use super::shapes::{ConstructionType, LayerType, Shape, WPoint};
use crate::{
    datapool::{DataPools, PointId, PointType, ShapeId, ShapePool},
    math::*,
};

#[derive(Clone)]
pub struct CubicBezier {
    pts_ids: HashMap<PointType, PointId>,
    init: bool,
}
impl CubicBezier {
    pub fn new(
        data_pools: &mut DataPools,
        start_point: &WPoint,
        ctrl1_point: &WPoint,
        ctrl2_point: &WPoint,
        end_point: &WPoint,
        snap_distance: f64,
    ) -> (ShapeId, PointId) {
        let position = *start_point;
        let start = *start_point - position;
        let ctrl1 = *ctrl1_point - position;
        let ctrl2 = *ctrl2_point - position;
        let end = *end_point - position;

        let (end, ctrl1, ctrl2) = if start.wx == end.wx || start.wy == end.wy {
            (
                start + 3. * snap_distance,
                start + snap_distance,
                start + 2. * snap_distance,
            )
        } else {
            (end, ctrl1, ctrl2)
        };
        let pos_id = data_pools.points_pool.insert(&position);
        let s_id = data_pools.points_pool.insert(&start);
        let c1_id = data_pools.points_pool.insert(&ctrl1);
        let c2_id = data_pools.points_pool.insert(&ctrl2);
        let e_id = data_pools.points_pool.insert(&end);

        let mut pts_ids = HashMap::new();
        pts_ids.insert(PointType::Position, pos_id);
        pts_ids.insert(PointType::Start, s_id);
        pts_ids.insert(PointType::Ctrl1, c1_id);
        pts_ids.insert(PointType::Ctrl2, c2_id);
        pts_ids.insert(PointType::End, e_id);

        let quadbezier = CubicBezier {
            pts_ids,
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(quadbezier);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        data_pools.pts_to_shs_pool.insert(s_id, sh_id);
        data_pools.pts_to_shs_pool.insert(c1_id, sh_id);
        data_pools.pts_to_shs_pool.insert(c2_id, sh_id);
        data_pools.pts_to_shs_pool.insert(e_id, sh_id);
        (sh_id, e_id)
    }
}
impl Shape for CubicBezier {
    fn is_init(&self) -> bool {
        self.init
    }
    fn get_pos_id(&self) -> PointId {
        *self.pts_ids.get(&PointType::Position).unwrap()
    }
    fn init_done(&mut self) {
        self.init = false;
    }
    fn get_points_ids(&self) -> HashMap<PointType, PointId> {
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
        let ctrl1 = pts_pos.get(&PointType::Ctrl1).unwrap().1;
        let ctrl2 = pts_pos.get(&PointType::Ctrl2).unwrap().1;
        let end = pts_pos.get(&PointType::End).unwrap().1;

        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;
        let pt = *pt - position;

        for _i in 0..MAX_ITERATIONS {
            let t_mid = (t_min + t_max) / 2.;
            let bt = get_point_on_cubic_bezier(t_mid, &start, &ctrl1, &ctrl2, &end);
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
        let (ctrl1_id, mut ctrl1_pos) = pts_pos.get(&PointType::Ctrl1).cloned().unwrap();
        let (ctrl2_id, mut ctrl2_pos) = pts_pos.get(&PointType::Ctrl2).cloned().unwrap();
        let (end_id, mut end_pos) = pts_pos.get(&PointType::End).cloned().unwrap();

        if self.init {
            if *pt_id == end_id {
                end_pos = rel_pick_point;
                if end_pos == start_pos {
                    end_pos += 3. * snap_distance;
                }
                ctrl1_pos = end_pos / 3.;
                ctrl2_pos = end_pos * 2. / 3.;
                pts_pos.insert(PointType::Ctrl1, (ctrl1_id, ctrl1_pos));
                pts_pos.insert(PointType::Ctrl2, (ctrl2_id, ctrl2_pos));
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
            if *pt_id == ctrl1_id {
                ctrl1_pos = rel_pick_point;
                if ctrl1_pos == end_pos || ctrl1_pos == start_pos {
                    ctrl1_pos += snap_distance;
                }
                pts_pos.insert(PointType::Ctrl1, (ctrl1_id, ctrl1_pos));
            }
            if *pt_id == ctrl2_id {
                ctrl2_pos = rel_pick_point;
                if ctrl2_pos == end_pos || ctrl2_pos == start_pos {
                    ctrl2_pos += snap_distance;
                }
                pts_pos.insert(PointType::Ctrl2, (ctrl2_id, ctrl2_pos));
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
        let (_, ctrl1_pos) = pts_pos.get(&PointType::Ctrl1).unwrap();
        let (_, ctrl2_pos) = pts_pos.get(&PointType::Ctrl2).unwrap();
        let (_, end_pos) = pts_pos.get(&PointType::End).unwrap();
        cst.push(ConstructionType::Move(position + start_pos));
        cst.push(ConstructionType::CubicBezier(
            position + ctrl1_pos,
            position + ctrl2_pos,
            position + end_pos,
        ));
        cst
    }
    fn get_handles_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        opt_sel_id: &Option<PointId>,
        size_handle: f64,
    ) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        let mut hdles = Vec::new();

        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (start_id, start_pos) = pts_pos.get(&PointType::Start).unwrap();
        let (ctrl1_id, ctrl1_pos) = pts_pos.get(&PointType::Ctrl1).unwrap();
        let (ctrl2_id, ctrl2_pos) = pts_pos.get(&PointType::Ctrl2).unwrap();
        let (end_id, end_pos) = pts_pos.get(&PointType::End).unwrap();

        hdles.push((*start_id, position + start_pos));
        hdles.push((*ctrl1_id, position + ctrl1_pos));
        hdles.push((*ctrl2_id, position + ctrl2_pos));
        hdles.push((*end_id, position + end_pos));
        push_handles(&mut cst, &hdles, opt_sel_id, size_handle);
        cst
    }
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];

        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (_, start_pos) = pts_pos.get(&PointType::Start).unwrap();
        let (_, ctrl1_pos) = pts_pos.get(&PointType::Ctrl1).unwrap();
        let (_, ctrl2_pos) = pts_pos.get(&PointType::Ctrl2).unwrap();
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

        if is_aligned_vert(&ctrl1_pos, &end_pos) {
            helper_vertical(
                &(position + ctrl1_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl1_pos, &end_pos) {
            helper_horizontal(
                &(position + ctrl1_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl1_pos, &end_pos) {
            helper_45_135(
                &(position + ctrl1_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_vert(&ctrl1_pos, &start_pos) {
            helper_vertical(
                &(position + ctrl1_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl1_pos, &start_pos) {
            helper_horizontal(
                &(position + ctrl1_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl1_pos, &start_pos) {
            helper_45_135(
                &(position + ctrl1_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }

        if is_aligned_vert(&ctrl2_pos, &end_pos) {
            helper_vertical(
                &(position + ctrl2_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl2_pos, &end_pos) {
            helper_horizontal(
                &(position + ctrl2_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl2_pos, &end_pos) {
            helper_45_135(
                &(position + ctrl2_pos),
                &(position + end_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_vert(&ctrl2_pos, &start_pos) {
            helper_vertical(
                &(position + ctrl2_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&ctrl2_pos, &start_pos) {
            helper_horizontal(
                &(position + ctrl2_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&ctrl2_pos, &start_pos) {
            helper_45_135(
                &(position + ctrl2_pos),
                &(position + start_pos),
                true,
                &mut cst,
            );
        }
        cst
    }
}
impl ShapePool for CubicBezier {}
