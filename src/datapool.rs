use crate::math::*;
use crate::shapes::shapes::{ConstructionType, Shape};
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::{Deref, DerefMut};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER_POINTS: AtomicUsize = AtomicUsize::new(0);
static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PointProperty {
    magnetic: bool,
    dragable: bool,
}
impl PointProperty {
    pub fn new(magnetic: bool, dragable: bool) -> PointProperty {
        PointProperty { magnetic, dragable }
    }
}
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
            .get(&self.shapes_pool.get(sh_id).unwrap().get_pos_id().0)
            .unwrap()
    }
    pub fn magnet_to_point(
        &self,
        pick_point: &mut WPoint,
        excluded_sh_id: Option<ShapeId>,
        magnet_distance: f64,
    ) {
        for (sh_id, shape) in self.shapes_pool.iter() {
            let exclude = if let Some(exc_sh_id) = excluded_sh_id {
                if *sh_id == exc_sh_id {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !exclude {
                for (_, (pt_id, pt_prop)) in shape.get_points_ids().iter() {
                    if pt_prop.magnetic {
                        let position = self.get_shape_position(&sh_id);
                        let point = self.points_pool.get(pt_id).unwrap();
                        if pick_point.dist(&(*point + position)) < magnet_distance {
                            *pick_point = *point + position;
                            break;
                        }
                    }
                }
            }
        }
    }
    pub fn move_shape(&mut self, sh_id: &ShapeId, rel_pos: &WPoint, magnet_distance: f64) {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let (position_id, _) = shape.get_pos_id();
        self.points_pool.modify(&position_id, &rel_pos);
    }
    pub fn move_shape_point(
        &mut self,
        sh_id: &ShapeId,
        pt_sel_id_prop: &(PointId, PointProperty),
        pick_pt: &WPoint,
        snap_distance: f64,
        magnet_distance: f64,
    ) {
        let mut pick_pt = *pick_pt;
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        // Retreive the points positions
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, (pt_id, _)) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        //
        if pt_sel_id_prop.1.magnetic {
            self.magnet_to_point(&mut pick_pt, Some(*sh_id), magnet_distance);
        }
        // Update positions from shape type
        shape.update_points_pos(&mut pts_pos, &pt_sel_id_prop.0, &pick_pt, snap_distance);

        // And set modifications on the points pool
        for (_, (pt_id, pt_pos)) in pts_pos.iter() {
            self.points_pool.modify(pt_id, pt_pos);
        }
    }
    pub fn is_point_on_shape(&self, sh_id: &ShapeId, pt: &WPoint, precision: f64) -> bool {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, (pt_id, _)) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.is_point_on_shape(&pts_pos, pt, precision)
    }
    pub fn get_construction(&self, sh_id: &ShapeId, selected: bool) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, (pt_id, _)) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_construction(&pts_pos, selected)
    }
    pub fn get_handles_construction(
        &self,
        sh_id: &ShapeId,
        opt_sel_id: &Option<(PointId, PointProperty)>,
        size_handle: f64,
    ) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(&sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, (pt_id, _)) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_handles_construction(&pts_pos, opt_sel_id, size_handle)
    }
    pub fn get_helpers_construction(&self, sh_id: &ShapeId) -> Vec<ConstructionType> {
        let shape = self.shapes_pool.get(&sh_id).unwrap();
        let pts_ids = shape.get_points_ids();
        let mut pts_pos: HashMap<PointType, (PointId, WPoint)> = HashMap::new();
        for (pt_type, (pt_id, _)) in pts_ids.iter() {
            pts_pos.insert(*pt_type, (*pt_id, *self.points_pool.get(pt_id).unwrap()));
        }
        shape.get_helpers_construction(&pts_pos)
    }
    pub fn get_id_from_position(
        &self,
        pick_pt: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<(PointId, PointProperty)> {
        for (_, shape) in self.shapes_pool.iter() {
            let (shape_pos_id, _) = shape.get_pos_id();
            let shape_pos = self.points_pool.get(&shape_pos_id).unwrap();
            for (pt_id, pt_prop) in shape.get_points_ids().values() {
                let pt = self.points_pool.get(pt_id).unwrap();
                if is_point_on_point(pick_pt, &(shape_pos + pt), grab_handle_precision) {
                    return Some((pt_id.clone(), pt_prop.clone()));
                }
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

#[derive(Copy, Clone, Debug)]
pub struct WPoint {
    pub wx: f64,
    pub wy: f64,
}
impl WPoint {
    pub fn new(wx: f64, wy: f64) -> Self {
        WPoint { wx, wy }
    }
    pub fn to_canvas(&self, scale: f64, offset: CXY) -> CXY {
        let canvas_x = (self.wx * scale) + offset.cx;
        let canvas_y = (self.wy * scale) + offset.cy;
        CXY {
            cx: canvas_x,
            cy: canvas_y,
        }
    }
    #[allow(dead_code)]
    pub fn round(&self) -> WPoint {
        WPoint {
            wx: self.wx.round(),
            wy: self.wy.round(),
        }
    }
    #[allow(dead_code)]
    pub fn addxy(&self, wx: f64, wy: f64) -> WPoint {
        WPoint {
            wx: self.wx + wx,
            wy: self.wy + wy,
        }
    }
    pub fn abs(&self) -> WPoint {
        WPoint {
            wx: self.wx.abs(),
            wy: self.wy.abs(),
        }
    }
    pub fn dist(&self, other: &WPoint) -> f64 {
        let dpt = *self - *other;
        (dpt.wx * dpt.wx + dpt.wy * dpt.wy).sqrt()
    }
    #[allow(dead_code)]
    pub fn norm(&self) -> f64 {
        (self.wx * self.wx + self.wy * self.wy).sqrt()
    }
    pub fn lerp(&self, other: &WPoint, t: f64) -> WPoint {
        WPoint {
            wx: self.wx + t * (other.wx - self.wx),
            wy: self.wy + t * (other.wy - self.wy),
        }
    }
}
impl Default for WPoint {
    fn default() -> Self {
        WPoint { wx: 0.0, wy: 0.0 }
    }
}
impl Neg for WPoint {
    type Output = WPoint;

    fn neg(self) -> WPoint {
        WPoint {
            wx: -self.wx,
            wy: -self.wy,
        }
    }
}
impl Add for WPoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<&WPoint> for &WPoint {
    type Output = WPoint;

    fn add(self, other: &WPoint) -> WPoint {
        WPoint {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<f64> for WPoint {
    type Output = WPoint;
    fn add(self, scalar: f64) -> Self::Output {
        WPoint {
            wx: self.wx + scalar,
            wy: self.wy + scalar,
        }
    }
}
impl AddAssign for WPoint {
    fn add_assign(&mut self, other: WPoint) {
        self.wx += other.wx;
        self.wy += other.wy;
    }
}
impl AddAssign<f64> for WPoint {
    fn add_assign(&mut self, scalar: f64) {
        self.wx += scalar;
        self.wy += scalar;
    }
}
impl Sub for WPoint {
    type Output = WPoint;
    fn sub(self, other: WPoint) -> WPoint {
        WPoint {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl Sub<&WPoint> for &WPoint {
    type Output = WPoint;

    fn sub(self, other: &WPoint) -> WPoint {
        WPoint {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl Sub<f64> for WPoint {
    type Output = WPoint;
    fn sub(self, scalar: f64) -> Self::Output {
        WPoint {
            wx: self.wx - scalar,
            wy: self.wy - scalar,
        }
    }
}
impl SubAssign for WPoint {
    fn sub_assign(&mut self, other: WPoint) {
        self.wx -= other.wx;
        self.wy -= other.wy;
    }
}
impl Div<f64> for WPoint {
    type Output = WPoint;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        WPoint {
            wx: self.wx / rhs,
            wy: self.wy / rhs,
        }
    }
}
impl DivAssign<f64> for WPoint {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        self.wx /= rhs;
        self.wy /= rhs;
    }
}
impl Mul<f64> for WPoint {
    type Output = WPoint;

    fn mul(self, rhs: f64) -> Self::Output {
        WPoint {
            wx: self.wx * rhs,
            wy: self.wy * rhs,
        }
    }
}
impl MulAssign<f64> for WPoint {
    fn mul_assign(&mut self, rhs: f64) {
        self.wx *= rhs;
        self.wy *= rhs;
    }
}
impl PartialEq for WPoint {
    fn eq(&self, other: &Self) -> bool {
        self.wx == other.wx && self.wy == other.wy
    }
}
impl Eq for WPoint {}

#[derive(Copy, Clone, Debug)]
pub struct CXY {
    pub cx: f64,
    pub cy: f64,
}
impl CXY {
    pub fn to_world(&self, scale: f64, offset: CXY) -> WPoint {
        let world_x = (self.cx - offset.cx) / scale;
        let world_y = (self.cy - offset.cy) / scale;
        WPoint {
            wx: world_x,
            wy: world_y,
        }
    }
}
impl Default for CXY {
    fn default() -> Self {
        CXY { cx: 0., cy: 0. }
    }
}
impl Add for CXY {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            cx: self.cx + other.cx,
            cy: self.cy + other.cy,
        }
    }
}
impl Add<f64> for CXY {
    type Output = CXY;
    fn add(self, scalar: f64) -> Self::Output {
        CXY {
            cx: self.cx + scalar,
            cy: self.cy + scalar,
        }
    }
}
impl AddAssign for CXY {
    fn add_assign(&mut self, other: CXY) {
        self.cx += other.cx;
        self.cy += other.cy;
    }
}
impl AddAssign<f64> for CXY {
    fn add_assign(&mut self, scalar: f64) {
        self.cx += scalar;
        self.cy += scalar;
    }
}
impl Sub for CXY {
    type Output = CXY;
    fn sub(self, other: CXY) -> CXY {
        CXY {
            cx: self.cx - other.cx,
            cy: self.cy - other.cy,
        }
    }
}
impl SubAssign for CXY {
    fn sub_assign(&mut self, other: CXY) {
        self.cx -= other.cx;
        self.cy -= other.cy;
    }
}
impl Div<f64> for CXY {
    type Output = CXY;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0. {
            panic!("Division by zero");
        }
        CXY {
            cx: self.cx / rhs,
            cy: self.cy / rhs,
        }
    }
}
impl DivAssign<f64> for CXY {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0. {
            panic!("Division by zero");
        }
        self.cx /= rhs;
        self.cy /= rhs;
    }
}
impl Mul<f64> for CXY {
    type Output = CXY;

    fn mul(self, rhs: f64) -> Self::Output {
        CXY {
            cx: self.cx * rhs,
            cy: self.cy * rhs,
        }
    }
}
impl MulAssign<f64> for CXY {
    fn mul_assign(&mut self, rhs: f64) {
        self.cx *= rhs;
        self.cy *= rhs;
    }
}
impl PartialEq for CXY {
    fn eq(&self, other: &Self) -> bool {
        self.cx == other.cx && self.cy == other.cy
    }
}
impl Eq for CXY {}
