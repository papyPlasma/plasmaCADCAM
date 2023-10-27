use crate::math::*;
use crate::shapes::shapes::{ConstructionType, Shape, WPoint};
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER_POINTS: AtomicUsize = AtomicUsize::new(0);
static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PointType {
    Position,
    Start,
    End,
    Center,
    Radius,
    StartAngle,
    EndAngle,
    BL,
    TL,
    TR,
    BR,
    Ctrl,
    Ctrl1,
    Ctrl2,
}
pub struct DataPools {
    pub points_pool: PointsPool,
    pub shapes_pool: ShapesPool,
    pub pts_to_shs_pool: PointsToShapesPool,
}
impl DataPools {
    pub fn get_shape_position(&self, sh_id: &ShapeId) -> WPoint {
        *self
            .points_pool
            .get(&self.shapes_pool.get(sh_id).unwrap().get_pos_id())
            .unwrap()
    }
    pub fn magnet_to_point(&self, pick_point: &mut WPoint, magnet_distance: f64) {
        for (pt_id, point) in self.points_pool.iter() {
            let sh_id = self.pts_to_shs_pool.get(pt_id).unwrap();
            let pos = self.get_shape_position(&sh_id);
            if pick_point.dist(&(*point + pos)) < magnet_distance {
                *pick_point = *point + pos;
                break;
            }
        }
    }
    pub fn move_shape(&mut self, sh_id: &ShapeId, position: &WPoint) {
        let position_id = self.shapes_pool.get(sh_id).unwrap().get_pos_id();
        self.points_pool.modify(&position_id, &position);
    }
    pub fn move_shape_point(
        &mut self,
        sh_id: &ShapeId,
        pt_sel_id: &PointId,
        pick_pt: &WPoint,
        snap_distance: f64,
    ) {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        // Retreive the points positions
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, pt_id) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        // Update positions from shape type
        shape.update_points_pos(&mut pts_pos, &pt_sel_id, &pick_pt, snap_distance);

        // And set modifications on the points pool
        for (_, (pt_id, pt_pos)) in pts_pos.iter() {
            self.points_pool.modify(pt_id, pt_pos);
        }
    }
    pub fn is_point_on_shape(&self, sh_id: &ShapeId, pt: &WPoint, precision: f64) -> bool {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, pt_id) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.is_point_on_shape(&pts_pos, pt, precision)
    }
    pub fn get_construction(&self, sh_id: &ShapeId, selected: bool) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, pt_id) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_construction(&pts_pos, selected)
    }
    pub fn get_handles_construction(
        &self,
        sh_id: &ShapeId,
        opt_sel_id: &Option<PointId>,
        size_handle: f64,
    ) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(&sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, pt_id) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_handles_construction(&pts_pos, opt_sel_id, size_handle)
    }
    pub fn get_helpers_construction(&self, sh_id: &ShapeId) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(&sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, pt_id) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_helpers_construction(&pts_pos)
    }
    pub fn get_id_from_position(
        &self,
        pick_pt: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<PointId> {
        for (pt_id, osh_id) in self.pts_to_shs_pool.iter() {
            let position_id = self.shapes_pool.get(osh_id).unwrap().get_pos_id();
            let position = self.points_pool.get(&position_id).unwrap();
            let pt = self.points_pool.get(pt_id).unwrap();
            if is_point_on_point(pick_pt, &(position + pt), grab_handle_precision) {
                return Some(pt_id.clone());
            }
        }
        None
    }
}

pub struct PointsPool(HashMap<PointId, WPoint>);
impl std::ops::Deref for PointsPool {
    type Target = HashMap<PointId, WPoint>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for PointsPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl PointsPool {
    pub fn new() -> PointsPool {
        PointsPool(HashMap::new())
    }
    pub fn insert(&mut self, pt: &WPoint) -> PointId {
        let pt_id = PointId(COUNTER_POINTS.fetch_add(1, Ordering::Relaxed));
        self.0.insert(pt_id.clone(), *pt);
        pt_id
    }
    pub fn modify(&mut self, pt_id: &PointId, pt: &WPoint) {
        self.0.insert(*pt_id, *pt);
    }
    pub fn get_all(&self) -> &HashMap<PointId, WPoint> {
        &self.0
    }
    pub fn get_all_mut(&mut self) -> &mut HashMap<PointId, WPoint> {
        &mut self.0
    }
}

// Define a trait for all shapes that can be inserted into ShapesPool
pub trait ShapePool: Shape + Any {}

pub struct ShapesPool(HashMap<ShapeId, Box<dyn ShapePool>>);
impl std::ops::Deref for ShapesPool {
    type Target = HashMap<ShapeId, Box<dyn ShapePool>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for ShapesPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl ShapesPool {
    pub fn new() -> ShapesPool {
        ShapesPool(HashMap::new())
    }
    pub fn insert<T: ShapePool>(&mut self, shape: T) -> ShapeId {
        let shape_id = ShapeId(COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed));
        self.0.insert(shape_id, Box::new(shape));
        shape_id
    }
    pub fn modify<T: ShapePool>(&mut self, shape_id: &ShapeId, shape: T) {
        self.0.insert(*shape_id, Box::new(shape));
    }
    pub fn get_all(&self) -> &HashMap<ShapeId, Box<dyn ShapePool>> {
        &self.0
    }
    pub fn get_all_mut(&mut self) -> &mut HashMap<ShapeId, Box<dyn ShapePool>> {
        &mut self.0
    }
}

pub struct PointsToShapesPool(HashMap<PointId, ShapeId>);
impl std::ops::Deref for PointsToShapesPool {
    type Target = HashMap<PointId, ShapeId>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for PointsToShapesPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl PointsToShapesPool {
    pub fn new() -> PointsToShapesPool {
        PointsToShapesPool(HashMap::new())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointId(usize);
impl Deref for PointId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for PointId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Hash for PointId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for PointId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for PointId {}

#[derive(Copy, Clone, Debug)]
pub struct ShapeId(usize);
impl Deref for ShapeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ShapeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Hash for ShapeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for ShapeId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for ShapeId {}
