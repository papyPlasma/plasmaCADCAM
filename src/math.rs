use crate::canvas::PlayingArea;
use crate::shapes::shapes::{ConstructionType, HandleType, Shape, ShapeType};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::{cell::RefMut, convert::TryInto};
use std::{
    collections::HashMap,
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    sync::atomic::{AtomicUsize, Ordering},
};
use web_sys::console;

const EPSILON: f64 = 1e-5; // Some small value
const MAX_ITERATIONS: usize = 100; // Or some other reasonable upper bound

static COUNTER_POINTS: AtomicUsize = AtomicUsize::new(0);
static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug)]
pub struct PointId {
    pt_id: usize,
}
impl Deref for PointId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.pt_id
    }
}
impl DerefMut for PointId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pt_id
    }
}
impl Hash for PointId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pt_id.hash(state);
    }
}
impl PartialEq for PointId {
    fn eq(&self, other: &Self) -> bool {
        self.pt_id == other.pt_id
    }
}
impl Eq for PointId {}

#[derive(Copy, Clone, Debug)]
pub struct ShapeId {
    sh_id: usize,
}
impl Deref for ShapeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.sh_id
    }
}
impl DerefMut for ShapeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sh_id
    }
}
impl Hash for ShapeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sh_id.hash(state);
    }
}
impl PartialEq for ShapeId {
    fn eq(&self, other: &Self) -> bool {
        self.sh_id == other.sh_id
    }
}
impl Eq for ShapeId {}

pub struct DataPool {
    pts_ids: HashMap<PointId, WPoint>,
    shs_ids: HashMap<ShapeId, Shape>,
}
impl DataPool {
    pub fn new() -> DataPool {
        DataPool {
            pts_ids: HashMap::new(),
            shs_ids: HashMap::new(),
        }
    }
    // Points
    pub fn insert_point(&mut self, pt: &WPoint) -> PointId {
        let pt_id = PointId {
            pt_id: COUNTER_POINTS.fetch_add(1, Ordering::Relaxed),
        };
        self.pts_ids.insert(pt_id.clone(), *pt);
        pt_id
    }
    pub fn modify_point(&mut self, pt_id: &PointId, pt: &WPoint) {
        self.pts_ids.insert(pt_id.clone(), *pt);
    }
    pub fn delete_point(&mut self, pt_id: &PointId) {
        self.pts_ids.remove(pt_id);
    }
    pub fn get_point(&self, pt_id: &PointId) -> Option<&WPoint> {
        self.pts_ids.get(pt_id)
    }

    // Shapes
    pub fn is_point_on_line(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Line = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(&pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let end = self.get_point(&pts.get(&HandleType::End).unwrap()).unwrap();
            point_on_segment(&start, &end, &pt, precision)
        } else {
            false
        }
    }
    pub fn is_point_on_quadbezier(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::QuadBezier = shape.get_type() {
            let mut t_min = 0.;
            let mut t_max = 1.;
            let mut min_dist = f64::MAX;
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(&pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let ctrl = self
                .get_point(&pts.get(&HandleType::Ctrl).unwrap())
                .unwrap();
            let end = self.get_point(&pts.get(&HandleType::End).unwrap()).unwrap();
            for _i in 0..MAX_ITERATIONS {
                // max iterations can be adjusted
                let t_mid = (t_min + t_max) / 2.;
                let bt = get_point_on_quad_bezier(t_mid, &start, &ctrl, &end);
                let dist = bt.dist(pt);
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
        } else {
            false
        }
    }
    pub fn is_point_on_cubicbezier(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::CubicBezier = shape.get_type() {
            let mut t_min = 0.;
            let mut t_max = 1.;
            let mut min_dist = f64::MAX;
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(&pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let ctrl1 = self
                .get_point(&pts.get(&HandleType::Ctrl1).unwrap())
                .unwrap();
            let ctrl2 = self
                .get_point(&pts.get(&HandleType::Ctrl2).unwrap())
                .unwrap();
            let end = self.get_point(&pts.get(&HandleType::End).unwrap()).unwrap();
            for _i in 0..MAX_ITERATIONS {
                let t_mid = (t_min + t_max) / 2.;
                let bt = get_point_on_cubic_bezier(t_mid, &start, &ctrl1, &ctrl2, &end);
                let dist = bt.dist(pt);
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
        } else {
            false
        }
    }
    pub fn is_point_on_rectangle(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Rectangle = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let bl = self.get_point(&pts.get(&HandleType::BL).unwrap()).unwrap();
            let tl = self.get_point(&pts.get(&HandleType::TL).unwrap()).unwrap();
            let tr = self.get_point(&pts.get(&HandleType::TR).unwrap()).unwrap();
            let br = self.get_point(&pts.get(&HandleType::BR).unwrap()).unwrap();
            return point_on_segment(&bl, &tl, &pt, precision)
                || point_on_segment(&tl, &tr, &pt, precision)
                || point_on_segment(&tr, &br, &pt, precision)
                || point_on_segment(&br, &bl, &pt, precision);
        } else {
            false
        }
    }
    fn ellipse_line_intersection(&self, center: &WPoint, radius: &WPoint, pt: &WPoint) -> WPoint {
        let m = (pt.wy - center.wy) / (pt.wx - center.wx);
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;

        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;

        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;

        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;

        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);

        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);

        if pt.dist(&pt1) < pt.dist(&pt2) {
            pt1
        } else {
            pt2
        }
    }
    fn ellipse_angle_intersection(&self, center: &WPoint, radius: &WPoint, angle: f64) -> WPoint {
        if angle == -PI / 2. {
            return WPoint {
                wx: 0.,
                wy: radius.wy,
            };
        }
        if angle == PI / 2. {
            return WPoint {
                wx: 0.,
                wy: -radius.wy,
            };
        }
        let m = angle.tan();
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;

        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;

        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;

        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;

        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);

        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);

        if angle >= 0. {
            if m >= 0. {
                pt1
            } else {
                pt2
            }
        } else {
            if m >= 0. {
                pt2
            } else {
                pt1
            }
        }
    }
    pub fn is_point_on_ellipse(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Ellipse = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let center = self
                .get_point(&pts.get(&HandleType::Center).unwrap())
                .unwrap();
            let radius = self
                .get_point(&pts.get(&HandleType::Radius).unwrap())
                .unwrap();
            let h_start_angle = self
                .get_point(&pts.get(&HandleType::StartAngle).unwrap())
                .unwrap();
            let h_end_angle = self
                .get_point(&pts.get(&HandleType::EndAngle).unwrap())
                .unwrap();

            // True radius
            let radius_pt = *radius - *center;
            let pt_int = self.ellipse_line_intersection(center, &radius_pt, pt);
            if pt_int.dist(pt) > precision {
                return false;
            }

            if h_start_angle.dist(h_end_angle) < 1. {
                return true;
            }
            let start_angle = -center.angle_on_ellipse(&h_start_angle, &radius);
            let end_angle = -center.angle_on_ellipse(&h_end_angle, &radius);
            let angle = -center.angle_on_ellipse(&pt, &radius);

            if end_angle > start_angle {
                angle >= start_angle && angle <= end_angle
            } else {
                !(angle >= end_angle && angle <= start_angle)
            }
        } else {
            false
        }
    }
    pub fn get_shape_id_from_pick_point(
        &self,
        pos: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<ShapeId> {
        for (shape_id, shape) in self.shs_ids.iter() {
            use ShapeType::*;
            let coord = shape.get_coord();
            match shape.get_type() {
                Line => {
                    if self.is_point_on_line(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                QuadBezier => {
                    if self.is_point_on_quadbezier(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                CubicBezier => {
                    if self.is_point_on_cubicbezier(&(*pos - *coord), shape, grab_handle_precision)
                    {
                        return Some(*shape_id);
                    }
                }
                Rectangle => {
                    if self.is_point_on_rectangle(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                Ellipse => {
                    if self.is_point_on_ellipse(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                Group => (),
            }
        }
        None
    }
    pub fn get_point_id_from_position(
        &self,
        pos: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<PointId> {
        for shape in self.shs_ids.values() {
            let coord = shape.get_coord();
            for (_, pt_id) in shape.get_handles_bundles().iter() {
                let pt = self.get_point(pt_id).unwrap();
                if is_point_on_point(pos, &(*pt + *coord), grab_handle_precision) {
                    return Some(pt_id.clone());
                }
            }
        }
        None
    }
    pub fn insert_shape(&mut self, shape: Shape) -> ShapeId {
        let shape_id = ShapeId {
            sh_id: COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed),
        };
        self.shs_ids.insert(shape_id, shape);
        shape_id
    }
    pub fn modify_shape(&mut self, shape_id: &ShapeId, shape: Shape) {
        self.shs_ids.insert(*shape_id, shape);
    }
    pub fn delete_shape(&mut self, shape_id: &ShapeId) {
        self.shs_ids.remove(shape_id);
    }
    pub fn select_shapes_bounded_by_rectangle(&mut self, bb: [WPoint; 2]) {
        // for shape in &mut pa_ref.pool.get_pool_shapes().values() {
        //     let bb_inner: [WPoint; 2] = shape.get_bounding_box();
        //     if is_box_inside(&bb_outer, &bb_inner) {
        //         shape.clear_any_shape_selection(Some(usize::All));
        //     } else {
        //         shape.clear_any_shape_selection(None);
        //     }
        // }
    }
    pub fn get_all_shapes_ids(&self) -> Vec<ShapeId> {
        self.shs_ids.keys().cloned().collect()
    }
    pub fn get_shape(&self, shape_id: &ShapeId) -> Option<&Shape> {
        self.shs_ids.get(shape_id)
    }
    pub fn get_shape_mut(&mut self, shape_id: &ShapeId) -> Option<&mut Shape> {
        self.shs_ids.get_mut(&shape_id)
    }
    pub fn get_shapes(&self) -> &HashMap<ShapeId, Shape> {
        &self.shs_ids
    }
    pub fn get_shapes_mut(&mut self) -> &mut HashMap<ShapeId, Shape> {
        &mut self.shs_ids
    }
}

fn is_vert(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wx - pt2.wx).abs() < 0.001
}
fn is_hori(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wy - pt2.wy).abs() < 0.001
}
fn is_45_135(pt1: &WPoint, pt2: &WPoint) -> bool {
    let dy = pt2.wy - pt1.wy;
    let dx = pt2.wx - pt1.wx;
    if dx != 0. {
        (dy / dx).abs() > 1. / 1.01 && (dy / dx).abs() < 1.01
    } else {
        false
    }
}

pub fn push_45_135(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    if is_45_135(pt1, pt2) {
        if full {
            use ConstructionType::*;
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: 2. * pt1.wx - pt2.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WPoint {
                wx: 2. * pt2.wx - pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        } else {
            use ConstructionType::*;
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: 2. * pt1.wx - pt2.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WPoint {
                wx: 2. * pt2.wx - pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        }
    }
}
pub fn push_vertical(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_vert(pt1, pt2) {
        if full {
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: pt1.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WPoint {
                wx: pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        } else {
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: pt1.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WPoint {
                wx: pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        }
    }
}
pub fn push_horizontal(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_hori(pt1, pt2) {
        if full {
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: 2. * pt1.wx - pt2.wx,
                wy: pt1.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WPoint {
                wx: 2. * pt2.wx - pt1.wx,
                wy: pt1.wy,
            }));
        } else {
            cst.push(Move(*pt1));
            cst.push(Line(WPoint {
                wx: 2. * pt1.wx - pt2.wx,
                wy: pt1.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WPoint {
                wx: 2. * pt2.wx - pt1.wx,
                wy: pt1.wy,
            }));
        }
    }
}
pub fn push_handle(pt: &WPoint, size_handle: f64, fill: bool, cst: &mut Vec<ConstructionType>) {
    let radius = WPoint::default() + size_handle / 2.;
    use ConstructionType::*;
    cst.push(Move(
        *pt + WPoint {
            wx: radius.wx,
            wy: 0.,
        },
    ));
    cst.push(Ellipse(*pt, radius, 0., 0., 2. * PI, fill));
}

pub fn magnet_geometry(pt1: &WPoint, pt2: &mut WPoint, snap_distance: f64) -> bool {
    let dx = (pt2.wx - pt1.wx).abs();
    let dy = (pt2.wy - pt1.wy).abs();

    let snap_distance = 2. * snap_distance;

    if dy < snap_distance {
        *pt2 = *pt1;
        true
    } else {
        if dx < snap_distance {
            *pt2 = *pt1;
            true
        } else {
            let x1 = pt1.wx;
            let y1 = pt1.wy;
            let x2 = pt2.wx;
            let y2 = pt2.wy;
            // Projection of p on (pt1, m=1)
            let p_proj = WPoint {
                wx: (x1 + x2 + y2 - y1) / 2.,
                wy: (-x1 + x2 + y2 + y1) / 2.,
            };
            if p_proj.dist(pt2) < snap_distance {
                *pt2 = p_proj;
                true
            } else {
                // Projection of p on (pt1, m=-1)
                let p_proj = WPoint {
                    wx: (x1 + x2 - y2 + y1) / 2.,
                    wy: (x1 - x2 + y2 + y1) / 2.,
                };

                if p_proj.dist(pt2) < snap_distance {
                    *pt2 = p_proj;
                    true
                } else {
                    false
                }
            }
        }
    }
}
pub fn snap_to_snap_grid(pos: &mut WPoint, snap_distance: f64) {
    *pos = (*pos / snap_distance).round() * snap_distance;
}
pub fn snap_to_snap_grid_y(pos: &mut WPoint, grid_spacing: f64) {
    pos.wy = (pos.wy / grid_spacing).round() * grid_spacing;
}
pub fn snap_to_snap_grid_x(pos: &mut WPoint, grid_spacing: f64) {
    pos.wx = (pos.wx / grid_spacing).round() * grid_spacing;
}

// pub fn snap_equidistant(handles: &mut Vec<WXY>, idx: &usize, idxs: &[usize; 2], snap_val: f64) {
//     let pt = handles[*idx];
//     let pt1 = handles[idxs[0]];
//     let pt2 = handles[idxs[1]];
//     let mid = (pt1 + pt2) / 2.0;
//     let dx = pt2.wx - pt1.wx;
//     let dy = pt2.wy - pt1.wy;
//     if dx == 0. && dy == 0. {
//         return;
//     }
//     let proj = if dx == 0. {
//         WXY {
//             wx: pt.wx,
//             wy: (pt2.wy + pt1.wy) / 2.,
//         }
//     } else {
//         if dy == 0. {
//             WXY {
//                 wx: (pt2.wx + pt1.wx) / 2.,
//                 wy: pt.wy,
//             }
//         } else {
//             let slope = dy / dx;
//             let perp_slope = -1. / slope;
//             let x_p = (perp_slope * mid.wx - slope * pt.wx + pt.wy - mid.wy) / (perp_slope - slope);
//             let y_p = perp_slope * (x_p - mid.wx) + mid.wy;
//             WXY { wx: x_p, wy: y_p }
//         }
//     };
//     if pt.dist(&proj) < snap_val {
//         handles[*idx] = proj;
//     }
// }

pub fn is_point_on_point(pt: &WPoint, pt1: &WPoint, precision: f64) -> bool {
    pt.dist(pt1) < precision
}
fn is_between(pt: &WPoint, pt1: &WPoint, pt2: &WPoint) -> bool {
    let dot_product = (pt.wx - pt1.wx) * (pt2.wx - pt1.wx) + (pt.wy - pt1.wy) * (pt2.wy - pt1.wy);
    if dot_product < 0. {
        return false;
    }
    let length2 = (pt2.wx - pt1.wx).powf(2.) + (pt2.wy - pt1.wy).powf(2.);
    if dot_product > length2 {
        return false;
    }
    return true;
}
fn point_on_segment(pt1: &WPoint, pt2: &WPoint, pt: &WPoint, precision: f64) -> bool {
    let denominator = ((pt2.wy - pt1.wy).powf(2.) + (pt2.wx - pt1.wx).powf(2.)).sqrt();
    if denominator == 0. {
        return is_point_on_point(pt, &pt1, precision);
    }
    let numerator = ((pt2.wy - pt1.wy) * pt.wx - (pt2.wx - pt1.wx) * pt.wy + pt2.wx * pt1.wy
        - pt2.wy * pt1.wx)
        .abs();

    if numerator / denominator > precision {
        return false;
    }
    is_between(pt, &pt1, &pt2)
}

pub fn is_box_inside(box_outer: &[WPoint; 2], box_inner: &[WPoint; 2]) -> bool {
    let bl_outer = box_outer[0];
    let tr_outer = box_outer[1];
    let bl_inner = box_inner[0];
    let tr_inner = box_inner[1];
    bl_inner.wx >= bl_outer.wx
        && bl_inner.wy >= bl_outer.wy
        && tr_inner.wx <= tr_outer.wx
        && tr_inner.wy <= tr_outer.wy
}
pub fn reorder_corners(bb: &mut [WPoint; 2]) {
    let pt1 = bb[0];
    let pt2 = bb[1];
    if pt1.wx < pt2.wx {
        if pt1.wy < pt2.wy {
            let bl = WPoint {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            let tr = WPoint {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WPoint {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            let tr = WPoint {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    } else {
        if pt1.wy < pt2.wy {
            let bl = WPoint {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            let tr = WPoint {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WPoint {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            let tr = WPoint {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    }
}

pub fn move_in_cubic_bezier(
    pa_mut: &mut RefMut<'_, PlayingArea>,
    shape_id: ShapeId,
    current_selected_point_id: &PointId,
    pick_pt: &WPoint,
    snap_distance: f64,
) {
}
pub fn move_in_rectangle(
    pa_mut: &mut RefMut<'_, PlayingArea>,
    shape_id: ShapeId,
    current_selected_point_id: &PointId,
    pick_pt: &WPoint,
    snap_distance: f64,
) {
    let shape = pa_mut.pool.get_shape(&shape_id).cloned().unwrap();
    let pts_ids = shape.get_handles_bundles();
    let bl_id = pts_ids.get(&HandleType::BL).unwrap();
    let tl_id = pts_ids.get(&HandleType::TL).unwrap();
    let tr_id = pts_ids.get(&HandleType::TR).unwrap();
    let br_id = pts_ids.get(&HandleType::BR).unwrap();
    let mut bl = pa_mut.pool.get_point(&bl_id).cloned().unwrap();
    let mut tl = pa_mut.pool.get_point(&tl_id).cloned().unwrap();
    let mut tr = pa_mut.pool.get_point(&tr_id).cloned().unwrap();
    let mut br = pa_mut.pool.get_point(&br_id).cloned().unwrap();
    if *current_selected_point_id == *bl_id {
        bl = *pick_pt;
        if bl.wx >= tr.wx {
            bl.wx = tr.wx - snap_distance;
        }
        if bl.wy <= tr.wy {
            bl.wy = tr.wy + snap_distance;
        }
        tl.wx = bl.wx;
        br.wy = bl.wy;
    }
    if *current_selected_point_id == *tl_id {
        tl = *pick_pt;
        if tl.wx >= br.wx {
            tl.wx = br.wx - snap_distance;
        }
        if tl.wy >= br.wy {
            tl.wy = br.wy - snap_distance;
        }
        tr.wy = tl.wy;
        bl.wx = tl.wx;
    }
    if *current_selected_point_id == *tr_id {
        tr = *pick_pt;
        if tr.wx <= bl.wx {
            tr.wx = bl.wx + snap_distance;
        }
        if tr.wy >= bl.wy {
            tr.wy = bl.wy - snap_distance;
        }
        tl.wy = tr.wy;
        br.wx = tr.wx;
    }
    if *current_selected_point_id == *br_id {
        br = *pick_pt;
        if br.wx <= tl.wx {
            br.wx = tl.wx + snap_distance;
        }
        if br.wy <= tl.wy {
            br.wy = tl.wy + snap_distance;
        }
        tr.wx = br.wx;
        bl.wy = br.wy;
    }
    pa_mut.pool.modify_point(&bl_id, &bl);
    pa_mut.pool.modify_point(&tl_id, &tl);
    pa_mut.pool.modify_point(&tr_id, &tr);
    pa_mut.pool.modify_point(&br_id, &br);
}

pub fn move_in_ellipse(
    pa_mut: &mut RefMut<'_, PlayingArea>,
    shape_id: ShapeId,
    current_selected_point_id: &PointId,
    rel_pick_pt: &WPoint,
    snap_distance: f64,
) {
    let shape = pa_mut.pool.get_shape(&shape_id).unwrap();
    let pts_ids = shape.get_handles_bundles().clone();

    let center_id = pts_ids.get(&HandleType::Center).unwrap();
    let radius_id = pts_ids.get(&HandleType::Radius).unwrap();
    let h_sa_id = pts_ids.get(&HandleType::StartAngle).unwrap();
    let h_ea_id = pts_ids.get(&HandleType::EndAngle).unwrap();

    let center = pa_mut.pool.get_point(&center_id).cloned().unwrap();
    let mut radius = pa_mut.pool.get_point(&radius_id).cloned().unwrap();
    let mut h_sa = pa_mut.pool.get_point(&h_sa_id).cloned().unwrap();
    let mut h_ea = pa_mut.pool.get_point(&h_ea_id).cloned().unwrap();

    let start_angle = get_atan2(&(h_sa - center));
    let end_angle = get_atan2(&(h_ea - center));

    let shape = pa_mut.pool.get_shape_mut(&shape_id).unwrap();
    let coord = shape.get_coord_mut();
    if *current_selected_point_id == *center_id {
        *coord = *coord + *rel_pick_pt;
    }

    if *current_selected_point_id == *radius_id {
        radius = *rel_pick_pt;
        if radius.wx <= center.wx {
            radius.wx = center.wx + snap_distance;
        }
        if radius.wy >= center.wy {
            radius.wy = center.wy - snap_distance;
        }
        h_sa = pa_mut
            .pool
            .ellipse_angle_intersection(&center, &radius, start_angle);
        h_ea = pa_mut
            .pool
            .ellipse_angle_intersection(&center, &radius, end_angle);
    }
    if *current_selected_point_id == *h_sa_id {
        let angle = get_atan2(&(*rel_pick_pt - center));
        h_sa = get_point_from_angle(&radius, angle);
    }
    if *current_selected_point_id == *h_ea_id {
        let angle = get_atan2(&(*rel_pick_pt - center));
        h_ea = get_point_from_angle(&radius, angle);
    }

    pa_mut.pool.modify_point(center_id, &center);
    pa_mut.pool.modify_point(radius_id, &radius);
    pa_mut.pool.modify_point(h_sa_id, &h_sa);
    pa_mut.pool.modify_point(h_ea_id, &h_ea);
}
//     magnet_geometry(&br, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tl = p;
//     if tl.wx >= br.wx {
//         tl.wx = br.wx - self.snap_distance;
//     }
//     if tl.wy >= br.wy {
//         tl.wy = br.wy - self.snap_distance;
//     }
//     tr.wy = tl.wy;
//     bl.wx = tl.wx;
// }
// TopRight => {
//     magnet_geometry(&bl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tr = p;
//     if tr.wx <= bl.wx {
//         tr.wx = bl.wx + self.snap_distance;
//     }
//     if tr.wy >= bl.wy {
//         tr.wy = bl.wy - self.snap_distance;
//     }
//     tl.wy = tr.wy;
//     br.wx = tr.wx;
// }
// BottomRight => {
//     magnet_geometry(&tl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     br = p;
//     if br.wx <= tl.wx {
//         br.wx = tl.wx + self.snap_distance;
//     }
//     if br.wy <= tl.wy {
//         br.wy = tl.wy + self.snap_distance;
//     }
//     tr.wx = br.wx;
//     bl.wy = br.wy;

// // Line spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_line(
//     pt: &WPoint,
//     shape: &SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::Line(start, end) = shape {
//         if is_point_on_point(pt, &start.1, EPSILON) || is_point_on_point(pt, &end.1, EPSILON) {
//             return None;
//         };
//         if is_point_on_line(pt, shape, EPSILON) {
//             Some((
//                 SimpleShape::Line((Handle::Start, start.1.clone()), (Handle::End, pt.clone())),
//                 SimpleShape::Line((Handle::Start, pt.clone()), (Handle::End, end.1.clone())),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Quad Bezier curve spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_quad_bezier(
//     pt: &WPoint,
//     shape: &SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::QuadBezier(start, ctrl, end) = shape {
//         if let Some(t) = find_t_for_point_on_quad_bezier(pt, &start.1, &ctrl.1, &end.1) {
//             let ctrl1 = start.1.lerp(&ctrl.1, t);
//             let ctrl2 = ctrl.1.lerp(&end.1, t);
//             let split = ctrl1.lerp(&ctrl2, t);
//             Some((
//                 SimpleShape::QuadBezier(
//                     (Handle::Start, start.1.clone()),
//                     (Handle::Ctrl, ctrl1),
//                     (Handle::End, split),
//                 ),
//                 SimpleShape::QuadBezier(
//                     (Handle::Start, split),
//                     (Handle::Ctrl, ctrl2),
//                     (Handle::End, end.1.clone()),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Cubic Bezier curve spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_cubic_bezier(
//     pt: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::CubicBezier(start, ctrl1, ctrl2, end) = shape {
//         if let Some(t) = find_t_for_point_on_cubic_bezier(pt, &start.1, &ctrl1.1, &ctrl2.1, &end.1)
//         {
//             let p0_prime = start.1.lerp(&ctrl1.1, t);
//             let p1_prime = ctrl1.1.lerp(&ctrl2.1, t);
//             let p2_prime = ctrl2.1.lerp(&end.1, t);
//             let q0 = p0_prime.lerp(&p1_prime, t);
//             let q1 = p1_prime.lerp(&p2_prime, t);
//             let r = q0.lerp(&q1, t);
//             Some((
//                 SimpleShape::CubicBezier(
//                     (Handle::Start, start.1.clone()),
//                     (Handle::Ctrl, p0_prime),
//                     (Handle::End, q0),
//                     (Handle::End, r.clone()),
//                 ),
//                 SimpleShape::CubicBezier(
//                     (Handle::Start, r),
//                     (Handle::Ctrl, q1),
//                     (Handle::End, p2_prime),
//                     (Handle::End, end.1.clone()),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Rectangle spliting (2 pts)
// #[allow(dead_code)]
// pub fn split_rectangle(
//     pt1: &WPoint,
//     pt2: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if pt1.dist(pt2) > EPSILON {
//         if let SimpleShape::Rectangle(bl, tl, tr, br) = shape {
//             let lines = vec![
//                 SimpleShape::Line((Handle::Start, bl), (Handle::End, tl)),
//                 SimpleShape::Line((Handle::Start, tl), (Handle::End, tr)),
//                 SimpleShape::Line((Handle::Start, tr), (Handle::End, br)),
//                 SimpleShape::Line((Handle::Start, br), (Handle::End, bl)),
//             ];
//             let mut oidx1 = None;
//             let mut oidx2 = None;
//             for (idx, line) in lines.iter().enumerate() {
//                 if let Some(v) = split_line(pt1, &line) {
//                     oidx1 = Some(idx);
//                     break;
//                 }
//             }
//             for (idx, line) in lines.iter().enumerate() {
//                 if let Some(v) = split_line(pt2, &line) {
//                     oidx2 = Some(idx);
//                     break;
//                 }
//             }
//             if let Some(idx1) = oidx1 {
//                 if let Some(idx2) = oidx2 {
//                     // TBD
//                     None
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//             //
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Ellipse curve splitting (2 pts)
// #[allow(dead_code)]
// pub fn split_ellipse(
//     pt1: &WPoint,
//     pt2: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::Ellipse(
//         center,
//         radius,
//         h_start_angle,
//         h_end_angle,
//         (rotation, start_angle, end_angle),
//     ) = shape
//     {
//         if pt1.dist(pt2) > EPSILON {
//             // Getting the angles for pt1 and pt2
//             let angle_pt1 = get_angle_from_point(pt1, &center.1, rotation);
//             let angle_pt2 = get_angle_from_point(pt2, &center.1, rotation);
//             // Ensuring angle_pt1 is smaller than angle_pt2
//             let (min_angle, max_angle) = if angle_pt1 < angle_pt2 {
//                 (angle_pt1, angle_pt2)
//             } else {
//                 (angle_pt2, angle_pt1)
//             };
//             let h_start_angle = get_point_from_angle(&center.1, &radius.1, rotation, -start_angle);
//             let h_min_angle = get_point_from_angle(&center.1, &radius.1, rotation, -min_angle);
//             let h_max_angle = get_point_from_angle(&center.1, &radius.1, rotation, -max_angle);
//             let h_end_angle = get_point_from_angle(&center.1, &radius.1, rotation, -end_angle);
//             Some((
//                 SimpleShape::Ellipse(
//                     (Handle::Center, center.1.clone()),
//                     (Handle::End, center.1.clone() + radius.1.clone()),
//                     (Handle::StartAngle, h_start_angle.addxy(center.1.wx, 0.)),
//                     (Handle::EndAngle, h_min_angle.addxy(center.1.wx, 0.)),
//                     (rotation, start_angle, min_angle),
//                 ),
//                 SimpleShape::Ellipse(
//                     (Handle::Center, center.1.clone()),
//                     (Handle::End, center.1.clone() + radius.1.clone()),
//                     (Handle::StartAngle, h_max_angle.addxy(center.1.wx, 0.)),
//                     (Handle::EndAngle, h_end_angle.addxy(center.1.wx, 0.)),
//                     (rotation, max_angle, end_angle),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }

fn get_point_on_quad_bezier(t: f64, start: &WPoint, ctrl: &WPoint, end: &WPoint) -> WPoint {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    let mut result = WPoint::default();
    result.wx = uu * start.wx + 2.0 * u * t * ctrl.wx + tt * end.wx;
    result.wy = uu * start.wy + 2.0 * u * t * ctrl.wy + tt * end.wy;

    result
}
fn get_point_on_cubic_bezier(
    t: f64,
    start: &WPoint,
    ctrl1: &WPoint,
    ctrl2: &WPoint,
    end: &WPoint,
) -> WPoint {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut result = *start * uuu; // (1-t)^3 * start
    result += *ctrl1 * 3.0 * uu * t; // 3(1-t)^2 * t * ctrl1
    result += *ctrl2 * 3.0 * u * tt; // 3(1-t) * t^2 * ctrl2
    result += *end * ttt; // t^3 * end

    result
}
#[inline]
pub fn get_atan2(point: &WPoint) -> f64 {
    point.wy.atan2(point.wx)
}
#[inline]
pub fn get_point_from_angle(radius: &WPoint, angle: f64) -> WPoint {
    let x = radius.wx.abs() * angle.cos();
    let y = radius.wy.abs() * angle.sin();
    WPoint { wx: x, wy: y }
}

#[allow(dead_code)]
fn find_t_for_point_on_quad_bezier(
    p: &WPoint,
    start: &WPoint,
    ctrl: &WPoint,
    end: &WPoint,
) -> Option<f64> {
    let mut t_min = 0.0;
    let mut t_max = 1.0;

    for _ in 0..MAX_ITERATIONS {
        let t_mid = (t_min + t_max) / 2.0;
        let mid_point = get_point_on_quad_bezier(t_mid, start, ctrl, end);

        let dist = mid_point.dist(p);
        if dist < EPSILON {
            return Some(t_mid);
        }

        if get_point_on_quad_bezier((t_min + t_mid) / 2.0, start, ctrl, end).dist(p) < dist {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }

    None
}

#[allow(dead_code)]
fn find_t_for_point_on_cubic_bezier(
    p: &WPoint,
    start: &WPoint,
    ctrl1: &WPoint,
    ctrl2: &WPoint,
    end: &WPoint,
) -> Option<f64> {
    let mut t_min = 0.0;
    let mut t_max = 1.0;

    for _ in 0..MAX_ITERATIONS {
        let t_mid = (t_min + t_max) / 2.0;
        let mid_point = get_point_on_cubic_bezier(t_mid, start, ctrl1, ctrl2, end);

        let dist = mid_point.dist(p);
        if dist < EPSILON {
            return Some(t_mid);
        }

        if get_point_on_cubic_bezier((t_min + t_mid) / 2.0, start, ctrl1, ctrl2, end).dist(p) < dist
        {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }

    None
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
    pub fn angle_on_ellipse(&self, point: &WPoint, radius: &WPoint) -> f64 {
        f64::atan2(
            (point.wy - self.wy) / radius.wy,
            (point.wx - self.wx) / radius.wx,
        )
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
