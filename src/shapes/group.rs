use std::collections::HashMap;

use super::shapes::{ConstructionType, Shape, WPoint};
use crate::{
    datapool::{DataPools, PointId, PointType, ShapeId, ShapePool},
    math::*,
};

#[derive(Clone)]
pub struct Group {
    pts_ids: HashMap<PointType, PointId>,
    shapes_ids: Vec<ShapeId>,
    init: bool,
}
impl Group {
    pub fn new(
        data_pools: &mut DataPools,
        shapes_ids: &Vec<ShapeId>,
        position: &WPoint,
        _snap_distance: f64,
    ) -> (ShapeId, PointId) {
        let pos_id = data_pools.points_pool.insert(&position);

        let mut pts_ids = HashMap::new();
        pts_ids.insert(PointType::Position, pos_id);

        let group = Group {
            pts_ids,
            shapes_ids: shapes_ids.clone(),
            init: true,
        };
        let sh_id = data_pools.shapes_pool.insert(group);
        data_pools.pts_to_shs_pool.insert(pos_id, sh_id);
        (sh_id, pos_id)
    }
}
impl Shape for Group {
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
        _opt_sel_id: &Option<PointId>,
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
