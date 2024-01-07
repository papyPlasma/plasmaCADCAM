// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
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

use js_sys::Array;
use wasm_bindgen::prelude::*;
use web_sys::CssStyleDeclaration;

use crate::math::*;

pub struct DrawStyles {
    // Drawing colors
    worksheet_color: String,
    dimension_color: String,
    geohelper_color: String,
    origin_color: String,
    grid_color: String,
    selection_color: String,
    selected_color: String,
    background_color: String,
    fill_color: String,
    highlight_color: String,
    // line patterns
    pattern_dashed: JsValue,
    pattern_solid: JsValue,
}
impl DrawStyles {
    pub fn build(style: CssStyleDeclaration) -> Result<DrawStyles, JsValue> {
        let worksheet_color = style.get_property_value("--canvas-worksheet-color")?;
        let dimension_color = style.get_property_value("--canvas-dimension-color")?;
        let geohelper_color = style.get_property_value("--canvas-geohelper-color")?;
        let origin_color = style.get_property_value("--canvas-origin-color")?;
        let grid_color = style.get_property_value("--canvas-grid-color")?;
        let selection_color = style.get_property_value("--canvas-selection-color")?;
        let selected_color = style.get_property_value("--canvas-selected-color")?;
        let background_color = style.get_property_value("--canvas-background-color")?;
        let fill_color = style.get_property_value("--canvas-fill-color")?;
        let highlight_color = style.get_property_value("--canvas-highlight-color")?;
        let dash_pattern = Array::new();
        dash_pattern.push(&JsValue::from_f64(3.0));
        dash_pattern.push(&JsValue::from_f64(3.0));
        let solid_pattern = Array::new();
        Ok(DrawStyles {
            worksheet_color,
            dimension_color,
            geohelper_color,
            origin_color,
            grid_color,
            selection_color,
            selected_color,
            background_color,
            fill_color,
            highlight_color,
            pattern_dashed: JsValue::from(dash_pattern),
            pattern_solid: JsValue::from(solid_pattern),
        })
    }
    pub fn get_default_styles(&self, layer: ConstructionLayer) -> (&str, &str, &JsValue, f64) {
        use ConstructionLayer::*;
        let (fill_color, color, line_dash, line_width) = match layer {
            Worksheet => (
                &self.fill_color,
                &self.worksheet_color,
                &self.pattern_solid,
                1.,
            ),
            Dimension => (
                &self.fill_color,
                &self.dimension_color,
                &self.pattern_solid,
                1.,
            ),
            GeometryHelpers => (
                &self.fill_color,
                &self.geohelper_color,
                &self.pattern_dashed,
                1.,
            ),
            Origin => (
                &self.fill_color,
                &self.origin_color,
                &self.pattern_solid,
                1.,
            ),
            Grid => (&self.fill_color, &self.grid_color, &self.pattern_solid, 1.),
            SelectionTool => (
                &self.fill_color,
                &self.selection_color,
                &self.pattern_dashed,
                1.,
            ),
            Selected => (
                &self.fill_color,
                &self.selected_color,
                &self.pattern_solid,
                2.,
            ),
            Handle => (
                &self.fill_color,
                &self.selected_color,
                &self.pattern_solid,
                1.,
            ),
            Highlight => (
                &self.highlight_color,
                &self.highlight_color,
                &self.pattern_solid,
                1.,
            ),
        };
        (fill_color, color, line_dash, line_width)
    }
    pub fn get_background_color(&self) -> &str {
        &self.background_color
    }
    pub fn get_selected_color(&self) -> &str {
        &self.selected_color
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct ShapeParameters {
    pub handles_size: f64,
    pub highlight_size: f64,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ConstructionLayer {
    Worksheet,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
    SelectionTool,
    Selected,
    Highlight,
    Handle,
}

pub enum ConstructionPattern {
    NoSelection,
    SimpleSelection,
    DoubleSelection,
}
pub enum ConstructionType {
    Move(WPos),
    Point(ConstructionPattern, WPos),
    Segment(ConstructionPattern, WPos, WPos),
    // QBezier(ConstructionPattern, WPos, WPos, WPos),
    // CBezier(ConstructionPattern, WPos, WPos, WPos, WPos),
    ArcEllipse(ConstructionPattern, WPos, WPos, f64, f64),
    Text(WPos, String),
}

// #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
// pub enum PointType {
//     Start,
//     End,
//     Center,
//     Radius,
//     StartAngle,
//     EndAngle,
//     Ctrl,
//     Ctrl1,
//     Ctrl2,
// }

pub enum PointConstraint {
    Equal,
    VerticalAlign,
    HorizontalAlign,
    Direction,
}
pub enum ShapeConstraint {
    Parallel,
    Perpendicular,
}

static COUNTER_GROUPS: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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

static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShapeId(usize);
impl ShapeId {
    pub fn new_id() -> ShapeId {
        ShapeId(COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed))
    }
}
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

static COUNTER_POINTS: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PointId(usize);
impl PointId {
    pub fn new_id() -> PointId {
        PointId(COUNTER_POINTS.fetch_add(1, Ordering::Relaxed))
    }
}
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

static COUNTER_BASIC_SHAPES: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct BasicShapeId(usize);
impl BasicShapeId {
    pub fn new_id() -> BasicShapeId {
        BasicShapeId(COUNTER_BASIC_SHAPES.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for BasicShapeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for BasicShapeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub id: PointId,
    pub wpos: WPos,
    pub saved_wpos: WPos,
    pub magnetic: bool,
    pub draggable: bool,
    pub selected: bool,
}
impl Point {
    pub fn new(wpos: &WPos, magnetic: bool, draggable: bool, selected: bool) -> (Point, PointId) {
        let id = PointId::new_id();
        (
            Point {
                id,
                wpos: *wpos,
                saved_wpos: *wpos,
                magnetic,
                draggable,
                selected,
            },
            id,
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BasicShapeType {
    Line,
    QBezier,
    CBezier,
    ArcEllipse,
}
impl BasicShapeType {
    pub fn segment_get_points(
        &self,
        hpts: &HashSet<PointId>,
        pts: &HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    ) -> Option<(Point, Point)> {
        if let BasicShapeType::Line = self {
            let pts_id: Vec<_> = hpts.into_iter().collect();
            if let Some(pt1_id) = pts_id.get(0) {
                if let Some(pt2_id) = pts_id.get(1) {
                    if let Some((pt1, _)) = pts.get(pt1_id) {
                        if let Some((pt2, _)) = pts.get(pt2_id) {
                            return Some((*pt1, *pt2));
                        }
                    }
                }
            }
        }
        None
    }
    pub fn s_dist(
        &self,
        pos: &WPos,
        hpts: &HashSet<PointId>,
        pts: &HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    ) -> Option<f64> {
        use BasicShapeType::*;
        match self {
            Line => {
                if let Some((pt1, pt2)) = self.segment_get_points(hpts, pts) {
                    Some(pos.s_dist_seg(&pt1.wpos, &pt2.wpos))
                } else {
                    // Something when wrong, return None
                    None
                }
            }
            QBezier => None,    //TODO
            CBezier => None,    //TODO
            ArcEllipse => None, //TODO
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BasicShape {
    pub id: BasicShapeId,
    pub bs_typ: BasicShapeType,
    pub selected: bool,
}
impl BasicShape {
    pub fn new(bs_typ: BasicShapeType, selected: bool) -> (BasicShape, BasicShapeId) {
        let id = BasicShapeId::new_id();
        (
            BasicShape {
                id,
                bs_typ,
                selected,
            },
            id,
        )
    }
    pub fn s_dist(
        &self,
        pos: &WPos,
        hpts: &HashSet<PointId>,
        pts: &HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    ) -> Option<f64> {
        self.bs_typ.s_dist(pos, hpts, pts)
    }
    pub fn get_bss_constructions(
        &self,
        cst: &mut Vec<ConstructionType>,
        parent_selected: bool,
        hpts: &HashSet<PointId>,
        pts: &HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    ) -> Option<ConstructionType> {
        use BasicShapeType::*;
        match self.bs_typ {
            Line => {
                if let Some((pt1, pt2)) = self.bs_typ.segment_get_points(hpts, pts) {
                    let pattern = if parent_selected {
                        if self.selected {
                            ConstructionPattern::DoubleSelection
                        } else {
                            ConstructionPattern::SimpleSelection
                        }
                    } else {
                        ConstructionPattern::NoSelection
                    };
                    Some(ConstructionType::Segment(pattern, pt1.wpos, pt2.wpos))
                } else {
                    // Something when wrong, return None
                    None
                }
            }
            QBezier => None,    //TODO
            CBezier => None,    //TODO
            ArcEllipse => None, //TODO
        }
    }
    pub fn get_helpers_construction(
        &self,
        cst: &mut Vec<ConstructionType>,
        hpts: &HashSet<PointId>,
        pts: &HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    ) {
        use BasicShapeType::*;
        match self.bs_typ {
            Line => {
                if let Some((pt1, pt2)) = self.bs_typ.segment_get_points(hpts, pts) {
                    if is_aligned_vert(&pt1.wpos, &pt2.wpos) {
                        helper_vertical(&pt1.wpos, &pt2.wpos, true, cst)
                    }
                    if is_aligned_hori(&pt1.wpos, &pt2.wpos) {
                        helper_horizontal(&pt1.wpos, &pt2.wpos, true, cst)
                    }
                    if is_aligned_45_or_135(&pt1.wpos, &pt2.wpos) {
                        helper_45_135(&pt1.wpos, &pt2.wpos, true, cst)
                    }
                }
            }
            QBezier => (),    //TODO
            CBezier => (),    //TODO
            ArcEllipse => (), //TODO
        }
    }
}

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
    pub fn s_dist_seg(&self, pos1: &WPos, pos2: &WPos) -> f64 {
        // Calculate the projection
        let num =
            (pos2.wx - pos1.wx) * (pos1.wy - self.wy) - (pos1.wx - self.wx) * (pos2.wy - pos1.wy);
        let den = ((pos2.wx - pos1.wx).powi(2) + (pos2.wy - pos1.wy).powi(2)).sqrt();

        if den > 0. {
            let t = ((self.wx - pos1.wx) * (pos2.wx - pos1.wx)
                + (self.wy - pos1.wy) * (pos2.wy - pos1.wy))
                / den.powi(2);
            // Check if the projection is within the segment
            if t >= 0.0 && t <= 1.0 {
                // The projection is on the segment, return the signed perpendicular distance
                num / den
            } else {
                // The projection is not on the segment, find the nearest endpoint and determine the signed distance
                let dist_to_pos1 = self.dist(pos1);
                let dist_to_pos2 = self.dist(pos2);
                let nearest_dist = dist_to_pos1.min(dist_to_pos2);

                // Determine the sign based on the side of the extended line the point falls on
                if num > 0. {
                    nearest_dist // The point is on the "positive" side of the line
                } else {
                    -nearest_dist // The point is on the "negative" side of the line
                }
            }
        } else {
            // The segment is a point, return the signed distance to this point
            let direct_dist = self.dist(pos1);
            if num > 0. {
                direct_dist
            } else {
                -direct_dist
            }
        }
    }
    pub fn tup(&self) -> (f64, f64) {
        (self.wx, self.wy)
    }
    // // Find the projection of a point onto a line segment defined by two points
    // pub fn project_to_seg(&self, pos1: &WPos, pos2: &WPos) -> WPos {
    //     let pos_v = self - pos1;
    //     let dir_v = pos2 - pos1;
    //     *pos1 + dir_v * (pos_v.dot(&dir_v) / dir_v.norm2())
    // }
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
