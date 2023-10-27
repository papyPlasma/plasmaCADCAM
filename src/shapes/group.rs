use std::collections::HashMap;

use super::shapes::{ConstructionType, PtIdProp, Shape};
use crate::{
    datapool::{DataPools, PointId, PointProperty, PointType, ShapeId, ShapePool, WPoint},
    math::*,
};

#[derive(Clone)]
pub struct Group {
    pts_ids: PtIdProp,
    shapes_ids: Vec<ShapeId>,
    init: bool,
}
impl Group {
    pub fn new(
        data_pools: &mut DataPools,
        shapes_ids: &Vec<ShapeId>,
        position: &WPoint,
        _snap_distance: f64,
    ) -> (ShapeId, (PointId, PointProperty)) {
        let pos_id = data_pools.points_pool.insert(&position);

        let mut pts_ids = HashMap::new();
        let pt_end_id_prop = (pos_id, PointProperty::new(false, false));
        pts_ids.insert(PointType::Position, pt_end_id_prop);

        let group = Group {
            pts_ids,
            shapes_ids: shapes_ids.clone(),
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(group);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        (sh_id, pt_end_id_prop)
    }
}
impl Shape for Group {
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
        _pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        _pt: &WPoint,
        _precision: f64,
    ) -> bool {
        // TODO
        false
    }

    fn update_points_pos(
        &self,
        _pts_pos: &mut HashMap<PointType, (PointId, WPoint)>,
        _pt_id: &PointId,
        _pick_pt: &WPoint,
        _snap_distance: f64,
    ) {
        // TODO
    }
    fn get_construction(
        &self,
        _pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        _selected: bool,
    ) -> Vec<ConstructionType> {
        // TODO
        vec![]
    }
    fn get_handles_construction(
        &self,
        _pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        _opt_sel_id_prop: &Option<(PointId, PointProperty)>,
        _size_handle: f64,
    ) -> Vec<ConstructionType> {
        // TODO
        vec![]
    }
    fn get_helpers_construction(
        &self,
        _pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType> {
        // TODO
        vec![]
    }
}
impl ShapePool for Group {}
