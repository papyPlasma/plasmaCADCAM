// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

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
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait Shape {
    fn is_init(&self) -> bool;
    fn init_done(&mut self);

    fn get_pos(&self) -> WPos;
    fn get_step_r(&self, grab_handle_precision: f64) -> f64;

    fn get_pos_from_ratio(&self, r: f64) -> WPos;
    fn get_ratio_from_pos(&self, pos: &WPos) -> f64;

    // Perpendicular projection of the point on the shape, then take the
    // ratio of the point found
    fn get_projected_pos(&self, pick_pos: &WPos) -> WPos;
    fn split(&self, pos: &WPos) -> (Option<Box<dyn Shape>>, Option<Box<dyn Shape>>);

    fn dist(&self, pick_pos: &WPos) -> f64;

    fn get_shape_point_type_under_pick_pos(
        &mut self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType>;
    fn clear_selection(&mut self);
    fn set_selected(&mut self, selected: bool);
    fn deselect_all_points(&mut self);
    fn is_selected(&self) -> bool;
    fn move_selection(&mut self, pick_pos: &WPos, pick_pos_ms_dwn: &WPos, magnet_distance: f64);
    fn select_point_type(&mut self, point_type: &PointType);

    fn save_current_position(&mut self);
    fn get_saved_position(&self) -> WPos;
    fn get_construction(&self) -> Vec<ConstructionType>;
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType>;
    fn get_helpers_construction(&self) -> Vec<ConstructionType>;
    fn get_bounded_rectangle(&self) -> [WPos; 2];
    fn magnet_to_point(&self, pick_pos: &mut WPos, magnet_distance: f64);
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
    Move(WPos),
    Line(WPos),
    QuadBezier(WPos, WPos),
    CubicBezier(WPos, WPos, WPos),
    Ellipse(WPos, WPos, f64, f64, f64, bool),
    Rectangle(WPos, WPos, bool),
    Text(WPos, String),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum PointType {
    #[default]
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

#[derive(Copy, Clone, Debug)]
pub struct ShapeId(pub usize);
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

static COUNTER_GROUPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug)]
pub struct GroupId(usize);
impl GroupId {
    pub fn new_id() -> GroupId {
        GroupId(COUNTER_GROUPS.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for GroupId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for GroupId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Hash for GroupId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for GroupId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for GroupId {}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub wpos: WPos,
    pub magnetic: bool,
    pub draggable: bool,
    pub selected: bool,
}
impl Point {
    pub fn new(wpos: &WPos, magnetic: bool, draggable: bool, selected: bool) -> Point {
        Point {
            wpos: *wpos,
            magnetic,
            draggable,
            selected,
        }
    }
}

// #[derive(Copy, Clone, Debug)]
// pub struct PointProp {
//     pub magnetic: bool,
//     pub draggable: bool,
//     pub selected: bool,
// }
// impl PointProp {
//     pub fn new(magnetic: bool, draggable: bool, selected: bool) -> PointProp {
//         PointProp {
//             magnetic,
//             draggable,
//             selected,
//         }
//     }
// }

#[derive(Copy, Clone, Debug)]
pub struct WPos {
    pub wx: f64,
    pub wy: f64,
}
impl WPos {
    pub fn new(wx: f64, wy: f64) -> Self {
        WPos { wx, wy }
    }
    pub fn zero() -> Self {
        WPos { wx: 0., wy: 0. }
    }
    pub fn snap(&mut self, snap_distance: f64) {
        *self = (*self / snap_distance).round() * snap_distance;
    }
    pub fn to_canvas(&self, scale: f64, offset: CPos) -> CPos {
        let canvas_x = (self.wx * scale) + offset.cx;
        let canvas_y = (self.wy * scale) + offset.cy;
        CPos {
            cx: canvas_x,
            cy: canvas_y,
        }
    }
    #[allow(dead_code)]
    pub fn round(&self) -> WPos {
        WPos {
            wx: self.wx.round(),
            wy: self.wy.round(),
        }
    }
    #[allow(dead_code)]
    pub fn addxy(&self, wx: f64, wy: f64) -> WPos {
        WPos {
            wx: self.wx + wx,
            wy: self.wy + wy,
        }
    }
    pub fn abs(&self) -> WPos {
        WPos {
            wx: self.wx.abs(),
            wy: self.wy.abs(),
        }
    }
    pub fn dist(&self, other: &WPos) -> f64 {
        let dpt = *self - *other;
        (dpt.wx * dpt.wx + dpt.wy * dpt.wy).sqrt()
    }
    #[allow(dead_code)]
    pub fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }
    #[allow(dead_code)]
    pub fn norm2(&self) -> f64 {
        self.wx * self.wx + self.wy * self.wy
    }
    #[allow(dead_code)]
    pub fn dot(&self, other: &WPos) -> f64 {
        self.wx * other.wx + self.wy * other.wy
    }
    pub fn lerp(&self, other: &WPos, t: f64) -> WPos {
        WPos {
            wx: self.wx + t * (other.wx - self.wx),
            wy: self.wy + t * (other.wy - self.wy),
        }
    }
    // pub fn sign_dist_to_seg(&self, pos1: &WPos, pos2: &WPos) -> f64 {
    //     let a = pos2.wy - pos1.wy;
    //     let b = pos1.wx - pos2.wx;
    //     let c = pos1.wx * pos2.wy - pos2.wx * pos1.wy;

    //     let num = a * self.wx + b * self.wy + c;
    //     let den = (a * a + b * b).sqrt();

    //     let dist = if den != 0. {
    //         num / den
    //     } else {
    //         self.dist(pos1)
    //     };
    //     dist
    // }
    pub fn sign_dist_to_seg(&self, pos1: &WPos, pos2: &WPos) -> f64 {
        let num =
            (pos2.wx - pos1.wx) * (pos1.wy - self.wy) - (pos1.wx - self.wx) * (pos2.wy - pos1.wy);
        let den = ((pos2.wx - pos1.wx).powi(2) + (pos2.wy - pos1.wy).powi(2)).sqrt();
        if den > 0. {
            num / den
        } else {
            // pos1 == pos2, hence return the distance between the 2 points
            self.dist(&pos1)
        }
    }
    // Find the projection of a point onto a line segment defined by two points
    pub fn project_to_seg(&self, pos1: &WPos, pos2: &WPos) -> WPos {
        let pos_v = WPos::new(self.wx - pos1.wx, self.wy - pos1.wy);
        let dir_v = WPos::new(pos2.wx - pos1.wx, pos2.wy - pos1.wy);
        *pos1 + dir_v * (pos_v.dot(&dir_v) / dir_v.norm2())
    }
    pub fn ratio(&self, pos1: &WPos, pos2: &WPos) -> f64 {
        let vec1 = self - pos1;
        let vec2 = pos2 - pos1;
        let norm1 = vec1.norm();
        let norm2 = vec2.norm();
        if norm2 > 0. {
            if vec1.dot(&vec2) >= 0. {
                norm1 / norm2
            } else {
                -norm1 / norm2
            }
        } else {
            0.
        }
    }
}
impl Default for WPos {
    fn default() -> Self {
        WPos { wx: 0.0, wy: 0.0 }
    }
}
impl Neg for WPos {
    type Output = WPos;

    fn neg(self) -> WPos {
        WPos {
            wx: -self.wx,
            wy: -self.wy,
        }
    }
}
impl Add for WPos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<&WPos> for &WPos {
    type Output = WPos;

    fn add(self, other: &WPos) -> WPos {
        WPos {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<f64> for WPos {
    type Output = WPos;
    fn add(self, scalar: f64) -> Self::Output {
        WPos {
            wx: self.wx + scalar,
            wy: self.wy + scalar,
        }
    }
}
impl AddAssign for WPos {
    fn add_assign(&mut self, other: WPos) {
        self.wx += other.wx;
        self.wy += other.wy;
    }
}
impl AddAssign<f64> for WPos {
    fn add_assign(&mut self, scalar: f64) {
        self.wx += scalar;
        self.wy += scalar;
    }
}
impl Sub for WPos {
    type Output = WPos;
    fn sub(self, other: WPos) -> WPos {
        WPos {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl Sub<&WPos> for &WPos {
    type Output = WPos;

    fn sub(self, other: &WPos) -> WPos {
        WPos {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl Sub<f64> for WPos {
    type Output = WPos;
    fn sub(self, scalar: f64) -> Self::Output {
        WPos {
            wx: self.wx - scalar,
            wy: self.wy - scalar,
        }
    }
}
impl SubAssign for WPos {
    fn sub_assign(&mut self, other: WPos) {
        self.wx -= other.wx;
        self.wy -= other.wy;
    }
}
impl Div<f64> for WPos {
    type Output = WPos;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        WPos {
            wx: self.wx / rhs,
            wy: self.wy / rhs,
        }
    }
}
impl DivAssign<f64> for WPos {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        self.wx /= rhs;
        self.wy /= rhs;
    }
}
impl Mul<f64> for WPos {
    type Output = WPos;

    fn mul(self, rhs: f64) -> Self::Output {
        WPos {
            wx: self.wx * rhs,
            wy: self.wy * rhs,
        }
    }
}
impl MulAssign<f64> for WPos {
    fn mul_assign(&mut self, rhs: f64) {
        self.wx *= rhs;
        self.wy *= rhs;
    }
}
impl PartialEq for WPos {
    fn eq(&self, other: &Self) -> bool {
        self.wx == other.wx && self.wy == other.wy
    }
}
impl Eq for WPos {}

#[derive(Copy, Clone, Debug)]
pub struct CPos {
    pub cx: f64,
    pub cy: f64,
}
impl CPos {
    pub fn to_world(&self, scale: f64, offset: CPos) -> WPos {
        let world_x = (self.cx - offset.cx) / scale;
        let world_y = (self.cy - offset.cy) / scale;
        WPos {
            wx: world_x,
            wy: world_y,
        }
    }
}
impl Default for CPos {
    fn default() -> Self {
        CPos { cx: 0., cy: 0. }
    }
}
impl Add for CPos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            cx: self.cx + other.cx,
            cy: self.cy + other.cy,
        }
    }
}
impl Add<f64> for CPos {
    type Output = CPos;
    fn add(self, scalar: f64) -> Self::Output {
        CPos {
            cx: self.cx + scalar,
            cy: self.cy + scalar,
        }
    }
}
impl AddAssign for CPos {
    fn add_assign(&mut self, other: CPos) {
        self.cx += other.cx;
        self.cy += other.cy;
    }
}
impl AddAssign<f64> for CPos {
    fn add_assign(&mut self, scalar: f64) {
        self.cx += scalar;
        self.cy += scalar;
    }
}
impl Sub for CPos {
    type Output = CPos;
    fn sub(self, other: CPos) -> CPos {
        CPos {
            cx: self.cx - other.cx,
            cy: self.cy - other.cy,
        }
    }
}
impl SubAssign for CPos {
    fn sub_assign(&mut self, other: CPos) {
        self.cx -= other.cx;
        self.cy -= other.cy;
    }
}
impl Div<f64> for CPos {
    type Output = CPos;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0. {
            panic!("Division by zero");
        }
        CPos {
            cx: self.cx / rhs,
            cy: self.cy / rhs,
        }
    }
}
impl DivAssign<f64> for CPos {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0. {
            panic!("Division by zero");
        }
        self.cx /= rhs;
        self.cy /= rhs;
    }
}
impl Mul<f64> for CPos {
    type Output = CPos;

    fn mul(self, rhs: f64) -> Self::Output {
        CPos {
            cx: self.cx * rhs,
            cy: self.cy * rhs,
        }
    }
}
impl MulAssign<f64> for CPos {
    fn mul_assign(&mut self, rhs: f64) {
        self.cx *= rhs;
        self.cy *= rhs;
    }
}
impl PartialEq for CPos {
    fn eq(&self, other: &Self) -> bool {
        self.cx == other.cx && self.cy == other.cy
    }
}
impl Eq for CPos {}
