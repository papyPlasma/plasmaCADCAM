// use std::collections::HashMap;

use std::collections::HashMap;

use crate::{
    datapool::{DataPools, PointId, PointType, ShapeId, ShapePool},
    math::*,
};

use super::shapes::{ConstructionType, LayerType, Shape, WPoint};

#[derive(Clone)]
pub struct Rectangle {
    pts_ids: HashMap<PointType, PointId>,
    init: bool,
}
impl Rectangle {
    pub fn new(
        data_pools: &mut DataPools,
        bl_point: &WPoint,
        w: f64,
        h: f64,
        snap_distance: f64,
    ) -> (ShapeId, PointId) {
        let position = *bl_point;
        let bl = *bl_point - position;

        let (w, h) = if w == 0. || h == 0. {
            (5. * snap_distance, 5. * snap_distance)
        } else {
            (w, h)
        };
        let pos_id = data_pools.points_pool.insert(&position);
        let bl_id = data_pools.points_pool.insert(&bl);
        let tl_id = data_pools.points_pool.insert(&WPoint::new(0., h));
        let tr_id = data_pools.points_pool.insert(&WPoint::new(w, h));
        let br_id = data_pools.points_pool.insert(&WPoint::new(w, 0.));

        let mut pts_ids = HashMap::new();
        pts_ids.insert(PointType::Position, pos_id);
        pts_ids.insert(PointType::BL, bl_id);
        pts_ids.insert(PointType::TL, tl_id);
        pts_ids.insert(PointType::TR, tr_id);
        pts_ids.insert(PointType::BR, br_id);

        let rectangle = Rectangle {
            pts_ids,
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(rectangle);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        data_pools.pts_to_shs_pool.insert(bl_id, sh_id);
        data_pools.pts_to_shs_pool.insert(tl_id, sh_id);
        data_pools.pts_to_shs_pool.insert(tr_id, sh_id);
        data_pools.pts_to_shs_pool.insert(br_id, sh_id);
        (sh_id, tr_id)
    }
}
impl Shape for Rectangle {
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
        let bl = pts_pos.get(&PointType::BL).unwrap().1;
        let tl = pts_pos.get(&PointType::TL).unwrap().1;
        let tr = pts_pos.get(&PointType::TR).unwrap().1;
        let br = pts_pos.get(&PointType::BR).unwrap().1;
        let pt = *pt - position;
        point_on_segment(&bl, &tl, &pt, precision)
            || point_on_segment(&tl, &tr, &pt, precision)
            || point_on_segment(&tr, &br, &pt, precision)
            || point_on_segment(&br, &bl, &pt, precision)
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
        let (bl_id, mut bl_pos) = pts_pos.get(&PointType::BL).cloned().unwrap();
        let (tl_id, mut tl_pos) = pts_pos.get(&PointType::TL).cloned().unwrap();
        let (tr_id, mut tr_pos) = pts_pos.get(&PointType::TR).cloned().unwrap();
        let (br_id, mut br_pos) = pts_pos.get(&PointType::BR).cloned().unwrap();

        if *pt_id == bl_id {
            bl_pos = rel_pick_point;
            if bl_pos.wx >= tr_pos.wx {
                bl_pos.wx = tr_pos.wx - snap_distance;
            }
            if bl_pos.wy <= tr_pos.wy {
                bl_pos.wy = tr_pos.wy + snap_distance;
            }
            tl_pos.wx = bl_pos.wx;
            br_pos.wy = bl_pos.wy;
        }
        if *pt_id == tl_id {
            tl_pos = rel_pick_point;
            if tl_pos.wx >= br_pos.wx {
                tl_pos.wx = br_pos.wx - snap_distance;
            }
            if tl_pos.wy >= br_pos.wy {
                tl_pos.wy = br_pos.wy - snap_distance;
            }
            tr_pos.wy = tl_pos.wy;
            bl_pos.wx = tl_pos.wx;
        }
        if *pt_id == tr_id {
            tr_pos = rel_pick_point;
            if tr_pos.wx <= bl_pos.wx {
                tr_pos.wx = bl_pos.wx + snap_distance;
            }
            if tr_pos.wy >= bl_pos.wy {
                tr_pos.wy = bl_pos.wy - snap_distance;
            }
            tl_pos.wy = tr_pos.wy;
            br_pos.wx = tr_pos.wx;
        }
        if *pt_id == br_id {
            br_pos = rel_pick_point;
            if br_pos.wx <= tl_pos.wx {
                br_pos.wx = tl_pos.wx + snap_distance;
            }
            if br_pos.wy <= tl_pos.wy {
                br_pos.wy = tl_pos.wy + snap_distance;
            }
            tr_pos.wx = br_pos.wx;
            bl_pos.wy = br_pos.wy;
        }
        pts_pos.insert(PointType::BL, (bl_id, bl_pos));
        pts_pos.insert(PointType::TL, (tl_id, tl_pos));
        pts_pos.insert(PointType::TR, (tr_id, tr_pos));
        pts_pos.insert(PointType::BR, (br_id, br_pos));
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
        let (_, bl_pos) = pts_pos.get(&PointType::BL).unwrap();
        let (_, tl_pos) = pts_pos.get(&PointType::TL).unwrap();
        let (_, tr_pos) = pts_pos.get(&PointType::TR).unwrap();
        let (_, br_pos) = pts_pos.get(&PointType::BR).unwrap();
        cst.push(ConstructionType::Move(position + bl_pos));
        cst.push(ConstructionType::Line(position + tl_pos));
        cst.push(ConstructionType::Line(position + tr_pos));
        cst.push(ConstructionType::Line(position + br_pos));
        cst.push(ConstructionType::Line(position + bl_pos));
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
        let (bl_id, bl_pos) = pts_pos.get(&PointType::BL).unwrap();
        let (tl_id, tl_pos) = pts_pos.get(&PointType::TL).unwrap();
        let (tr_id, tr_pos) = pts_pos.get(&PointType::TR).unwrap();
        let (br_id, br_pos) = pts_pos.get(&PointType::BR).unwrap();
        hdles.push((*bl_id, position + bl_pos));
        hdles.push((*tl_id, position + tl_pos));
        hdles.push((*tr_id, position + tr_pos));
        hdles.push((*br_id, position + br_pos));
        push_handles(&mut cst, &hdles, opt_sel_id, size_handle);
        cst
    }
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];

        let (_, position) = pts_pos.get(&PointType::Position).unwrap();
        let (_, bl_pos) = pts_pos.get(&PointType::BL).unwrap();
        let (_, tl_pos) = pts_pos.get(&PointType::TL).unwrap();
        let (_, tr_pos) = pts_pos.get(&PointType::TR).unwrap();
        let (_, br_pos) = pts_pos.get(&PointType::BR).unwrap();

        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));

        if is_aligned_45_or_135(&bl_pos, &tr_pos) {
            helper_45_135(&(position + bl_pos), &(position + tr_pos), true, &mut cst);
        }

        if is_aligned_45_or_135(&tl_pos, &br_pos) {
            helper_45_135(&(position + tl_pos), &(position + br_pos), true, &mut cst);
        }
        cst
    }
}
impl ShapePool for Rectangle {}
