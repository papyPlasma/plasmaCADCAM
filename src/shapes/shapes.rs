use crate::datapool::PointId;
use crate::datapool::PointType;
use std::collections::HashMap;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

pub trait Shape {
    fn is_init(&self) -> bool;
    fn get_pos_id(&self) -> PointId;
    fn init_done(&mut self);
    fn get_points_ids(&self) -> HashMap<PointType, PointId>;
    fn is_point_on_shape(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        pt: &WPoint,
        precision: f64,
    ) -> bool;
    fn update_points_pos(
        &self,
        pts_pos: &mut HashMap<PointType, (PointId, WPoint)>,
        pt_id: &PointId,
        pick_pt: &WPoint,
        snap_distance: f64,
    );
    fn get_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        selected: bool,
    ) -> Vec<ConstructionType>;
    fn get_handles_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        opt_sel_id: &Option<PointId>,
        size_handle: f64,
    ) -> Vec<ConstructionType>;
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType>;
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct ShapeParameters {
    pub handles_size: f64,
    pub highlight_size: f64,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum LayerType {
    Worksheet,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
    SelectionTool,
    Selected,
    Highlight,
    Handle(bool),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ConstructionType {
    Layer(LayerType),
    Move(WPoint),
    Line(WPoint),
    QuadBezier(WPoint, WPoint),
    CubicBezier(WPoint, WPoint, WPoint),
    Ellipse(WPoint, WPoint, f64, f64, f64, bool),
    Rectangle(WPoint, WPoint, bool),
    Text(WPoint, String),
}

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
